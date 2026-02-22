import { useMemo, useState } from "react";
import { exportSession } from "./lib/tauriApi";
import { formatSeconds, isoToDisplay } from "./lib/format";
import { useSessions } from "./hooks/useSessions";
import { SpeakerBars } from "./components/SpeakerBars";
import { SessionsList } from "./components/SessionsList";

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
    onStopRecording,
    onAnalyze,
    onDelete
  } = useSessions();
  const [exportMessage, setExportMessage] = useState<string | null>(null);

  const selectedSession = details?.session;
  const analysis = details?.analysis;

  const canAnalyze = useMemo(() => {
    if (!selectedSession) {
      return false;
    }
    return selectedSession.status !== "recording" && selectedSession.status !== "processing";
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
      setExportMessage(`Exported CSV: ${paths.csvPath} | JSON: ${paths.jsonPath}`);
    } catch (err) {
      setExportMessage(err instanceof Error ? err.message : "Export failed");
    }
  };

  const onDeleteSession = async () => {
    if (!selectedSession || !canDelete) {
      return;
    }

    const confirmed = window.confirm("Sure you want to delete this recording? This cannot be undone.");
    if (!confirmed) {
      return;
    }

    await onDelete(selectedSession.id);
    setExportMessage(null);
  };

  return (
    <main className="layout">
      <header className="header">
        <div>
          <h1>Meeting Minder</h1>
          <p>Local meeting recorder with per-speaker speaking-time percentages.</p>
        </div>
        <div className="actions">
          <button type="button" onClick={onStartRecording} disabled={loading || !!recordingSession}>
            Start Recording
          </button>
          <button type="button" onClick={onStopRecording} disabled={loading || !recordingSession}>
            Stop Recording
          </button>
        </div>
      </header>

      {recordingSession && (
        <section className="recording-banner">Recording in progress: session {recordingSession.id}</section>
      )}

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
                <button type="button" onClick={onExport} disabled={!analysis || loading}>
                  Export CSV/JSON
                </button>
                <button type="button" className="danger-button" onClick={() => void onDeleteSession()} disabled={!canDelete || loading}>
                  Delete Recording
                </button>
              </div>

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
    </main>
  );
}
