export function formatSeconds(totalSec: number): string {
  const seconds = Math.max(0, Math.floor(totalSec));
  const mm = Math.floor(seconds / 60)
    .toString()
    .padStart(2, "0");
  const ss = (seconds % 60).toString().padStart(2, "0");
  return `${mm}:${ss}`;
}

export function formatPercent(value: number): string {
  return `${value.toFixed(1)}%`;
}

export function isoToDisplay(iso: string): string {
  return new Date(iso).toLocaleString();
}
