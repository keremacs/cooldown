import { useEffect, useState } from "react";
import type { DashboardState, EmailSettings } from "../types";
import { useTheme } from "../context/ThemeContext";

interface SettingsTabProps {
  state: DashboardState;
  onFocusMode: (active: boolean, duration?: number) => void;
  onStartPomodoro: () => void;
  onRetention: (days: number) => void;
  onAutostart: (enabled: boolean) => void;
  onSaveEmail: (settings: EmailSettings, smtpPassword?: string) => Promise<void>;
  onSendWeeklyReport: () => Promise<string>;
  onSaveWeeklyReportFile: () => Promise<string>;
}

const PHASE_LABELS: Record<string, string> = {
  idle: "Idle",
  work: "Work",
  break: "Break",
  long_break: "Long break",
};

const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

export function SettingsTab({
  state,
  onFocusMode,
  onStartPomodoro,
  onRetention,
  onAutostart,
  onSaveEmail,
  onSendWeeklyReport,
  onSaveWeeklyReportFile,
}: SettingsTabProps) {
  const { theme, toggleTheme } = useTheme();
  const [emailForm, setEmailForm] = useState<EmailSettings>(state.email_settings);
  const [smtpPassword, setSmtpPassword] = useState("");
  const [emailStatus, setEmailStatus] = useState<string | null>(null);

  useEffect(() => {
    setEmailForm(state.email_settings);
  }, [state.email_settings]);

  const focus = state.focus_mode;
  const isPomodoro = focus.pomodoro && focus.active;

  async function handleSaveEmail() {
    try {
      await onSaveEmail(emailForm, smtpPassword || undefined);
      setSmtpPassword("");
      setEmailStatus("Settings saved.");
    } catch (e) {
      setEmailStatus(String(e));
    }
  }

  async function handleSaveDailySummary() {
    try {
      await onSaveEmail(emailForm);
      setEmailStatus("Daily summary settings saved.");
    } catch (e) {
      setEmailStatus(String(e));
    }
  }

  async function handleSaveReportFile() {
    try {
      const path = await onSaveWeeklyReportFile();
      setEmailStatus(`Report saved: ${path}`);
    } catch (e) {
      setEmailStatus(String(e));
    }
  }

  async function handleSendReport() {
    try {
      const msg = await onSendWeeklyReport();
      setEmailStatus(msg);
    } catch (e) {
      setEmailStatus(String(e));
    }
  }

  return (
    <div className="space-y-6">
      {/* Focus mode */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Focus Mode</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          Suppresses non-critical alerts. Only fatigue ≥ 90 will notify.
        </p>
        <div className="flex flex-wrap gap-2">
          <button
            type="button"
            onClick={() => onFocusMode(true, 25)}
            disabled={focus.active}
            className="px-3 py-1.5 text-xs rounded-lg bg-cool-600 text-white disabled:opacity-40"
          >
            Start 25 min
          </button>
          <button
            type="button"
            onClick={() => onFocusMode(true, 50)}
            disabled={focus.active}
            className="px-3 py-1.5 text-xs rounded-lg border border-[var(--border)] disabled:opacity-40"
          >
            Start 50 min
          </button>
          <button
            type="button"
            onClick={onStartPomodoro}
            disabled={focus.active}
            className="px-3 py-1.5 text-xs rounded-lg border border-indigo-500/50 text-indigo-400 disabled:opacity-40"
          >
            Start Pomodoro
          </button>
          {focus.active && (
            <button
              type="button"
              onClick={() => onFocusMode(false)}
              className="px-3 py-1.5 text-xs rounded-lg border border-red-500/50 text-red-400"
            >
              Stop
            </button>
          )}
        </div>
        {focus.active && (
          <div className="mt-3 rounded-lg bg-cool-600/10 border border-cool-600/30 px-3 py-2">
            <p className="text-xs font-medium text-cool-400">
              {isPomodoro ? "Pomodoro" : "Focus session"} active
              {focus.until ? ` · ${formatCountdown(focus.until)} remaining` : ""}
            </p>
            <p className="text-[11px] text-[var(--text-muted)] mt-1">
              {isPomodoro && (
                <>
                  Phase: {PHASE_LABELS[focus.phase ?? "work"] ?? focus.phase}
                  {(focus.cycle ?? 0) > 0 && ` · Cycle ${focus.cycle}`}
                  {" · "}
                </>
              )}
              {Math.floor(focus.session_secs / 60)}m focused in IDE · alerts suppressed below 90
            </p>
          </div>
        )}
      </section>

      {/* Daily summary */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Daily Summary</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          In-app notification at the end of your work day with screen time, commits, and fatigue.
        </p>
        <label className="flex items-center gap-3 cursor-pointer mb-3">
          <input
            type="checkbox"
            checked={emailForm.daily_summary_enabled ?? true}
            onChange={(e) =>
              setEmailForm((f) => ({ ...f, daily_summary_enabled: e.target.checked }))
            }
            className="h-4 w-4 rounded border-[var(--border)] accent-indigo-500"
          />
          <span className="text-sm">Enable daily summary notification</span>
        </label>
        <label className="block text-xs text-[var(--text-muted)]">
          Notification hour (0–23)
          <input
            type="number"
            min={0}
            max={23}
            value={emailForm.daily_summary_hour ?? 18}
            onChange={(e) =>
              setEmailForm((f) => ({
                ...f,
                daily_summary_hour: Number(e.target.value),
              }))
            }
            className="mt-1 block w-24 text-sm rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
          />
        </label>
        <button
          type="button"
          onClick={handleSaveDailySummary}
          className="px-3 py-1.5 text-xs rounded-lg bg-indigo-600 text-white"
        >
          Save daily summary settings
        </button>
      </section>
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Weekly Email Report</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          Optional SMTP report — data stays local until you send.
        </p>
        <label className="flex items-center gap-3 cursor-pointer mb-3">
          <input
            type="checkbox"
            checked={emailForm.enabled}
            onChange={(e) => setEmailForm((f) => ({ ...f, enabled: e.target.checked }))}
            className="h-4 w-4 rounded border-[var(--border)] accent-indigo-500"
          />
          <span className="text-sm">Send weekly email automatically</span>
        </label>
        <div className="grid gap-2 text-sm">
          <input
            type="email"
            placeholder="To email"
            value={emailForm.to}
            onChange={(e) => setEmailForm((f) => ({ ...f, to: e.target.value }))}
            className="rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
          />
          <input
            type="text"
            placeholder="SMTP host (e.g. smtp.gmail.com)"
            value={emailForm.smtp_host}
            onChange={(e) => setEmailForm((f) => ({ ...f, smtp_host: e.target.value }))}
            className="rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
          />
          <div className="flex gap-2">
            <input
              type="number"
              placeholder="Port"
              value={emailForm.smtp_port}
              onChange={(e) =>
                setEmailForm((f) => ({ ...f, smtp_port: Number(e.target.value) }))
              }
              className="w-24 rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
            />
            <input
              type="text"
              placeholder="SMTP user"
              value={emailForm.smtp_user}
              onChange={(e) => setEmailForm((f) => ({ ...f, smtp_user: e.target.value }))}
              className="flex-1 rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
            />
          </div>
          <input
            type="password"
            placeholder="SMTP password (leave blank to keep)"
            value={smtpPassword}
            onChange={(e) => setSmtpPassword(e.target.value)}
            className="rounded-lg border border-[var(--border)] bg-[var(--surface-2)] px-3 py-1.5"
          />
          <div className="flex gap-2 items-center">
            <label className="text-xs text-[var(--text-muted)]">
              Day
              <select
                value={emailForm.weekly_day}
                onChange={(e) =>
                  setEmailForm((f) => ({ ...f, weekly_day: Number(e.target.value) }))
                }
                className="ml-1 rounded border border-[var(--border)] bg-[var(--surface-2)] px-2 py-1"
              >
                {WEEKDAYS.map((d, i) => (
                  <option key={d} value={i}>
                    {d}
                  </option>
                ))}
              </select>
            </label>
            <label className="text-xs text-[var(--text-muted)]">
              Hour
              <input
                type="number"
                min={0}
                max={23}
                value={emailForm.weekly_hour}
                onChange={(e) =>
                  setEmailForm((f) => ({ ...f, weekly_hour: Number(e.target.value) }))
                }
                className="ml-1 w-16 rounded border border-[var(--border)] bg-[var(--surface-2)] px-2 py-1"
              />
            </label>
          </div>
        </div>
        <div className="flex gap-2 mt-3">
          <button
            type="button"
            onClick={handleSaveEmail}
            className="px-3 py-1.5 text-xs rounded-lg bg-indigo-600 text-white"
          >
            Save email settings
          </button>
          <button
            type="button"
            onClick={handleSendReport}
            className="px-3 py-1.5 text-xs rounded-lg border border-[var(--border)]"
          >
            Send report now
          </button>
          <button
            type="button"
            onClick={handleSaveReportFile}
            className="px-3 py-1.5 text-xs rounded-lg border border-[var(--border)]"
          >
            Save HTML report
          </button>
        </div>
        {emailStatus && (
          <p className="text-xs text-[var(--text-muted)] mt-2">{emailStatus}</p>
        )}
      </section>

      {/* Git hook */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Git Commit Hook</h3>
        <p className="text-xs text-[var(--text-muted)]">
          Copy <code className="text-[var(--text)]">integrations/git/post-commit</code> to{" "}
          <code className="text-[var(--text)]">.git/hooks/post-commit</code> in each repo.
          Commits appear in daily summary ({state.git_commits_today} today).
        </p>
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

      {/* Autostart */}
      <section className="rounded-xl border border-[var(--border)] bg-[var(--surface)] p-4">
        <h3 className="text-sm font-medium mb-1">Start at Login</h3>
        <p className="text-xs text-[var(--text-muted)] mb-3">
          Launch Cooldown in the background when you sign in. Only the tray icon appears until you open the app.
        </p>
        <label className="flex items-center gap-3 cursor-pointer">
          <input
            type="checkbox"
            checked={state.autostart_enabled}
            onChange={(e) => onAutostart(e.target.checked)}
            className="h-4 w-4 rounded border-[var(--border)] accent-indigo-500"
          />
          <span className="text-sm">Start automatically at login</span>
        </label>
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

function formatCountdown(untilUnix: number): string {
  const secs = Math.max(0, untilUnix - Math.floor(Date.now() / 1000));
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, "0")}`;
}
