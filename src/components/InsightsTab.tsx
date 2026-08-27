import type { CooldownEventPayload, DashboardState } from "../types";
import { fatigueColor } from "../lib/utils";

const ERROR_LABELS: Record<string, string> = {
  build: "Build",
  lint: "Lint",
  test: "Test",
  runtime: "Runtime",
  terminal: "Terminal",
  task: "Task",
  unknown: "Other",
};

export function InsightsTab({
  state,
  events,
}: {
  state: DashboardState;
  events: CooldownEventPayload[];
}) {
  return (
    <div className="space-y-5">
      {/* Live hook events */}
      {events.length > 0 && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3">
            Recent Hook Events
          </h3>
          <ul className="space-y-2 max-h-36 overflow-y-auto">
            {events.slice(0, 8).map((e, i) => (
              <li key={`${e.ts}-${i}`} className="text-xs flex gap-2">
                <span className="text-[var(--text-muted)] shrink-0">
                  {new Date(e.ts * 1000).toLocaleTimeString()}
                </span>
                <span className="text-cool-400 shrink-0">{e.event.source}</span>
                <span className="text-[var(--text)] truncate">
                  {e.event.event ?? (e.event.exit_code != null ? `exit ${e.event.exit_code}` : "event")}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Deep work score */}
      <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4 flex items-center gap-4">
        <div
          className="text-3xl font-bold tabular-nums"
          style={{ color: fatigueColor(100 - state.deep_work_score) }}
        >
          {Math.round(state.deep_work_score)}
        </div>
        <div>
          <h3 className="text-sm font-medium">Deep Work Score</h3>
          <p className="text-xs text-[var(--text-muted)] mt-0.5">
            Based on longest uninterrupted IDE focus block today.
          </p>
        </div>
      </div>

      {/* Error breakdown */}
      {state.error_breakdown.length > 0 && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3">
            Error Types (24h)
          </h3>
          <div className="space-y-2">
            {state.error_breakdown.map((e) => (
              <div key={e.category} className="flex items-center gap-2 text-sm">
                <span className="w-24 text-[var(--text-muted)]">
                  {ERROR_LABELS[e.category] ?? e.category}
                </span>
                <div className="flex-1 h-2 rounded-full bg-[var(--surface-2)] overflow-hidden">
                  <div
                    className="h-full bg-red-500/70 rounded-full"
                    style={{
                      width: `${Math.min(100, e.count * 10)}%`,
                    }}
                  />
                </div>
                <span className="tabular-nums text-[var(--text-muted)] w-6 text-right">{e.count}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Anomalies */}
      {state.anomalies.length > 0 && (
        <div className="rounded-xl border border-amber-500/30 bg-amber-500/5 p-4 space-y-2">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-amber-400">
            Anomaly Detection
          </h3>
          {state.anomalies.map((a) => (
            <p key={a.metric} className="text-sm text-[var(--text)]">{a.message}</p>
          ))}
        </div>
      )}

      {/* Proactive suggestions */}
      {state.proactive_suggestions.length > 0 && (
        <div className="rounded-xl border border-cool-500/30 bg-cool-500/5 p-4 space-y-2">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-cool-400">
            Proactive Suggestions
          </h3>
          {state.proactive_suggestions.map((s) => (
            <p key={s.hour} className="text-sm text-[var(--text)]">{s.message}</p>
          ))}
        </div>
      )}

      {/* Baseline */}
      {state.baseline && (
        <div className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
          <h3 className="text-xs font-semibold uppercase tracking-wider text-[var(--text-muted)] mb-3">
            Personal Baseline (14-day avg)
          </h3>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <Stat label="Fatigue" value={state.baseline.avg_fatigue.toFixed(1)} />
            <Stat label="Switches" value={state.baseline.avg_switches.toFixed(1)} />
            <Stat label="Errors" value={state.baseline.avg_errors.toFixed(1)} />
            <Stat label="Deep Work" value={state.baseline.avg_deep_work.toFixed(1)} />
          </div>
        </div>
      )}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg bg-[var(--surface-2)] px-3 py-2">
      <p className="text-[10px] uppercase text-[var(--text-muted)]">{label}</p>
      <p className="font-semibold tabular-nums">{value}</p>
    </div>
  );
}
