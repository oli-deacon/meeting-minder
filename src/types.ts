export type SessionStatus = "recording" | "recorded" | "processing" | "analyzed" | "error";
export type TranscriptionStatus = "notStarted" | "processing" | "completed" | "error";

export interface Session {
  id: string;
  startedAt: string;
  endedAt?: string;
  audioPath: string;
  status: SessionStatus;
  transcriptionStatus: TranscriptionStatus;
  transcriptionError?: string;
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

export interface TranscriptSegment {
  startSec: number;
  endSec: number;
  textEn: string;
  sourceLanguage: string;
  speakerId?: string;
}

export interface TranscriptResult {
  sessionId: string;
  segments: TranscriptSegment[];
  fullTextEn: string;
  modelVersion: string;
  processingMs: number;
}

export interface SessionDetails {
  session: Session;
  analysis?: AnalysisResult;
  transcript?: TranscriptResult;
}
