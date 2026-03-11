mod analyzer;
mod models;
mod recorder;
mod state;
mod storage;
mod transcriber;

use analyzer::run_python_analyzer;
use chrono::Utc;
use models::{
    AnalysisResult, ExportPaths, ImportRecordingResponse, Session, SessionDetails, SessionStatus,
    StartRecordingResponse, StopRecordingResponse, TranscriptResult, TranscriptionStatus,
};
use state::{ActiveRecording, AppState};
use std::fs;
use std::sync::mpsc;
use storage::{
    analysis_json_path, audio_wav_path, load_analysis, load_session, load_transcript, save_session,
    session_dir, session_json_path, sessions_root, transcript_json_path, transcript_txt_path,
};
use tauri::State;
use transcriber::run_python_transcriber;
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
    let transcript_path = transcript_json_path(&dir);

    let session = load_session(&session_path)?;
    let analysis = if analysis_path.exists() {
        Some(load_analysis(&analysis_path)?)
    } else {
        None
    };
    let transcript = if transcript_path.exists() {
        Some(load_transcript(&transcript_path)?)
    } else {
        None
    };

    Ok(SessionDetails {
        session,
        analysis,
        transcript,
    })
}

#[tauri::command]
fn get_transcript(app: tauri::AppHandle, session_id: String) -> Result<TranscriptResult, String> {
    let dir = session_dir(&app, &session_id)?;
    let transcript_path = transcript_json_path(&dir);
    if !transcript_path.exists() {
        return Err("transcript not found for session".to_string());
    }
    load_transcript(&transcript_path)
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
        transcription_status: TranscriptionStatus::NotStarted,
        transcription_error: None,
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
fn import_wav_recording(app: tauri::AppHandle, path: String) -> Result<ImportRecordingResponse, String> {
    let source = std::path::PathBuf::from(path);
    if !source.exists() {
        return Err("wav file does not exist".to_string());
    }
    let ext = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if ext != "wav" {
        return Err("only .wav files are supported for import".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let dir = session_dir(&app, &id)?;
    let dest_audio = audio_wav_path(&dir);
    fs::copy(&source, &dest_audio).map_err(|e| format!("failed to copy wav file: {e}"))?;

    let now = Utc::now().to_rfc3339();
    let session = Session {
        id,
        started_at: now.clone(),
        ended_at: Some(now),
        audio_path: dest_audio.to_string_lossy().to_string(),
        status: SessionStatus::Recorded,
        transcription_status: TranscriptionStatus::NotStarted,
        transcription_error: None,
    };

    save_session(&session_json_path(&dir), &session)?;
    Ok(ImportRecordingResponse { session })
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
fn transcribe_session(
    app: tauri::AppHandle,
    state: State<AppState>,
    session_id: String,
) -> Result<TranscriptResult, String> {
    let active_guard = state
        .active_recording
        .lock()
        .map_err(|_| "failed to lock active recording state".to_string())?;
    if let Some(active) = active_guard.as_ref() {
        if active.session_id == session_id {
            return Err("cannot transcribe while session is recording".to_string());
        }
    }
    drop(active_guard);

    let dir = session_dir(&app, &session_id)?;
    let session_path = session_json_path(&dir);
    let analysis_path = analysis_json_path(&dir);
    let transcript_path = transcript_json_path(&dir);
    let transcript_txt = transcript_txt_path(&dir);

    let mut session = load_session(&session_path)?;
    session.transcription_status = TranscriptionStatus::Processing;
    session.transcription_error = None;
    save_session(&session_path, &session)?;

    let input_audio = std::path::PathBuf::from(&session.audio_path);
    let result = run_python_transcriber(
        &input_audio,
        &transcript_path,
        &transcript_txt,
        if analysis_path.exists() {
            Some(analysis_path.as_path())
        } else {
            None
        },
    );

    match result {
        Ok(()) => {
            let transcript = load_transcript(&transcript_path)?;
            session.transcription_status = TranscriptionStatus::Completed;
            session.transcription_error = None;
            save_session(&session_path, &session)?;
            Ok(transcript)
        }
        Err(err) => {
            session.transcription_status = TranscriptionStatus::Error;
            session.transcription_error = Some(err.clone());
            save_session(&session_path, &session)?;
            Err(err)
        }
    }
}

#[tauri::command]
fn export_session(app: tauri::AppHandle, session_id: String) -> Result<ExportPaths, String> {
    let dir = session_dir(&app, &session_id)?;
    let analysis_path = analysis_json_path(&dir);
    let mut csv_path: Option<String> = None;
    let mut json_path: Option<String> = None;

    if analysis_path.exists() {
        let analysis = load_analysis(&analysis_path)?;
        let generated_csv = dir.join("analysis.csv");
        let mut csv = String::from("speakerId,totalSec,percentage,segmentCount\n");
        for speaker in analysis.speakers {
            csv.push_str(&format!(
                "{},{:.4},{:.4},{}\n",
                speaker.speaker_id, speaker.total_sec, speaker.percentage, speaker.segment_count
            ));
        }
        fs::write(&generated_csv, csv).map_err(|e| format!("failed to write csv export: {e}"))?;
        csv_path = Some(generated_csv.to_string_lossy().to_string());
        json_path = Some(analysis_path.to_string_lossy().to_string());
    }

    let transcript_json = transcript_json_path(&dir);
    let transcript_txt = transcript_txt_path(&dir);
    if csv_path.is_none() && !transcript_json.exists() && !transcript_txt.exists() {
        return Err("nothing to export for session (analysis/transcript missing)".to_string());
    }

    Ok(ExportPaths {
        csv_path,
        json_path,
        transcript_json_path: if transcript_json.exists() {
            Some(transcript_json.to_string_lossy().to_string())
        } else {
            None
        },
        transcript_txt_path: if transcript_txt.exists() {
            Some(transcript_txt.to_string_lossy().to_string())
        } else {
            None
        },
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
            import_wav_recording,
            list_sessions,
            get_session_details,
            get_transcript,
            analyze_session,
            transcribe_session,
            export_session,
            delete_session
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
