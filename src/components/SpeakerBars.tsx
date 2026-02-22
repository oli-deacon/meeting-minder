import { formatPercent, formatSeconds } from "../lib/format";
import type { SpeakerStats } from "../types";

interface SpeakerBarsProps {
  speakers: SpeakerStats[];
}

export function SpeakerBars({ speakers }: SpeakerBarsProps) {
  return (
    <div className="speaker-bars">
      {speakers.map((speaker) => (
        <div key={speaker.speakerId} className="speaker-row">
          <div className="speaker-label">{speaker.speakerId}</div>
          <div className="speaker-bar-track">
            <div className="speaker-bar-fill" style={{ width: `${speaker.percentage}%` }} />
          </div>
          <div className="speaker-value">
            {formatPercent(speaker.percentage)} ({formatSeconds(speaker.totalSec)})
          </div>
        </div>
      ))}
    </div>
  );
}
