//! Shared application state integrating persistence, analytics, focus, and alerts.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Local, Timelike};
use parking_lot::RwLock;

use crate::analytics::{detect_anomalies, proactive_suggestions, DeepWorkTracker};
use crate::benchmark::BenchmarkMonitor;
use crate::db::Database;
use crate::fatigue::{compute_fatigue_score, generate_insight, zone_from_score, FatigueInputs};
use crate::focus::FocusController;
use crate::models::{
    AlertLevel, AppCategory, BreakNotification, CognitiveBucket, CognitiveZone, DailySummary,
    DashboardState, DevEvent, ErrorCategory, PomodoroNotification, ScreenTimeTotals, TrendPeriod,
};
use crate::notifications::{
    break_suggestion_for, build_hint_toast, build_notification, build_pomodoro_toast,
};
use crate::plugins::PluginRegistry;
use crate::privacy::{is_sensitive, redacted_label};

const SWITCH_WINDOW: Duration = Duration::from_secs(30 * 60);
const ERROR_WINDOW: Duration = Duration::from_secs(60 * 60);
const KEYSTROKE_WINDOW: Duration = Duration::from_secs(5 * 60);
const TYPING_PAUSE: Duration = Duration::from_secs(20);
const CONSTANT_TYPING_WINDOW: Duration = Duration::from_secs(2 * 60);
const MIN_KEYSTROKES_FOR_CONSTANT: u32 = 30;

pub struct AppState {
    inner: RwLock<StateInner>,
    pub db: Arc<Database>,
    pub plugins: Arc<PluginRegistry>,
    benchmark: RwLock<BenchmarkMonitor>,
}

struct StateInner {
    window_switches: VecDeque<Instant>,
    error_events: VecDeque<(Instant, String)>,
    keystrokes: VecDeque<Instant>,
    last_keystroke: Option<Instant>,
    cognitive_buckets: Vec<CognitiveBucket>,
    screen_time: ScreenTimeTotals,
    current_window: String,
    current_app: String,
    current_category: AppCategory,
    category_since: Instant,
    fatigue_score: f64,
    notification_pending: bool,
    notification_shown: bool,
    snoozed_until: Option<i64>,
    last_bucket_hour: Option<u32>,
    last_persist_hour: Option<u32>,
    last_hint_shown: Option<AlertLevel>,
    deep_work: DeepWorkTracker,
    focus: FocusController,
    /// When true, time on sensitive windows is not counted.
    tracking_paused: bool,
}

impl AppState {
    pub fn new(db: Arc<Database>, plugins: Arc<PluginRegistry>) -> Arc<Self> {
        let screen_time = db.load_screen_time_today();
        Arc::new(Self {
            inner: RwLock::new(StateInner {
                window_switches: VecDeque::new(),
                error_events: VecDeque::new(),
                keystrokes: VecDeque::new(),
                last_keystroke: None,
                cognitive_buckets: Vec::new(),
                screen_time,
                current_window: String::new(),
                current_app: String::from("Unknown"),
                current_category: AppCategory::Other,
                category_since: Instant::now(),
                fatigue_score: 0.0,
                notification_pending: false,
                notification_shown: false,
                snoozed_until: None,
                last_bucket_hour: None,
                last_persist_hour: None,
                last_hint_shown: None,
                deep_work: DeepWorkTracker::default(),
                focus: FocusController::default(),
                tracking_paused: false,
            }),
            db,
            plugins,
            benchmark: RwLock::new(BenchmarkMonitor::new()),
        })
    }

    /// Flush elapsed time for the current foreground app into screen-time totals.
    pub fn tick_screen_time(&self) {
        let mut inner = self.inner.write();
        Self::flush_active_time(&mut inner, &self.db);
        self.db.upsert_screen_time(
            inner.screen_time.ide_secs,
            inner.screen_time.browser_secs,
            inner.screen_time.communication_secs,
            inner.screen_time.other_secs,
        );
    }

    /// Stop accruing screen time while Cooldown itself is focused.
    pub fn pause_self_tracking(&self) {
        let mut inner = self.inner.write();
        Self::flush_active_time(&mut inner, &self.db);
        inner.tracking_paused = true;
    }

    pub fn record_window_change(&self, title: String, app_name: String) {
        let mut inner = self.inner.write();
        Self::flush_active_time(&mut inner, &self.db);

        if is_sensitive(&title, &app_name) {
            inner.tracking_paused = true;
            inner.current_window = redacted_label().into();
            inner.current_app = String::from("protected");
            inner.category_since = Instant::now();
            return;
        }

        inner.tracking_paused = false;
        let category = AppCategory::classify(&title, &app_name);
        let window_changed = inner.current_window != title;
        let app_changed = inner.current_app != app_name;
        let category_changed = inner.current_category != category;

        let had_prior = !inner.current_app.is_empty()
            && inner.current_app != "Unknown"
            && inner.current_app != "protected"
            && !inner.current_window.is_empty();

        if (window_changed || app_changed) && had_prior {
            inner.window_switches.push_back(Instant::now());
            prune_old(&mut inner.window_switches, SWITCH_WINDOW);
        }

        if window_changed || app_changed {
            inner.current_window = title;
            inner.current_app = app_name;
        }

        if window_changed || app_changed || category_changed {
            inner.category_since = Instant::now();
        }

        inner.current_category = category;

        Self::recompute(&mut inner, &self.db);
        self.db.upsert_screen_time(
            inner.screen_time.ide_secs,
            inner.screen_time.browser_secs,
            inner.screen_time.communication_secs,
            inner.screen_time.other_secs,
        );
    }

    pub fn record_keystroke(&self) {
        let mut inner = self.inner.write();
        let now = Instant::now();
        inner.keystrokes.push_back(now);
        inner.last_keystroke = Some(now);
        prune_old(&mut inner.keystrokes, KEYSTROKE_WINDOW);
        Self::recompute(&mut inner, &self.db);
    }

    pub fn record_dev_event(&self, event: DevEvent) {
        self.plugins.ingest(&event);
        let ts = chrono::Utc::now().timestamp();

        if event.is_activity() {
            let event_type = event.event.as_deref().unwrap_or("activity");
            self.db.record_activity(
                ts,
                &event.source,
                event_type,
                event.message.as_deref(),
            );
            return;
        }

        if !ErrorCategory::is_error_event(&event) {
            return;
        }

        let category = ErrorCategory::from_event(&event);
        self.db.record_error(
            ts,
            &event.source,
            category,
            event.message.as_deref(),
        );

        let mut inner = self.inner.write();
        let label = format!("{}:{}", event.source, category.as_str());
        inner.error_events.push_back((Instant::now(), label));
        prune_old_pairs(&mut inner.error_events, ERROR_WINDOW);
        Self::recompute(&mut inner, &self.db);
    }

    pub fn start_pomodoro(&self) {
        let mut inner = self.inner.write();
        inner.focus.start_pomodoro();
        inner.focus.sync_expiry();
    }

    pub fn set_focus_mode(&self, active: bool, duration_min: u32) {
        let mut inner = self.inner.write();
        if active {
            inner.focus.enable(duration_min);
        } else {
            inner.focus.disable();
        }
        inner.focus.sync_expiry();
    }

    pub fn snooze(&self, minutes: u32) {
        let mut inner = self.inner.write();
        let until = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + (minutes as i64 * 60);
        inner.snoozed_until = Some(until);
        inner.notification_pending = false;
        inner.notification_shown = false;
    }

    pub fn dismiss_notification(&self) {
        let mut inner = self.inner.write();
        inner.notification_pending = false;
        inner.notification_shown = true;
    }

    pub fn save_journal(&self, text: &str) {
        let ts = chrono::Utc::now().timestamp();
        self.db.save_journal(ts, text);
        self.dismiss_notification();
    }

    pub fn dashboard(&self) -> DashboardState {
        self.tick_screen_time();
        {
            let mut inner = self.inner.write();
            inner.focus.sync_expiry();
        }
        let inner = self.inner.read();
        let switches = count_in_window(&inner.window_switches, SWITCH_WINDOW);
        let errors = count_pairs_in_window(&inner.error_events, ERROR_WINDOW);
        let keystrokes = count_in_window(&inner.keystrokes, KEYSTROKE_WINDOW);
        let keystrokes_per_min = keystrokes as f64 / 5.0;
        let deep_work_score = inner.deep_work.score();

        let peak_hour = inner
            .cognitive_buckets
            .iter()
            .max_by(|a, b| a.fatigue.partial_cmp(&b.fatigue).unwrap())
            .map(|b| b.hour.clone());

        let baseline = self.db.load_baseline();
        let weekly = self.db.trend(TrendPeriod::Weekly);
        let monthly = self.db.trend(TrendPeriod::Monthly);
        let since_day = Local::now().timestamp() - 86400;
        let error_breakdown: Vec<_> = self
            .db
            .error_breakdown(since_day)
            .into_iter()
            .map(|(category, count)| crate::models::ErrorBreakdown { category, count })
            .collect();

        let anomalies = detect_anomalies(
            inner.fatigue_score,
            switches as f64,
            errors as f64,
            keystrokes_per_min,
            deep_work_score,
            baseline.as_ref(),
        );

        let hour_map = crate::analytics::peak_hours_from_trends(&weekly);
        let proactive = proactive_suggestions(&hour_map);

        let alert_level = AlertLevel::from_fatigue(inner.fatigue_score);
        let break_suggestion = alert_level
            .map(|l| break_suggestion_for(l, inner.fatigue_score, errors));

        let mut benchmark = self.benchmark.write();
        let bench = benchmark.snapshot();

        DashboardState {
            fatigue_score: inner.fatigue_score,
            zone: zone_from_score(inner.fatigue_score),
            insight: generate_insight(
                inner.fatigue_score,
                zone_from_score(inner.fatigue_score),
                switches,
                errors,
                peak_hour.as_deref(),
            ),
            cognitive_history: inner.cognitive_buckets.clone(),
            screen_time: inner.screen_time.clone(),
            switches_last_30min: switches,
            errors_last_hour: errors,
            keystrokes_per_min,
            active_window: inner.current_window.clone(),
            active_app: inner.current_app.clone(),
            active_category: inner.current_category.as_str().to_string(),
            notification_pending: inner.notification_pending,
            snoozed_until: inner.snoozed_until,
            deep_work_score,
            alert_level,
            break_suggestion,
            weekly_trend: weekly,
            monthly_trend: monthly,
            error_breakdown,
            baseline,
            anomalies,
            proactive_suggestions: proactive,
            focus_mode: inner.focus.state(),
            benchmark: bench,
            plugins: self.plugins.list(),
            theme: self.db.theme(),
            retention_days: self.db.retention_days(),
            autostart_enabled: self.db.autostart_enabled(),
            app_usage: self.db.app_usage_today(),
            git_commits_today: self.db.git_commits_today(),
            email_settings: self.db.load_email_settings(),
        }
    }

    pub fn take_pomodoro_notification(&self) -> Option<PomodoroNotification> {
        let mut inner = self.inner.write();
        let message = inner.focus.take_phase_message()?;
        let state = inner.focus.state();
        build_pomodoro_toast(message, state.phase, state.cycle)
    }

    pub fn check_daily_summary(&self) -> Option<DailySummary> {
        if !self.db.daily_summary_enabled() {
            return None;
        }
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        if self.db.last_daily_summary_date().as_deref() == Some(today.as_str()) {
            return None;
        }
        if now.hour() as u8 != self.db.daily_summary_hour() {
            return None;
        }
        let summary = crate::reports::build_daily_summary(&self.db);
        self.db.set_last_daily_summary_date(&today);
        Some(summary)
    }

    pub fn maybe_send_weekly_email(&self, app: &tauri::AppHandle) -> Option<String> {
        if !self.db.email_enabled() {
            return None;
        }
        let now = Local::now();
        let today = now.format("%Y-%m-%d").to_string();
        if self.db.last_weekly_report_date().as_deref() == Some(today.as_str()) {
            return None;
        }
        if now.weekday().num_days_from_monday() as u8 != self.db.weekly_email_day() {
            return None;
        }
        if now.hour() as u8 != self.db.weekly_email_hour() {
            return None;
        }
        crate::email::send_weekly_email(app, &self.db).ok()
    }

    pub fn check_notification(&self) -> Option<BreakNotification> {
        let mut inner = self.inner.write();
        let errors = count_pairs_in_window(&inner.error_events, ERROR_WINDOW);

        if inner.focus.should_suppress_alert(inner.fatigue_score) {
            return None;
        }

        let level = AlertLevel::from_fatigue(inner.fatigue_score)?;
        if level == AlertLevel::Hint {
            return None;
        }

        if inner.notification_shown {
            return None;
        }

        if let Some(until) = inner.snoozed_until {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            if now < until {
                return None;
            }
        }

        let now = Instant::now();
        let recent_keystrokes = count_in_window(&inner.keystrokes, CONSTANT_TYPING_WINDOW);
        if recent_keystrokes >= MIN_KEYSTROKES_FOR_CONSTANT {
            if let Some(last) = inner.last_keystroke {
                if now.duration_since(last) < TYPING_PAUSE {
                    inner.notification_pending = true;
                    return None;
                }
            }
        }

        inner.notification_pending = false;
        inner.notification_shown = true;

        let insight = generate_insight(
            inner.fatigue_score,
            CognitiveZone::Burnout,
            count_in_window(&inner.window_switches, SWITCH_WINDOW),
            errors,
            None,
        );

        build_notification(inner.fatigue_score, insight, errors)
    }

    pub fn check_hint_toast(&self) -> Option<crate::models::AlertToast> {
        let mut inner = self.inner.write();
        if inner.focus.should_suppress_hint() {
            return None;
        }
        let toast = build_hint_toast(inner.fatigue_score)?;
        if inner.last_hint_shown == Some(AlertLevel::Hint) {
            return None;
        }
        inner.last_hint_shown = Some(AlertLevel::Hint);
        Some(toast)
    }

    pub fn maybe_update_baseline(&self) {
        if let Some(metrics) = self.db.compute_baseline_from_history() {
            self.db.update_baseline(&metrics);
        }
    }

    fn flush_active_time(inner: &mut StateInner, db: &Database) {
        if inner.tracking_paused {
            inner.category_since = Instant::now();
            return;
        }

        let elapsed = inner.category_since.elapsed().as_secs();
        if elapsed == 0 {
            return;
        }

        inner.deep_work.tick(inner.current_category, elapsed);
        inner.focus.tick(elapsed, inner.current_category == AppCategory::Ide);

        match inner.current_category {
            AppCategory::Ide => inner.screen_time.ide_secs += elapsed,
            AppCategory::Browser => inner.screen_time.browser_secs += elapsed,
            AppCategory::Communication => inner.screen_time.communication_secs += elapsed,
            AppCategory::Other => inner.screen_time.other_secs += elapsed,
        }

        if !inner.current_app.is_empty()
            && inner.current_app != "Unknown"
            && inner.current_app != "protected"
        {
            db.upsert_app_usage(
                &inner.current_app,
                inner.current_category.as_str(),
                elapsed,
            );
        }

        inner.category_since = Instant::now();
    }

    fn recompute(inner: &mut StateInner, db: &Database) {
        let switches = count_in_window(&inner.window_switches, SWITCH_WINDOW);
        let errors = count_pairs_in_window(&inner.error_events, ERROR_WINDOW);
        let keystrokes = count_in_window(&inner.keystrokes, KEYSTROKE_WINDOW);

        let inputs = FatigueInputs {
            switches_last_30min: switches,
            errors_last_hour: errors,
            keystrokes_per_min: keystrokes as f64 / 5.0,
        };

        inner.fatigue_score = compute_fatigue_score(&inputs);

        if inner.fatigue_score >= 75.0 && !inner.notification_shown {
            inner.notification_pending = true;
        }
        if inner.fatigue_score < 60.0 {
            inner.last_hint_shown = None;
            inner.notification_shown = false;
        }

        Self::update_hourly_bucket(inner, switches, errors, db);
    }

    fn update_hourly_bucket(inner: &mut StateInner, switches: u32, errors: u32, db: &Database) {
        let hour = Local::now().hour();
        let label = format!("{:02}:00", hour);
        let deep = inner.deep_work.score();
        let cpm = count_in_window(&inner.keystrokes, KEYSTROKE_WINDOW) as f64 / 5.0;

        if inner.last_bucket_hour != Some(hour) {
            inner.cognitive_buckets.push(CognitiveBucket {
                hour: label.clone(),
                fatigue: inner.fatigue_score,
                zone: zone_from_score(inner.fatigue_score),
                switch_count: switches,
                error_count: errors,
            });
            inner.last_bucket_hour = Some(hour);
            if inner.cognitive_buckets.len() > 24 {
                inner.cognitive_buckets.remove(0);
            }
        } else if let Some(bucket) = inner.cognitive_buckets.last_mut() {
            bucket.fatigue = inner.fatigue_score;
            bucket.zone = zone_from_score(inner.fatigue_score);
            bucket.switch_count = switches;
            bucket.error_count = errors;
        }

        if inner.last_persist_hour != Some(hour) {
            inner.last_persist_hour = Some(hour);
            db.persist_hourly(
                Local::now().timestamp(),
                inner.fatigue_score,
                switches,
                errors,
                deep,
                cpm,
            );
        }
    }
}

fn prune_old(queue: &mut VecDeque<Instant>, window: Duration) {
    let cutoff = Instant::now() - window;
    while queue.front().is_some_and(|t| *t < cutoff) {
        queue.pop_front();
    }
}

fn prune_old_pairs(queue: &mut VecDeque<(Instant, String)>, window: Duration) {
    let cutoff = Instant::now() - window;
    while queue.front().is_some_and(|(t, _)| *t < cutoff) {
        queue.pop_front();
    }
}

fn count_in_window(queue: &VecDeque<Instant>, window: Duration) -> u32 {
    let cutoff = Instant::now() - window;
    queue.iter().filter(|t| **t >= cutoff).count() as u32
}

fn count_pairs_in_window(queue: &VecDeque<(Instant, String)>, window: Duration) -> u32 {
    let cutoff = Instant::now() - window;
    queue.iter().filter(|(t, _)| *t >= cutoff).count() as u32
}
