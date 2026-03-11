import { isoToDisplay } from "../lib/format";
import type { Session } from "../types";

interface SessionsListProps {
  sessions: Session[];
  selectedId: string | null;
  onSelect: (id: string) => void;
}

export function SessionsList({ sessions, selectedId, onSelect }: SessionsListProps) {
  if (sessions.length === 0) {
    return <p className="empty-state">No sessions yet. Start recording or import WAV to create one.</p>;
  }

  return (
    <ul className="sessions-list">
      {sessions.map((session) => {
        const isSelected = session.id === selectedId;
        return (
          <li key={session.id}>
            <button
              className={`session-item ${isSelected ? "active" : ""}`}
              onClick={() => onSelect(session.id)}
              type="button"
            >
              <span>{isoToDisplay(session.startedAt)}</span>
              <span className="session-badges">
                <span className={`badge badge-${session.status}`}>{session.status}</span>
                <span className={`badge badge-transcription-${session.transcriptionStatus}`}>
                  tx:{session.transcriptionStatus}
                </span>
              </span>
            </button>
          </li>
        );
      })}
    </ul>
  );
}
