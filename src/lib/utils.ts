import type { CognitiveZone } from "../types";

export function zoneColor(zone: CognitiveZone): string {
  switch (zone) {
    case "flow":
      return "#22c55e";
    case "distraction":
      return "#f59e0b";
    case "burnout":
      return "#ef4444";
  }
}

export function zoneLabel(zone: CognitiveZone): string {
  switch (zone) {
    case "flow":
      return "Flow";
    case "distraction":
      return "Distraction";
    case "burnout":
      return "Burnout";
  }
}

export function fatigueColor(score: number): string {
  if (score >= 75) return "#ef4444";
  if (score >= 40) return "#f59e0b";
  return "#22c55e";
}

export function formatDuration(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export function formatPercent(value: number, total: number): string {
  if (total === 0) return "0%";
  return `${Math.round((value / total) * 100)}%`;
}
