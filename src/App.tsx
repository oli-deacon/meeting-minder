import { useEffect, useMemo, useState } from "react";
import { exportSession } from "./lib/tauriApi";
import { formatSeconds, isoToDisplay } from "./lib/format";
import { useSessions } from "./hooks/useSessions";
import { SpeakerBars } from "./components/SpeakerBars";
import { RecordingPromptPanel } from "./components/RecordingPromptPanel";
import { SessionsList } from "./components/SessionsList";
import { pickSessionStarters, PROMPTS_PER_SESSION, STATEMENT_STARTERS } from "./lib/statementStarters";

export function App() {
  const {
    sessions,
    selectedId,
    setSelectedId,
    details,
    loading,
    error,
    recordingSession,
    onStartRecording,
    onImportWav,
    onStopRecording,
    onAnalyze,
    onTranscribe,
    onDelete
  } = useSessions();
  const [exportMessage, setExportMessage] = useState<string | null>(null);
  const [pendingDeleteSessionId, setPendingDeleteSessionId] = useState<string | null>(null);
  const [sessionPrompts, setSessionPrompts] = useState<string[]>([]);
  const [isPromptPanelCollapsed, setIsPromptPanelCollapsed] = useState(false);

  const selectedSession = details?.session;
  const analysis = details?.analysis;
  const transcript = details?.transcript;

  const canAnalyze = useMemo(() => {
    if (!selectedSession) {
      return false;
    }
    return selectedSession.status !== "recording" && selectedSession.status !== "processing";
  }, [selectedSession]);

  const canTranscribe = useMemo(() => {
    if (!selectedSession) {
      return false;
    }
    return selectedSession.status !== "recording" && selectedSession.transcriptionStatus !== "processing";
  }, [selectedSession]);

  const canDelete = useMemo(() => {
    if (!selectedSession) {
      return false;
    }
    return selectedSession.status !== "recording" && selectedSession.status !== "processing";
  }, [selectedSession]);

  const onExport = async () => {
    if (!selectedSession) {
      return;
    }
    try {
      const paths = await exportSession(selectedSession.id);
      const parts: string[] = [];
      if (paths.csvPath) {
        parts.push(`Exported CSV: ${paths.csvPath}`);
      }
      if (paths.jsonPath) {
        parts.push(`Analysis JSON: ${paths.jsonPath}`);
      }
      if (paths.transcriptJsonPath) {
        parts.push(`Transcript JSON: ${paths.transcriptJsonPath}`);
      }
      if (paths.transcriptTxtPath) {
        parts.push(`Transcript TXT: ${paths.transcriptTxtPath}`);
      }
      setExportMessage(parts.join(" | "));
    } catch (err) {
      setExportMessage(err instanceof Error ? err.message : "Export failed");
    }
  };

  const onImport = async () => {
    const path = window.prompt("Enter absolute path to .wav file:");
    if (!path) {
      return;
    }
    await onImportWav(path.trim());
  };

  const onDeleteSession = async () => {
    if (!selectedSession || !canDelete) {
      return;
    }
    setPendingDeleteSessionId(selectedSession.id);
  };

  const onConfirmDelete = async () => {
    if (!selectedSession || pendingDeleteSessionId !== selectedSession.id) {
      return;
    }
    await onDelete(selectedSession.id);
    setPendingDeleteSessionId(null);
    setExportMessage(null);
  };

  useEffect(() => {
    if (!selectedSession || pendingDeleteSessionId !== selectedSession.id) {
      setPendingDeleteSessionId(null);
    }
  }, [selectedSession, pendingDeleteSessionId]);

  useEffect(() => {
    if (!recordingSession) {
      setSessionPrompts([]);
      setIsPromptPanelCollapsed(false);
      return;
    }
    setSessionPrompts(pickSessionStarters(STATEMENT_STARTERS, PROMPTS_PER_SESSION));
    setIsPromptPanelCollapsed(false);
  }, [recordingSession?.id]);

  return (
    <main className="layout">
      <header className="header">
        <div>
          <h1>Meeting Minder</h1>
          <p>Local meeting recorder with offline Thai-to-English transcription and speaker-time percentages.</p>
        </div>
        <div className="actions">
          <button type="button" onClick={onStartRecording} disabled={loading || !!recordingSession}>
            Start Recording
          </button>
          <button type="button" onClick={onStopRecording} disabled={loading || !recordingSession}>
            Stop Recording
          </button>
          <button type="button" onClick={() => void onImport()} disabled={loading || !!recordingSession}>
            Import WAV
          </button>
        </div>
      </header>

      {recordingSession && (
        <section className="recording-banner">Recording in progress: session {recordingSession.id}</section>
      )}
      <RecordingPromptPanel
        visible={!!recordingSession}
        sessionId={recordingSession?.id ?? ""}
        prompts={sessionPrompts}
        isCollapsed={isPromptPanelCollapsed}
        onToggleCollapse={() => setIsPromptPanelCollapsed((current) => !current)}
      />

      {error && <section className="error-banner">{error}</section>}

      <section className="grid">
        <aside className="panel">
          <h2>Sessions</h2>
          <SessionsList sessions={sessions} selectedId={selectedId} onSelect={setSelectedId} />
        </aside>

        <section className="panel">
          <h2>Details</h2>
          {!selectedSession && <p className="empty-state">Select a session.</p>}

          {selectedSession && (
            <div className="session-details">
              <p>
                <strong>ID:</strong> {selectedSession.id}
              </p>
              <p>
                <strong>Started:</strong> {isoToDisplay(selectedSession.startedAt)}
              </p>
              {selectedSession.endedAt && (
                <p>
                  <strong>Ended:</strong> {isoToDisplay(selectedSession.endedAt)}
                </p>
              )}
              <p>
                <strong>Status:</strong> {selectedSession.status}
              </p>
              <p>
                <strong>Transcription:</strong> {selectedSession.transcriptionStatus}
              </p>
              {selectedSession.transcriptionError && (
                <p className="hint">
                  <strong>Transcription Error:</strong> {selectedSession.transcriptionError}
                </p>
              )}
              <p>
                <strong>Audio Path:</strong> {selectedSession.audioPath}
              </p>

              <div className="actions">
                <button
                  type="button"
                  onClick={() => selectedSession && void onAnalyze(selectedSession.id)}
                  disabled={!canAnalyze || loading}
                >
                  Analyze Session
                </button>
                <button
                  type="button"
                  onClick={() => selectedSession && void onTranscribe(selectedSession.id)}
                  disabled={!canTranscribe || loading}
                >
                  Transcribe to English
                </button>
                <button type="button" onClick={onExport} disabled={(!analysis && !transcript) || loading}>
                  Export CSV/JSON
                </button>
                <button
                  type="button"
                  className="danger-button"
                  onClick={() => void onDeleteSession()}
                  disabled={!canDelete || loading}
                >
                  Delete Recording
                </button>
              </div>

              {pendingDeleteSessionId === selectedSession.id && (
                <div className="confirm-delete">
                  <p>Sure you want to delete this recording? This cannot be undone.</p>
                  <div className="actions">
                    <button type="button" className="danger-button" onClick={() => void onConfirmDelete()} disabled={loading}>
                      Yes, Delete
                    </button>
                    <button type="button" onClick={() => setPendingDeleteSessionId(null)} disabled={loading}>
                      Cancel
                    </button>
                  </div>
                </div>
              )}

              {exportMessage && <p className="hint">{exportMessage}</p>}
            </div>
          )}
        </section>
      </section>

      <section className="panel">
        <h2>Results (% of detected speech time)</h2>
        {!analysis && <p className="empty-state">Run analysis to see per-speaker percentages.</p>}

        {analysis && (
          <div className="results">
            <p>
              <strong>Total Speech:</strong> {formatSeconds(analysis.totalSpeechSec)}
            </p>
            <SpeakerBars speakers={analysis.speakers} />

            <table>
              <thead>
                <tr>
                  <th>Speaker</th>
                  <th>Total Sec</th>
                  <th>Percent</th>
                  <th>Segments</th>
                </tr>
              </thead>
              <tbody>
                {analysis.speakers.map((speaker) => (
                  <tr key={speaker.speakerId}>
                    <td>{speaker.speakerId}</td>
                    <td>{speaker.totalSec.toFixed(2)}</td>
                    <td>{speaker.percentage.toFixed(1)}%</td>
                    <td>{speaker.segmentCount}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="panel">
        <h2>Transcript (English)</h2>
        {!transcript && (
          <p className="empty-state">
            Run "Transcribe to English" to generate transcript. Install dependencies with
            `pip install -r python/requirements.txt` first.
          </p>
        )}

        {transcript && (
          <div className="results">
            <p>
              <strong>Model:</strong> {transcript.modelVersion} | <strong>Processing:</strong>{" "}
              {formatSeconds(Math.max(0, transcript.processingMs / 1000))}
            </p>
            <table>
              <thead>
                <tr>
                  <th>Start</th>
                  <th>End</th>
                  <th>Speaker</th>
                  <th>Source Lang</th>
                  <th>English Text</th>
                </tr>
              </thead>
              <tbody>
                {transcript.segments.map((segment, idx) => (
                  <tr key={`${segment.startSec}-${segment.endSec}-${idx}`}>
                    <td>{segment.startSec.toFixed(2)}</td>
                    <td>{segment.endSec.toFixed(2)}</td>
                    <td>{segment.speakerId || "Unknown"}</td>
                    <td>{segment.sourceLanguage}</td>
                    <td>{segment.textEn}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </main>
  );
}
