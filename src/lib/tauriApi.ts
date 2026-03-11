import { invoke } from "@tauri-apps/api/core";
import type { AnalysisResult, Session, SessionDetails, TranscriptResult } from "../types";

export interface StartRecordingResponse {
  session: Session;
}

export interface StopRecordingResponse {
  session: Session;
}

export interface ImportRecordingResponse {
  session: Session;
}

export interface ExportSessionResponse {
  csvPath?: string;
  jsonPath?: string;
  transcriptJsonPath?: string;
  transcriptTxtPath?: string;
}

export async function startRecording(): Promise<StartRecordingResponse> {
  return invoke("start_recording");
}

export async function stopRecording(sessionId: string): Promise<StopRecordingResponse> {
  return invoke("stop_recording", { sessionId });
}

export async function importWavRecording(path: string): Promise<ImportRecordingResponse> {
  return invoke("import_wav_recording", { path });
}

export async function listSessions(): Promise<Session[]> {
  return invoke("list_sessions");
}

export async function getSessionDetails(sessionId: string): Promise<SessionDetails> {
  return invoke("get_session_details", { sessionId });
}

export async function analyzeSession(sessionId: string): Promise<AnalysisResult> {
  return invoke("analyze_session", { sessionId });
}

export async function transcribeSession(sessionId: string): Promise<TranscriptResult> {
  return invoke("transcribe_session", { sessionId });
}

export async function getTranscript(sessionId: string): Promise<TranscriptResult> {
  return invoke("get_transcript", { sessionId });
}

export async function exportSession(sessionId: string): Promise<ExportSessionResponse> {
  return invoke("export_session", { sessionId });
}

export async function deleteSession(sessionId: string): Promise<void> {
  return invoke("delete_session", { sessionId });
}
