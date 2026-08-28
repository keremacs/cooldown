import type { DailySummary } from "../types";
import { formatDuration } from "../lib/utils";

interface DailySummaryToastProps {
  summary: DailySummary;
  onDismiss: () => void;
}

export function DailySummaryToast({ summary, onDismiss }: DailySummaryToastProps) {
  const st = summary.screen_time;
  const total = st.ide_secs + st.browser_secs + st.communication_secs + st.other_secs;

  return (
    <div className="fixed bottom-4 right-4 z-50 max-w-sm rounded-xl border border-indigo-500/40 bg-[var(--surface)] shadow-xl p-4 animate-in slide-in-from-bottom-2">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-xs font-semibold uppercase tracking-wider text-indigo-400">
            Daily Summary
          </p>
          <p className="text-sm text-[var(--text)] mt-1">{summary.message}</p>
        </div>
        <button
          type="button"
          onClick={onDismiss}
          className="text-[var(--text-muted)] hover:text-[var(--text)] text-xs shrink-0"
          aria-label="Dismiss"
        >
          ✕
        </button>
      </div>
      <div className="mt-3 grid grid-cols-2 gap-2 text-[11px] text-[var(--text-muted)]">
        <span>Screen time: {formatDuration(total)}</span>
        <span>Peak fatigue: {summary.peak_fatigue.toFixed(0)}</span>
        <span>Git commits: {summary.git_commits}</span>
        <span>Errors: {summary.total_errors}</span>
      </div>
      {summary.top_apps.length > 0 && (
        <div className="mt-2 pt-2 border-t border-[var(--border)]/50">
          <p className="text-[10px] uppercase text-[var(--text-muted)] mb-1">Top apps</p>
          {summary.top_apps.slice(0, 3).map((app) => (
            <div key={app.app_name} className="flex justify-between text-xs py-0.5">
              <span className="truncate">{app.app_name}</span>
              <span className="tabular-nums shrink-0 ml-2">{formatDuration(app.secs)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
