import { useState } from "react";
import type { BreakNotification } from "../types";
import { fatigueColor } from "../lib/utils";

const LEVEL_STYLES = {
  hint: { border: "border-amber-500/40", title: "Heads up" },
  warning: { border: "border-orange-500/50", title: "Time for a break" },
  critical: { border: "border-red-500/60", title: "Mandatory break" },
};

interface Props {
  notification: BreakNotification;
  onDismiss: () => void;
  onSnooze: (minutes: number) => void;
  onSaveJournal: (text: string) => void;
}

export function FatigueNotification({ notification, onDismiss, onSnooze, onSaveJournal }: Props) {
  const [journal, setJournal] = useState("");
  const style = LEVEL_STYLES[notification.level];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (journal.trim()) onSaveJournal(journal.trim());
    else onDismiss();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center p-4 pointer-events-none">
      <div
        className={`pointer-events-auto w-full max-w-md rounded-2xl border ${style.border} bg-[var(--surface-2)]/95 backdrop-blur-md shadow-2xl animate-slide-up`}
        role="alertdialog"
      >
        <div className="p-5">
          <div className="flex items-start gap-3">
            <div
              className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-lg font-bold"
              style={{
                backgroundColor: `${fatigueColor(notification.fatigue_score)}22`,
                color: fatigueColor(notification.fatigue_score),
              }}
            >
              {Math.round(notification.fatigue_score)}
            </div>
            <div>
              <h2 className="text-sm font-semibold">{style.title}</h2>
              <p className="text-xs text-[var(--text-muted)] mt-0.5">{notification.insight}</p>
              <p className="text-xs text-cool-400 mt-1 font-medium">
                {notification.break_suggestion.title} ({notification.break_suggestion.duration_min}m)
              </p>
              <p className="text-xs text-[var(--text-muted)]">{notification.break_suggestion.detail}</p>
            </div>
          </div>

          <form onSubmit={handleSubmit} className="mt-4">
            <input
              type="text"
              value={journal}
              onChange={(e) => setJournal(e.target.value)}
              placeholder="What were you working on before taking a break?"
              className="w-full rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 py-2 text-sm placeholder:text-[var(--text-muted)] focus:outline-none focus:ring-1 focus:ring-cool-500"
            />
            <div className="flex gap-2 mt-3">
              <button type="submit" className="flex-1 rounded-lg bg-cool-600 px-3 py-2 text-xs font-medium text-white">
                {notification.level === "critical" ? "Start Break Now" : "Take Break"}
              </button>
              {notification.level !== "critical" && (
                <button type="button" onClick={() => onSnooze(15)} className="rounded-lg border border-[var(--border)] px-3 py-2 text-xs">
                  Snooze 15m
                </button>
              )}
              <button type="button" onClick={onDismiss} className="rounded-lg px-3 py-2 text-xs text-[var(--text-muted)]">
                Dismiss
              </button>
            </div>
          </form>
        </div>
      </div>
    </div>
  );
}

export function AlertHintToast({ message }: { message: string }) {
  return (
    <div
      className="fixed top-4 right-4 z-40 max-w-xs rounded-xl border backdrop-blur px-4 py-3 animate-slide-up"
      style={{
        background: "var(--hint-bg)",
        borderColor: "var(--hint-border)",
      }}
    >
      <p className="text-xs" style={{ color: "var(--hint-text)" }}>{message}</p>
    </div>
  );
}
