//! Extended domain models for Cooldown v2 features.

use serde::{Deserialize, Serialize};

/// Incoming developer event from VS Code, terminal hooks, plugins, etc.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DevEvent {
    pub source: String,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub message: Option<String>,
    /// Optional plugin namespace for the plugin registry.
    #[serde(default)]
    pub plugin: Option<String>,
}

/// Classified error type for breakdown analytics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Build,
    Lint,
    Test,
    Runtime,
    Terminal,
    Task,
    Unknown,
}

impl ErrorCategory {
    pub fn from_event(event: &DevEvent) -> Self {
        match event.event.as_deref() {
            Some("build_error") => Self::Build,
            Some("lint_error") => Self::Lint,
            Some("test_failed") | Some("test_error") => Self::Test,
            Some("task_failed") => Self::Task,
            Some("runtime_error") => Self::Runtime,
            Some("git_commit") | Some("git_push") => Self::Unknown, // activity, not error
            _ if event.exit_code.is_some_and(|c| c != 0) => Self::Terminal,
            _ if event.exit_code.is_some() => Self::Unknown, // success terminal — activity
            _ => Self::Unknown,
        }
    }

    pub fn is_error_event(event: &DevEvent) -> bool {
        if event.is_activity() {
            return false;
        }
        event.exit_code.is_none_or(|c| c != 0)
            || matches!(
                event.event.as_deref(),
                Some("build_error" | "lint_error" | "test_failed" | "test_error" | "task_failed" | "runtime_error")
            )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Lint => "lint",
            Self::Test => "test",
            Self::Runtime => "runtime",
            Self::Terminal => "terminal",
            Self::Task => "task",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "build" => Self::Build,
            "lint" => Self::Lint,
            "test" => Self::Test,
            "runtime" => Self::Runtime,
            "terminal" => Self::Terminal,
            "task" => Self::Task,
            _ => Self::Unknown,
        }
    }
}

impl DevEvent {
    pub fn is_activity(&self) -> bool {
        if let Some(ev) = self.event.as_deref() {
            if ev == "git_commit" || ev == "git_push" || ev.starts_with("git_") {
                return true;
            }
        }
        (self.source == "git" && self.exit_code.is_none())
            || (self.exit_code == Some(0) && self.source == "terminal")
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CognitiveZone {
    Flow,
    Distraction,
    Burnout,
}

impl CognitiveZone {
    pub fn from_fatigue(score: f64) -> Self {
        if score >= 75.0 {
            Self::Burnout
        } else if score >= 40.0 {
            Self::Distraction
        } else {
            Self::Flow
        }
    }
}

/// Graduated alert severity.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Hint,     // fatigue >= 60
    Warning,  // fatigue >= 75
    Critical, // fatigue >= 90
}

impl AlertLevel {
    pub fn from_fatigue(score: f64) -> Option<Self> {
        if score >= 90.0 {
            Some(Self::Critical)
        } else if score >= 75.0 {
            Some(Self::Warning)
        } else if score >= 60.0 {
            Some(Self::Hint)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AppCategory {
    Ide,
    Browser,
    Communication,
    Other,
}

impl AppCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ide => "ide",
            Self::Browser => "browser",
            Self::Communication => "communication",
            Self::Other => "other",
        }
    }

    pub fn classify(title: &str, app_name: &str) -> Self {
        let stem = process_stem(app_name);
        let haystack = format!("{} {}", title, app_name).to_lowercase();

        // Cooldown itself — don't categorize as productive time.
        if stem == "cooldown" {
            return Self::Other;
        }

        // UWP / shell hosts expose the real app in the window title, not the process name.
        const UWP_HOSTS: &[&str] = &[
            "applicationframehost", "dllhost", "systemsettings", "shellexperiencehost",
            "searchhost", "startmenuexperiencehost", "textinputhost", "widgetservice",
        ];
        let use_title_only = UWP_HOSTS.contains(&stem.as_str()) || stem == "unknown";

        if !use_title_only {
            const IDE_PROCESSES: &[&str] = &[
                "code", "cursor", "devenv", "idea64", "idea", "webstorm64", "webstorm",
                "pycharm64", "pycharm", "goland64", "goland", "rider64", "rider",
                "windowsterminal", "wt", "powershell", "pwsh", "cmd", "sublime_text",
                "notepad++", "xcode", "terminal", "iterm2", "iterm", "warp", "alacritty",
                "kitty", "ghostty", "wezterm", "hyper", "tabby", "sublime text", "nova",
                "bbedit", "zed", "datagrip", "intellij", "android studio", "fleet",
                "vscodium", "neovide", "lapce", "docker desktop", "postman", "insomnia",
            ];
            const BROWSER_PROCESSES: &[&str] = &[
                "chrome", "firefox", "msedge", "msedgewebview2", "brave", "opera",
                "vivaldi", "iexplore", "safari", "arc", "orion", "waterfox", "librewolf",
                "chromium",
            ];
            const COMM_PROCESSES: &[&str] = &[
                "slack", "discord", "teams", "ms-teams", "zoom", "outlook", "olk",
                "telegram", "whatsapp", "signal", "skype", "msteams", "commsapps",
            ];

            if matches_process_stem(&stem, IDE_PROCESSES) {
                return Self::Ide;
            }
            if matches_process_stem(&stem, BROWSER_PROCESSES) {
                return Self::Browser;
            }
            if matches_process_stem(&stem, COMM_PROCESSES) {
                return Self::Communication;
            }
        }

        const IDE_TITLES: &[&str] = &[
            "visual studio code", "vscode", "visual studio", "jetbrains", "intellij",
            "pycharm", "webstorm", "goland", "rider", "datagrip", "android studio",
            "neovim", "nvim", " - vim", "emacs", "sublime", "zed", "xcode", "github copilot",
            "iterm", "warp", "alacritty", "wezterm", "windows terminal", "cursor",
            "vscodium", "neovide", "lapce", "fleet", "postman", "docker desktop",
        ];
        const BROWSER_TITLES: &[&str] = &[
            "google chrome", "mozilla firefox", "microsoft edge", "brave", "opera",
            "vivaldi", "safari", "arc browser", " - chrome", " - firefox", " - edge",
            "and google chrome", "in chrome", "in firefox", "in edge",
        ];
        const COMM_TITLES: &[&str] = &[
            "slack", "discord", "microsoft teams", "ms teams", "zoom meeting", "zoom ",
            "outlook", "telegram", "whatsapp", "signal", "skype",
        ];

        if IDE_TITLES.iter().any(|k| haystack.contains(k)) {
            Self::Ide
        } else if BROWSER_TITLES.iter().any(|k| haystack.contains(k)) {
            Self::Browser
        } else if COMM_TITLES.iter().any(|k| haystack.contains(k)) {
            Self::Communication
        } else if matches_keyword(&haystack, &["chrome", "firefox", "edge", "safari", "brave", "opera", "vivaldi"]) {
            Self::Browser
        } else if matches_keyword(&haystack, &["teams", "discord", "slack", "zoom", "outlook", "telegram"]) {
            Self::Communication
        } else {
            Self::Other
        }
    }
}

/// Basename without path or extension, lowercased — e.g. `C:\App\Cursor.exe` → `cursor`.
fn process_stem(app_name: &str) -> String {
    let base = app_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(app_name)
        .trim()
        .to_lowercase();
    base.strip_suffix(".exe")
        .or_else(|| base.strip_suffix(".app"))
        .unwrap_or(&base)
        .to_string()
}

fn matches_process_stem(stem: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        let pattern = pattern.to_lowercase();
        stem == pattern
    })
}

/// Word-aware substring match — avoids false positives like "mail" inside unrelated words.
fn matches_keyword(haystack: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| {
        let kw = kw.to_lowercase();
        haystack.contains(&kw)
    })
}

#[cfg(test)]
mod app_category_tests {
    use super::AppCategory;

    #[test]
    fn classifies_windows_cursor() {
        assert_eq!(
            AppCategory::classify("lib.rs - cooldown - Cursor", "Cursor.exe"),
            AppCategory::Ide
        );
    }

    #[test]
    fn classifies_windows_vscode() {
        assert_eq!(
            AppCategory::classify("main.ts - project - Visual Studio Code", "Code.exe"),
            AppCategory::Ide
        );
    }

    #[test]
    fn classifies_windows_chrome() {
        assert_eq!(
            AppCategory::classify("Google - Chrome", "chrome.exe"),
            AppCategory::Browser
        );
    }

    #[test]
    fn classifies_windows_terminal() {
        assert_eq!(
            AppCategory::classify("PowerShell", "WindowsTerminal.exe"),
            AppCategory::Ide
        );
    }

    #[test]
    fn classifies_uwp_by_title() {
        assert_eq!(
            AppCategory::classify("WhatsApp", "ApplicationFrameHost.exe"),
            AppCategory::Communication
        );
    }

    #[test]
    fn classifies_unknown_process_by_title() {
        assert_eq!(
            AppCategory::classify("Inbox - Outlook", "unknown"),
            AppCategory::Communication
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveBucket {
    pub hour: String,
    pub fatigue: f64,
    pub zone: CognitiveZone,
    pub switch_count: u32,
    pub error_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScreenTimeTotals {
    pub ide_secs: u64,
    pub browser_secs: u64,
    pub communication_secs: u64,
    pub other_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BaselineMetrics {
    pub avg_fatigue: f64,
    pub avg_switches: f64,
    pub avg_errors: f64,
    pub avg_keystrokes_per_min: f64,
    pub avg_deep_work: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendBucket {
    pub label: String,
    pub avg_fatigue: f64,
    pub avg_switches: f64,
    pub avg_errors: f64,
    pub avg_deep_work: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendPeriod {
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBreakdown {
    pub category: ErrorCategory,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyReport {
    pub metric: String,
    pub current: f64,
    pub baseline: f64,
    pub deviation_pct: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveSuggestion {
    pub hour: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakSuggestion {
    pub title: String,
    pub detail: String,
    pub duration_min: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroPhase {
    #[default]
    Idle,
    Work,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusModeState {
    pub active: bool,
    pub until: Option<i64>,
    pub session_secs: u64,
    #[serde(default)]
    pub pomodoro: bool,
    #[serde(default)]
    pub phase: PomodoroPhase,
    #[serde(default)]
    pub cycle: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageEntry {
    pub app_name: String,
    pub category: String,
    pub secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub screen_time: ScreenTimeTotals,
    pub top_apps: Vec<AppUsageEntry>,
    pub git_commits: u32,
    pub peak_fatigue: f64,
    pub total_errors: u32,
    pub journal_entries: u32,
    pub context_switches: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroNotification {
    pub message: String,
    pub phase: PomodoroPhase,
    pub cycle: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub enabled: bool,
    pub to: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub weekly_day: u8,
    pub weekly_hour: u8,
    #[serde(default)]
    pub daily_summary_hour: u8,
    #[serde(default = "default_daily_summary_enabled")]
    pub daily_summary_enabled: bool,
}

fn default_daily_summary_enabled() -> bool {
    true
}

impl Default for EmailSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            to: String::new(),
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_user: String::new(),
            weekly_day: 0,
            weekly_hour: 9,
            daily_summary_hour: 18,
            daily_summary_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub threads: u32,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub id: i64,
    pub ts: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub events_received: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub retention_days: u32,
    pub focus_mode: FocusModeState,
}

/// Full dashboard snapshot pushed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub fatigue_score: f64,
    pub zone: CognitiveZone,
    pub insight: String,
    pub cognitive_history: Vec<CognitiveBucket>,
    pub screen_time: ScreenTimeTotals,
    pub switches_last_30min: u32,
    pub errors_last_hour: u32,
    pub keystrokes_per_min: f64,
    pub active_window: String,
    pub active_app: String,
    pub active_category: String,
    pub notification_pending: bool,
    pub snoozed_until: Option<i64>,
    pub deep_work_score: f64,
    pub alert_level: Option<AlertLevel>,
    pub break_suggestion: Option<BreakSuggestion>,
    pub weekly_trend: Vec<TrendBucket>,
    pub monthly_trend: Vec<TrendBucket>,
    pub error_breakdown: Vec<ErrorBreakdown>,
    pub baseline: Option<BaselineMetrics>,
    pub anomalies: Vec<AnomalyReport>,
    pub proactive_suggestions: Vec<ProactiveSuggestion>,
    pub focus_mode: FocusModeState,
    pub benchmark: BenchmarkMetrics,
    pub plugins: Vec<PluginInfo>,
    pub theme: String,
    pub retention_days: u32,
    pub autostart_enabled: bool,
    pub app_usage: Vec<AppUsageEntry>,
    pub git_commits_today: u32,
    pub email_settings: EmailSettings,
}

/// Payload emitted to the frontend when an HTTP hook delivers a new event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CooldownEventPayload {
    pub event: DevEvent,
    pub category: ErrorCategory,
    pub ts: i64,
    pub fatigue_score: f64,
    pub errors_last_hour: u32,
    pub dashboard: DashboardState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakNotification {
    pub fatigue_score: f64,
    pub insight: String,
    pub level: AlertLevel,
    pub break_suggestion: BreakSuggestion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertToast {
    pub level: AlertLevel,
    pub message: String,
}
