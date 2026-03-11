import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionsList } from "../src/components/SessionsList";
import type { Session } from "../src/types";

describe("SessionsList", () => {
  it("renders transcription status badge", () => {
    const sessions: Session[] = [
      {
        id: "s1",
        startedAt: "2026-03-11T09:00:00Z",
        endedAt: "2026-03-11T09:30:00Z",
        audioPath: "/tmp/audio.wav",
        status: "recorded",
        transcriptionStatus: "completed"
      }
    ];

    render(<SessionsList sessions={sessions} selectedId={null} onSelect={vi.fn()} />);

    expect(screen.getByText("recorded")).toBeInTheDocument();
    expect(screen.getByText("tx:completed")).toBeInTheDocument();
  });
});
