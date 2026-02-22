import { invoke } from "@tauri-apps/api/core";
import type { AnalysisResult, Session, SessionDetails } from "../types";

export interface StartRecordingResponse {
  session: Session;
}

export interface StopRecordingResponse {
  session: Session;
}

export async function startRecording(): Promise<StartRecordingResponse> {
  return invoke("start_recording");
}

export async function stopRecording(sessionId: string): Promise<StopRecordingResponse> {
  return invoke("stop_recording", { sessionId });
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

export async function exportSession(sessionId: string): Promise<{ csvPath: string; jsonPath: string }> {
  return invoke("export_session", { sessionId });
}
