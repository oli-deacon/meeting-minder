export type SessionStatus = "recording" | "recorded" | "processing" | "analyzed" | "error";

export interface Session {
  id: string;
  startedAt: string;
  endedAt?: string;
  audioPath: string;
  status: SessionStatus;
}

export interface Segment {
  startSec: number;
  endSec: number;
  speakerId: string;
}

export interface SpeakerStats {
  speakerId: string;
  totalSec: number;
  percentage: number;
  segmentCount: number;
}

export interface AnalysisMeta {
  totalSpeechSec: number;
  processingMs: number;
  modelVersion: string;
}

export interface AnalysisResult {
  sessionId: string;
  totalSpeechSec: number;
  speakers: SpeakerStats[];
  segments: Segment[];
  meta: AnalysisMeta;
}

export interface SessionDetails {
  session: Session;
  analysis?: AnalysisResult;
}
