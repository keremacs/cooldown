export type CognitiveZone = "flow" | "distraction" | "burnout";
export type AlertLevel = "hint" | "warning" | "critical";
export type ErrorCategory =
  | "build"
  | "lint"
  | "test"
  | "runtime"
  | "terminal"
  | "task"
  | "unknown";

export interface CognitiveBucket {
  hour: string;
  fatigue: number;
  zone: CognitiveZone;
  switch_count: number;
  error_count: number;
}

export interface ScreenTimeTotals {
  ide_secs: number;
  browser_secs: number;
  communication_secs: number;
  other_secs: number;
}

export interface BaselineMetrics {
  avg_fatigue: number;
  avg_switches: number;
  avg_errors: number;
  avg_keystrokes_per_min: number;
  avg_deep_work: number;
}

export interface TrendBucket {
  label: string;
  avg_fatigue: number;
  avg_switches: number;
  avg_errors: number;
  avg_deep_work: number;
}

export interface ErrorBreakdown {
  category: ErrorCategory;
  count: number;
}

export interface AnomalyReport {
  metric: string;
  current: number;
  baseline: number;
  deviation_pct: number;
  message: string;
}

export interface ProactiveSuggestion {
  hour: string;
  message: string;
}

export interface BreakSuggestion {
  title: string;
  detail: string;
  duration_min: number;
}

export type PomodoroPhase = "idle" | "work" | "break" | "long_break";

export interface FocusModeState {
  active: boolean;
  until: number | null;
  session_secs: number;
  pomodoro?: boolean;
  phase?: PomodoroPhase;
  cycle?: number;
}

export interface AppUsageEntry {
  app_name: string;
  category: string;
  secs: number;
}

export interface DailySummary {
  date: string;
  screen_time: ScreenTimeTotals;
  top_apps: AppUsageEntry[];
  git_commits: number;
  peak_fatigue: number;
  total_errors: number;
  journal_entries: number;
  context_switches: number;
  message: string;
}

export interface PomodoroNotification {
  message: string;
  phase: PomodoroPhase;
  cycle: number;
}

export interface EmailSettings {
  enabled: boolean;
  to: string;
  smtp_host: string;
  smtp_port: number;
  smtp_user: string;
  weekly_day: number;
  weekly_hour: number;
  daily_summary_hour?: number;
  daily_summary_enabled?: boolean;
}

export interface BenchmarkMetrics {
  memory_mb: number;
  cpu_percent: number;
  threads: number;
  uptime_secs: number;
}

export interface JournalEntry {
  id: number;
  ts: number;
  text: string;
}

export interface PluginInfo {
  id: string;
  name: string;
  events_received: number;
}

export interface DashboardState {
  fatigue_score: number;
  zone: CognitiveZone;
  insight: string;
  cognitive_history: CognitiveBucket[];
  screen_time: ScreenTimeTotals;
  switches_last_30min: number;
  errors_last_hour: number;
  keystrokes_per_min: number;
  active_window: string;
  active_app: string;
  active_category: string;
  notification_pending: boolean;
  snoozed_until: number | null;
  deep_work_score: number;
  alert_level: AlertLevel | null;
  break_suggestion: BreakSuggestion | null;
  weekly_trend: TrendBucket[];
  monthly_trend: TrendBucket[];
  error_breakdown: ErrorBreakdown[];
  baseline: BaselineMetrics | null;
  anomalies: AnomalyReport[];
  proactive_suggestions: ProactiveSuggestion[];
  focus_mode: FocusModeState;
  benchmark: BenchmarkMetrics;
  plugins: PluginInfo[];
  theme: string;
  retention_days: number;
  autostart_enabled: boolean;
  app_usage: AppUsageEntry[];
  git_commits_today: number;
  email_settings: EmailSettings;
}

export interface DevEvent {
  source: string;
  event?: string;
  exit_code?: number;
  message?: string;
  plugin?: string;
}

export interface CooldownEventPayload {
  event: DevEvent;
  category: ErrorCategory;
  ts: number;
  fatigue_score: number;
  errors_last_hour: number;
  dashboard: DashboardState;
}

export interface BreakNotification {
  fatigue_score: number;
  insight: string;
  level: AlertLevel;
  break_suggestion: BreakSuggestion;
}

export interface AlertToast {
  level: AlertLevel;
  message: string;
}
