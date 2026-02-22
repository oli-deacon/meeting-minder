mod analyzer;
mod models;
mod recorder;
mod state;
mod storage;

use analyzer::run_python_analyzer;
use chrono::Utc;
use models::{
    AnalysisResult, ExportPaths, Session, SessionDetails, SessionStatus, StartRecordingResponse,
    StopRecordingResponse,
};
use state::{ActiveRecording, AppState};
use std::fs;
use std::sync::mpsc;
use storage::{
    analysis_json_path, audio_wav_path, load_analysis, load_session, save_session, session_dir,
    session_json_path, sessions_root,
};
use tauri::State;
use uuid::Uuid;

fn collect_sessions(app: &tauri::AppHandle) -> Result<Vec<Session>, String> {
    let root = sessions_root(app)?;
    let mut sessions: Vec<Session> = Vec::new();

    for entry in fs::read_dir(root).map_err(|e| format!("failed to read sessions root: {e}"))? {
        let entry = entry.map_err(|e| format!("failed to read session entry: {e}"))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let session_path = session_json_path(&path);
        if !session_path.exists() {
            continue;
        }

        let session = load_session(&session_path)?;
        sessions.push(session);
    }

    sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(sessions)
}

#[tauri::command]
fn list_sessions(app: tauri::AppHandle) -> Result<Vec<Session>, String> {
    collect_sessions(&app)
}

#[tauri::command]
fn get_session_details(app: tauri::AppHandle, session_id: String) -> Result<SessionDetails, String> {
    let dir = session_dir(&app, &session_id)?;
    let session_path = session_json_path(&dir);
    let analysis_path = analysis_json_path(&dir);

    let session = load_session(&session_path)?;
    let analysis = if analysis_path.exists() {
        Some(load_analysis(&analysis_path)?)
    } else {
        None
    };

    Ok(SessionDetails { session, analysis })
}

#[tauri::command]
fn start_recording(
    app: tauri::AppHandle,
    state: State<AppState>,
) -> Result<StartRecordingResponse, String> {
    let mut active_guard = state
        .active_recording
        .lock()
        .map_err(|_| "failed to lock active recording state".to_string())?;

    if active_guard.is_some() {
        return Err("a recording session is already active".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let dir = session_dir(&app, &id)?;
    let audio_path = audio_wav_path(&dir);

    let session = Session {
        id: id.clone(),
        started_at: Utc::now().to_rfc3339(),
        ended_at: None,
        audio_path: audio_path.to_string_lossy().to_string(),
        status: SessionStatus::Recording,
    };

    save_session(&session_json_path(&dir), &session)?;

    let (stop_tx, stop_rx) = mpsc::channel();
    let thread_audio_path = audio_path.clone();
    let join_handle = std::thread::spawn(move || recorder::record_until_stopped(thread_audio_path, stop_rx));

    *active_guard = Some(ActiveRecording {
        session_id: id,
        stop_tx,
        join_handle: Some(join_handle),
    });

    Ok(StartRecordingResponse { session })
}

#[tauri::command]
fn stop_recording(
    app: tauri::AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<StopRecordingResponse, String> {
    let mut active_guard = state
        .active_recording
        .lock()
        .map_err(|_| "failed to lock active recording state".to_string())?;

    let mut active = active_guard
        .take()
        .ok_or_else(|| "no active recording session".to_string())?;

    if active.session_id != session_id {
        *active_guard = Some(active);
        return Err("requested session does not match active session".to_string());
    }

    active
        .stop_tx
        .send(())
        .map_err(|e| format!("failed to send stop signal: {e}"))?;

    let join_result = active
        .join_handle
        .take()
        .ok_or_else(|| "missing recording thread handle".to_string())?
        .join()
        .map_err(|_| "recording thread panicked".to_string())?;

    let dir = session_dir(&app, &session_id)?;
    let session_path = session_json_path(&dir);
    let mut session = load_session(&session_path)?;
    session.ended_at = Some(Utc::now().to_rfc3339());

    match join_result {
        Ok(()) => {
            session.status = SessionStatus::Recorded;
            save_session(&session_path, &session)?;
            Ok(StopRecordingResponse { session })
        }
        Err(err) => {
            session.status = SessionStatus::Error;
            save_session(&session_path, &session)?;
            Err(err)
        }
    }
}

#[tauri::command]
fn analyze_session(
    app: tauri::AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<AnalysisResult, String> {
    let active_guard = state
        .active_recording
        .lock()
        .map_err(|_| "failed to lock active recording state".to_string())?;
    if let Some(active) = active_guard.as_ref() {
        if active.session_id == session_id {
            return Err("cannot analyze while session is recording".to_string());
        }
    }
    drop(active_guard);

    let dir = session_dir(&app, &session_id)?;
    let session_path = session_json_path(&dir);
    let analysis_path = analysis_json_path(&dir);

    let mut session = load_session(&session_path)?;
    session.status = SessionStatus::Processing;
    save_session(&session_path, &session)?;

    let input_audio = std::path::PathBuf::from(&session.audio_path);
    let result = run_python_analyzer(&input_audio, &analysis_path);

    match result {
        Ok(()) => {
            let analysis = load_analysis(&analysis_path)?;
            session.status = SessionStatus::Analyzed;
            save_session(&session_path, &session)?;
            Ok(analysis)
        }
        Err(err) => {
            session.status = SessionStatus::Error;
            save_session(&session_path, &session)?;
            Err(err)
        }
    }
}

#[tauri::command]
fn export_session(app: tauri::AppHandle, session_id: String) -> Result<ExportPaths, String> {
    let dir = session_dir(&app, &session_id)?;
    let analysis_path = analysis_json_path(&dir);
    if !analysis_path.exists() {
        return Err("analysis not found for session".to_string());
    }
    let analysis = load_analysis(&analysis_path)?;

    let csv_path = dir.join("analysis.csv");
    let mut csv = String::from("speakerId,totalSec,percentage,segmentCount\n");
    for speaker in analysis.speakers {
        csv.push_str(&format!(
            "{},{:.4},{:.4},{}\n",
            speaker.speaker_id, speaker.total_sec, speaker.percentage, speaker.segment_count
        ));
    }

    fs::write(&csv_path, csv).map_err(|e| format!("failed to write csv export: {e}"))?;

    Ok(ExportPaths {
        csv_path: csv_path.to_string_lossy().to_string(),
        json_path: analysis_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
fn delete_session(
    app: tauri::AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<(), String> {
    let active_guard = state
        .active_recording
        .lock()
        .map_err(|_| "failed to lock active recording state".to_string())?;
    if let Some(active) = active_guard.as_ref() {
        if active.session_id == session_id {
            return Err("cannot delete while session is recording".to_string());
        }
    }
    drop(active_guard);

    let dir = sessions_root(&app)?.join(&session_id);
    if !dir.exists() {
        return Ok(());
    }

    fs::remove_dir_all(&dir).map_err(|e| format!("failed to delete session dir: {e}"))?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            start_recording,
            stop_recording,
            list_sessions,
            get_session_details,
            analyze_session,
            export_session,
            delete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
