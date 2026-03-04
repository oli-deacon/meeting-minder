interface RecordingPromptPanelProps {
  visible: boolean;
  sessionId: string;
  prompts: string[];
  isCollapsed: boolean;
  onToggleCollapse: () => void;
}

export function RecordingPromptPanel({
  visible,
  sessionId,
  prompts,
  isCollapsed,
  onToggleCollapse
}: RecordingPromptPanelProps) {
  if (!visible) {
    return null;
  }

  const listId = `prompt-list-${sessionId}`;

  return (
    <section className="prompt-panel" aria-live="polite">
      <div className="prompt-panel-header">
        <h2 className="prompt-panel-title">Prompt Starters</h2>
        <button
          type="button"
          className="prompt-panel-toggle"
          aria-expanded={!isCollapsed}
          aria-controls={listId}
          onClick={onToggleCollapse}
        >
          {isCollapsed ? "Show" : "Hide"}
        </button>
      </div>
      {!isCollapsed && (
        <ol id={listId} className="prompt-list">
          {prompts.map((prompt) => (
            <li key={prompt}>{prompt}</li>
          ))}
        </ol>
      )}
    </section>
  );
}
