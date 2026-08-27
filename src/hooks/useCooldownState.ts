import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AlertToast,
  BreakNotification,
  CooldownEventPayload,
  DashboardState,
  JournalEntry,
} from "../types";

const DEFAULT: DashboardState = {
  fatigue_score: 0,
  zone: "flow",
  insight: "Collecting activity data…",
  cognitive_history: [],
  screen_time: { ide_secs: 0, browser_secs: 0, communication_secs: 0, other_secs: 0 },
  switches_last_30min: 0,
  errors_last_hour: 0,
  keystrokes_per_min: 0,
  active_window: "",
  notification_pending: false,
  snoozed_until: null,
  deep_work_score: 0,
  alert_level: null,
  break_suggestion: null,
  weekly_trend: [],
  monthly_trend: [],
  error_breakdown: [],
  baseline: null,
  anomalies: [],
  proactive_suggestions: [],
  focus_mode: { active: false, until: null, session_secs: 0 },
  benchmark: { memory_mb: 0, cpu_percent: 0, threads: 0, uptime_secs: 0 },
  plugins: [],
  theme: "dark",
  retention_days: 90,
};

export function useCooldownState() {
  const [state, setState] = useState<DashboardState>(DEFAULT);
  const [events, setEvents] = useState<CooldownEventPayload[]>([]);
  const [notification, setNotification] = useState<BreakNotification | null>(null);
  const [hint, setHint] = useState<AlertToast | null>(null);
  const [journal, setJournal] = useState<JournalEntry[]>([]);

  const refresh = useCallback(async () => {
    try {
      setState(await invoke<DashboardState>("get_dashboard"));
    } catch (e) {
      console.error(e);
    }
  }, []);

  const loadJournal = useCallback(async () => {
    try {
      setJournal(await invoke<JournalEntry[]>("get_journal", { limit: 50 }));
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => {
    refresh();
    loadJournal();

    const subs = [
      listen<DashboardState>("fatigue-update", (e) => setState(e.payload)),
      listen<CooldownEventPayload>("cooldown-event", (e) => {
        const payload = e.payload;
        setEvents((prev) => [payload, ...prev].slice(0, 50));
        setState(payload.dashboard);
      }),
      listen<BreakNotification>("break-notification", (e) => setNotification(e.payload)),
      listen<AlertToast>("alert-hint", (e) => {
        setHint(e.payload);
        setTimeout(() => setHint(null), 8000);
      }),
    ];

    return () => {
      subs.forEach((p) => p.then((fn) => fn()));
    };
  }, [refresh, loadJournal]);

  const dismissNotification = useCallback(async () => {
    await invoke("dismiss_notification");
    setNotification(null);
  }, []);

  const snoozeNotification = useCallback(async (minutes: number) => {
    await invoke("snooze_notification", { minutes });
    setNotification(null);
  }, []);

  const saveJournal = useCallback(
    async (text: string) => {
      await invoke("save_journal", { text });
      setNotification(null);
      loadJournal();
    },
    [loadJournal],
  );

  const setFocusMode = useCallback(
    async (active: boolean, durationMin = 25) => {
      await invoke("set_focus_mode", { active, duration_min: durationMin });
      refresh();
    },
    [refresh],
  );

  const setRetentionDays = useCallback(
    async (days: number) => {
      await invoke("set_retention_days", { days });
      refresh();
    },
    [refresh],
  );

  return {
    state,
    events,
    notification,
    hint,
    journal,
    dismissNotification,
    snoozeNotification,
    saveJournal,
    setFocusMode,
    setRetentionDays,
    refresh,
    loadJournal,
  };
}
