import type { DashboardState } from "../types";
import { useTheme } from "../context/ThemeContext";

interface SettingsTabProps {
  state: DashboardState;
  onFocusMode: (active: boolean, duration?: number) => void;
  onRetention: (days: number) => void;
}

export function SettingsTab({ state, onFocusMode, onRetention }: SettingsTabProps) {
  const { theme, toggleTheme } = useTheme();

  return (
    <div className="space-y-6">
      {/* Focus mode */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Focus Mode</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          Suppresses non-critical alerts. Only fatigue ≥ 90 will notify.
        </p>
        <div className="flex gap-2">
          <button
            type="button"
            onClick={() => onFocusMode(true, 25)}
            disabled={state.focus_mode.active}
            className="px-3 py-1.5 text-xs rounded-lg bg-cool-600 text-white disabled:opacity-40"
          >
            Start 25 min
          </button>
          <button
            type="button"
            onClick={() => onFocusMode(true, 50)}
            disabled={state.focus_mode.active}
            className="px-3 py-1.5 text-xs rounded-lg border border-[var(--border)] disabled:opacity-40"
          >
            Start 50 min
          </button>
          {state.focus_mode.active && (
            <button
              type="button"
              onClick={() => onFocusMode(false)}
              className="px-3 py-1.5 text-xs rounded-lg border border-red-500/50 text-red-400"
            >
              Stop
            </button>
          )}
        </div>
        {state.focus_mode.active && (
          <p className="text-xs text-cool-400 mt-2">
            Focus session: {Math.floor(state.focus_mode.session_secs / 60)}m in IDE
          </p>
        )}
      </section>

      {/* Theme */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-3">Theme</h3>
        <button
          type="button"
          onClick={toggleTheme}
          className="px-3 py-1.5 text-xs rounded-lg border border-[var(--border)]"
        >
          Switch to {theme === "dark" ? "Light" : "Dark"} Mode
        </button>
      </section>

      {/* Retention */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Data Retention</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          Automatically delete metrics older than this many days.
        </p>
        <select
          value={state.retention_days}
          onChange={(e) => onRetention(Number(e.target.value))}
          className="text-sm rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
        >
          <option value={30}>30 days</option>
          <option value={90}>90 days</option>
          <option value={180}>180 days</option>
          <option value={365}>1 year</option>
        </select>
      </section>

      {/* Benchmark */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-3">Benchmark (self)</h3>
        <div className="grid grid-cols-2 gap-2 text-sm">
          <Metric label="Memory" value={`${state.benchmark.memory_mb.toFixed(1)} MB`} />
          <Metric label="CPU" value={`${state.benchmark.cpu_percent.toFixed(1)}%`} />
          <Metric label="Uptime" value={`${Math.floor(state.benchmark.uptime_secs / 60)}m`} />
          <Metric label="Threads" value={String(state.benchmark.threads)} />
        </div>
      </section>

      {/* Plugins */}
      {state.plugins.length > 0 && (
        <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
          <h3 className="text-sm font-medium mb-3">Event Plugins</h3>
          <div className="scroll-area max-h-40 space-y-1 pr-1">
            {state.plugins.map((p) => (
              <div
                key={p.id}
                className="flex justify-between gap-3 text-xs text-[var(--text-muted)] py-1 border-b border-[var(--border)]/50 last:border-0"
              >
                <span className="truncate">{p.name}</span>
                <span className="shrink-0 tabular-nums">{p.events_received} events</span>
              </div>
            ))}
          </div>
        </section>
      )}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg bg-[var(--surface-2)] px-3 py-2">
      <p className="text-[10px] uppercase text-[var(--text-muted)]">{label}</p>
      <p className="font-semibold tabular-nums">{value}</p>
    </div>
  );
}
