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
            _ if event.exit_code.is_some() => Self::Terminal,
            _ => Self::Unknown,
        }
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
    pub fn classify(title: &str, app_name: &str) -> Self {
        let app = app_name.to_lowercase();
        let haystack = format!("{} {}", title, app_name).to_lowercase();

        // Match process / bundle name first (reliable on Windows .exe and macOS .app).
        const IDE_EXES: &[&str] = &[
            "code.exe", "cursor.exe", "devenv.exe", "idea64.exe", "idea.exe",
            "webstorm64.exe", "webstorm.exe", "pycharm64.exe", "pycharm.exe",
            "goland64.exe", "goland.exe", "rider64.exe", "rider.exe",
            "windowsterminal.exe", "wt.exe", "powershell.exe", "pwsh.exe", "cmd.exe",
            "sublime_text.exe", "notepad++.exe",
            // macOS process / bundle names
            "code.app", "cursor.app", "visual studio code.app", "xcode.app",
            "terminal.app", "iterm2.app", "iterm.app", "warp.app", "alacritty.app",
            "kitty.app", "ghostty.app", "wezterm.app", "hyper.app", "tabby.app",
            "sublime text.app", "nova.app", "bbedit.app", "zed.app",
            "datagrip.app", "pycharm.app", "webstorm.app", "goland.app", "rider.app",
            "intellij idea.app", "android studio.app", "fleet.app",
            // macOS process names (without .app suffix)
            "code", "cursor", "xcode", "terminal", "iterm2", "iterm", "warp",
            "alacritty", "kitty", "ghostty", "wezterm", "hyper", "tabby", "zed", "nova",
        ];
        const BROWSER_EXES: &[&str] = &[
            "chrome.exe", "firefox.exe", "msedge.exe", "brave.exe", "opera.exe",
            "vivaldi.exe", "iexplore.exe",
            "google chrome.app", "firefox.app", "safari.app", "brave browser.app",
            "microsoft edge.app", "opera.app", "arc.app", "vivaldi.app", "orion.app",
            "chrome", "firefox", "safari", "brave browser", "microsoft edge", "arc",
        ];
        const COMM_EXES: &[&str] = &[
            "slack.exe", "discord.exe", "teams.exe", "ms-teams.exe", "zoom.exe",
            "outlook.exe", "telegram.exe", "whatsapp.exe", "signal.exe", "skype.exe",
            "slack.app", "discord.app", "microsoft teams.app", "teams.app", "zoom.app",
            "outlook.app", "mail.app", "messages.app", "telegram.app", "whatsapp.app",
            "signal.app", "skype.app", "facetime.app",
        ];

        if matches_process(&app, IDE_EXES) {
            return Self::Ide;
        }
        if matches_process(&app, BROWSER_EXES) {
            return Self::Browser;
        }
        if matches_process(&app, COMM_EXES) {
            return Self::Communication;
        }

        const IDE: &[&str] = &[
            "visual studio", "jetbrains", "intellij", "neovim", "nvim", "vim", "emacs",
            "sublime", "zed", "android studio", "xcode", "github copilot",
            "iterm", "warp", "alacritty", "wezterm",
        ];
        const BROWSER: &[&str] = &[
            "chrome", "firefox", "edge", "safari", "brave", "opera", "vivaldi",
        ];
        const COMM: &[&str] = &[
            "slack", "discord", "teams", "zoom", "outlook", "mail", "telegram",
            "whatsapp", "signal", "skype",
        ];

        if IDE.iter().any(|k| haystack.contains(k)) {
            Self::Ide
        } else if BROWSER.iter().any(|k| haystack.contains(k)) {
            Self::Browser
        } else if COMM.iter().any(|k| haystack.contains(k)) {
            Self::Communication
        } else {
            Self::Other
        }
    }
}

fn matches_process(app: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| {
        let pattern = pattern.to_lowercase();
        app == pattern
            || app.ends_with(&pattern)
            || app.ends_with(&format!("/{pattern}"))
            || app.ends_with(&format!("\\{pattern}"))
    })
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusModeState {
    pub active: bool,
    pub until: Option<i64>,
    pub session_secs: u64,
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
