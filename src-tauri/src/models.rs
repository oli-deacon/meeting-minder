use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Recording,
    Recorded,
    Processing,
    Analyzed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum TranscriptionStatus {
    NotStarted,
    Processing,
    Completed,
    Error,
}

impl Default for TranscriptionStatus {
    fn default() -> Self {
        Self::NotStarted
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub audio_path: String,
    pub status: SessionStatus,
    #[serde(default)]
    pub transcription_status: TranscriptionStatus,
    #[serde(default)]
    pub transcription_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Segment {
    pub start_sec: f64,
    pub end_sec: f64,
    pub speaker_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerStats {
    pub speaker_id: String,
    pub total_sec: f64,
    pub percentage: f64,
    pub segment_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisMeta {
    pub total_speech_sec: f64,
    pub processing_ms: u64,
    pub model_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisResult {
    pub session_id: String,
    pub total_speech_sec: f64,
    pub speakers: Vec<SpeakerStats>,
    pub segments: Vec<Segment>,
    pub meta: AnalysisMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSegment {
    pub start_sec: f64,
    pub end_sec: f64,
    pub text_en: String,
    pub source_language: String,
    pub speaker_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptResult {
    pub session_id: String,
    pub segments: Vec<TranscriptSegment>,
    pub full_text_en: String,
    pub model_version: String,
    pub processing_ms: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetails {
    pub session: Session,
    pub analysis: Option<AnalysisResult>,
    pub transcript: Option<TranscriptResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRecordingResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StopRecordingResponse {
    pub session: Session,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPaths {
    pub csv_path: Option<String>,
    pub json_path: Option<String>,
    pub transcript_json_path: Option<String>,
    pub transcript_txt_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRecordingResponse {
    pub session: Session,
}
