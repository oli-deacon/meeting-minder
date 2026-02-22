use crate::models::{AnalysisResult, Session};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
    fs::create_dir_all(&base).map_err(|e| format!("failed to create app data dir: {e}"))?;
    Ok(base)
}

pub fn sessions_root(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app_data_dir(app)?.join("sessions");
    fs::create_dir_all(&root).map_err(|e| format!("failed to create sessions dir: {e}"))?;
    Ok(root)
}

pub fn session_dir(app: &AppHandle, session_id: &str) -> Result<PathBuf, String> {
    let dir = sessions_root(app)?.join(session_id);
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create session dir: {e}"))?;
    Ok(dir)
}

pub fn session_json_path(session_dir: &Path) -> PathBuf {
    session_dir.join("session.json")
}

pub fn analysis_json_path(session_dir: &Path) -> PathBuf {
    session_dir.join("analysis.json")
}

pub fn audio_wav_path(session_dir: &Path) -> PathBuf {
    session_dir.join("audio.wav")
}

pub fn load_session(path: &Path) -> Result<Session, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read session json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("failed to parse session json: {e}"))
}

pub fn save_session(path: &Path, session: &Session) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(session)
        .map_err(|e| format!("failed to serialize session json: {e}"))?;
    fs::write(path, raw).map_err(|e| format!("failed to write session json: {e}"))
}

pub fn load_analysis(path: &Path) -> Result<AnalysisResult, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("failed to read analysis json: {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("failed to parse analysis json: {e}"))
}

pub fn save_analysis(path: &Path, analysis: &AnalysisResult) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(analysis)
        .map_err(|e| format!("failed to serialize analysis json: {e}"))?;
    fs::write(path, raw).map_err(|e| format!("failed to write analysis json: {e}"))
}
