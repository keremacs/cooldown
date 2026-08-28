//! Daily & weekly report builders.

use chrono::Local;

use crate::db::Database;
use crate::models::{DailySummary, ScreenTimeTotals};

pub fn build_daily_summary(db: &Database) -> DailySummary {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let screen = db.load_screen_time_today();
    let apps = db.app_usage_today();
    let git_commits = db.git_commits_today();
    let peak_fatigue = db.peak_fatigue_today();
    let total_errors = db.error_count_today();
    let journal_count = db.journal_count_today();
    let switches = db.total_switches_today();

    let message = format_daily_message(&screen, git_commits, peak_fatigue, total_errors);
    DailySummary {
        date: date.clone(),
        screen_time: screen,
        top_apps: apps.into_iter().take(5).collect(),
        git_commits,
        peak_fatigue,
        total_errors,
        journal_entries: journal_count,
        context_switches: switches,
        message,
    }
}

fn format_daily_message(
    screen: &ScreenTimeTotals,
    commits: u32,
    peak: f64,
    errors: u32,
) -> String {
    let total = screen.ide_secs + screen.browser_secs + screen.communication_secs + screen.other_secs;
    let ide_pct = if total > 0 {
        (screen.ide_secs as f64 / total as f64 * 100.0) as u32
    } else {
        0
    };
    format!(
        "Today: {}% IDE time, peak fatigue {:.0}, {} errors, {} git commits.",
        ide_pct, peak, errors, commits
    )
}

pub fn build_weekly_html(db: &Database) -> String {
    let weekly = db.trend(crate::models::TrendPeriod::Weekly);
    let apps = db.app_usage_week();
    let git_commits = db.git_commits_week();
    let today = Local::now().format("%Y-%m-%d").to_string();

    let mut rows = String::new();
    for bucket in &weekly {
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{:.1}</td><td>{:.1}</td><td>{:.1}</td></tr>",
            bucket.label, bucket.avg_fatigue, bucket.avg_switches, bucket.avg_errors
        ));
    }

    let mut app_rows = String::new();
    for app in apps.iter().take(10) {
        app_rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            app.app_name,
            app.category,
            format_secs(app.secs)
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Cooldown Weekly Report</title>
<style>
body {{ font-family: system-ui, sans-serif; max-width: 720px; margin: 2rem auto; color: #1e293b; }}
h1 {{ color: #4f46e5; }} table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
th, td {{ border: 1px solid #e2e8f0; padding: 8px; text-align: left; }}
th {{ background: #f1f5f9; }} .stat {{ background: #eef2ff; padding: 1rem; border-radius: 8px; }}
</style></head><body>
<h1>Cooldown Weekly Report</h1>
<p>Generated {today} · Local-first developer wellness</p>
<div class="stat"><strong>Git commits this week:</strong> {git_commits}</div>
<h2>Daily Trends</h2>
<table><tr><th>Day</th><th>Avg Fatigue</th><th>Avg Switches</th><th>Avg Errors</th></tr>{rows}</table>
<h2>Top Apps (7 days)</h2>
<table><tr><th>App</th><th>Category</th><th>Time</th></tr>{app_rows}</table>
<p style="color:#64748b;font-size:0.85rem">Data stored locally on your device. Cooldown v0.2</p>
</body></html>"#
    )
}

fn format_secs(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

pub fn weekly_summary_text(db: &Database) -> String {
    let weekly = db.trend(crate::models::TrendPeriod::Weekly);
    let git = db.git_commits_week();
    let avg_fatigue = if weekly.is_empty() {
        0.0
    } else {
        weekly.iter().map(|b| b.avg_fatigue).sum::<f64>() / weekly.len() as f64
    };
    format!(
        "Cooldown weekly: avg fatigue {:.0}, {} git commits, {} days tracked.",
        avg_fatigue,
        git,
        weekly.len()
    )
}
