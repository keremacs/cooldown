import { useCallback } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { DashboardState } from "../types";
import { fatigueColor, zoneLabel } from "../lib/utils";

export function MiniWidget({ state }: { state: DashboardState }) {
  const win = getCurrentWindow();

  const onDrag = useCallback(async (e: React.PointerEvent) => {
    if (e.button !== 0) return;
    await win.startDragging();
  }, [win]);

  const onMinimize = useCallback(async () => {
    await win.hide();
  }, [win]);

  const onClose = useCallback(async () => {
    await win.hide();
  }, [win]);

  return (
    <div className="h-screen w-full flex flex-col overflow-hidden rounded-lg border border-[var(--border)] bg-[var(--surface-2)] shadow-xl">
      {/* Title bar — drag handle + window controls */}
      <div className="flex items-center h-7 shrink-0 bg-[var(--surface)] border-b border-[var(--border)]">
        <div
          className="flex-1 flex items-center gap-1.5 px-2 cursor-grab active:cursor-grabbing select-none"
          onPointerDown={onDrag}
        >
          <img
            src="/app-icon.png"
            alt=""
            className="w-3.5 h-3.5 pointer-events-none"
            draggable={false}
          />
          <span className="text-[10px] font-medium text-[var(--text-muted)] pointer-events-none">
            Cooldown
          </span>
        </div>
        <div className="flex items-center shrink-0">
          <WidgetButton label="Minimize" onClick={onMinimize}>
            −
          </WidgetButton>
          <WidgetButton label="Close" onClick={onClose} danger>
            ×
          </WidgetButton>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 flex items-center justify-center px-3">
        <div
          className="text-2xl font-bold tabular-nums"
          style={{ color: fatigueColor(state.fatigue_score) }}
        >
          {Math.round(state.fatigue_score)}
        </div>
        <div className="ml-3">
          <p className="text-xs font-medium text-[var(--text)]">{zoneLabel(state.zone)}</p>
          <p className="text-[10px] text-[var(--text-muted)]">
            Deep: {Math.round(state.deep_work_score)}
          </p>
        </div>
      </div>
    </div>
  );
}

function WidgetButton({
  children,
  label,
  onClick,
  danger,
}: {
  children: React.ReactNode;
  label: string;
  onClick: () => void;
  danger?: boolean;
}) {
  return (
    <button
      type="button"
      aria-label={label}
      onClick={onClick}
      className={`w-7 h-7 flex items-center justify-center text-sm leading-none transition-colors ${
        danger
          ? "hover:bg-red-500/80 hover:text-white text-[var(--text-muted)]"
          : "hover:bg-[var(--border)] text-[var(--text-muted)]"
      }`}
    >
      {children}
    </button>
  );
}
