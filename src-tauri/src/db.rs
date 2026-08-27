//! SQLite persistence — metrics, journal, baseline, settings, retention policy.

use std::sync::Arc;

use chrono::Local;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

use crate::models::{
    BaselineMetrics, ErrorCategory, JournalEntry, TrendBucket, TrendPeriod,
};

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(app: &AppHandle) -> Arc<Self> {
        let dir = app.path().app_data_dir().expect("app data dir");
        std::fs::create_dir_all(&dir).ok();
        let path = dir.join("cooldown.db");
        let conn = Connection::open(&path).expect("open sqlite");
        let db = Arc::new(Self {
            conn: Mutex::new(conn),
        });
        db.init_schema();
        db
    }

    fn init_schema(&self) {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS hourly_metrics (
                id INTEGER PRIMARY KEY,
                ts INTEGER NOT NULL,
                fatigue REAL NOT NULL,
                switches INTEGER NOT NULL DEFAULT 0,
                errors INTEGER NOT NULL DEFAULT 0,
                deep_work_score REAL NOT NULL DEFAULT 0,
                keystrokes_per_min REAL NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_hourly_ts ON hourly_metrics(ts);

            CREATE TABLE IF NOT EXISTS error_events (
                id INTEGER PRIMARY KEY,
                ts INTEGER NOT NULL,
                source TEXT NOT NULL,
                category TEXT NOT NULL,
                message TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_error_ts ON error_events(ts);

            CREATE TABLE IF NOT EXISTS journal_entries (
                id INTEGER PRIMARY KEY,
                ts INTEGER NOT NULL,
                text TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS screen_time_daily (
                date TEXT PRIMARY KEY,
                ide_secs INTEGER NOT NULL DEFAULT 0,
                browser_secs INTEGER NOT NULL DEFAULT 0,
                communication_secs INTEGER NOT NULL DEFAULT 0,
                other_secs INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS break_sessions (
                id INTEGER PRIMARY KEY,
                ts_start INTEGER NOT NULL,
                ts_end INTEGER,
                kind TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS baseline (
                metric TEXT PRIMARY KEY,
                value REAL NOT NULL,
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS focus_sessions (
                id INTEGER PRIMARY KEY,
                ts_start INTEGER NOT NULL,
                ts_end INTEGER,
                duration_min INTEGER NOT NULL
            );
            ",
        )
        .expect("schema");
    }

    pub fn retention_days(&self) -> u32 {
        self.get_setting("retention_days")
            .and_then(|v| v.parse().ok())
            .unwrap_or(90)
    }

    pub fn set_retention_days(&self, days: u32) {
        self.set_setting("retention_days", &days.to_string());
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |r| r.get(0),
        )
        .ok()
    }

    pub fn set_setting(&self, key: &str, value: &str) {
        let conn = self.conn.lock();
        let _ = conn.execute(
            "INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
            params![key, value],
        );
    }

    pub fn theme(&self) -> String {
        self.get_setting("theme").unwrap_or_else(|| "dark".into())
    }

    pub fn set_theme(&self, theme: &str) {
        self.set_setting("theme", theme);
    }

    pub fn persist_hourly(
        &self,
        ts: i64,
        fatigue: f64,
        switches: u32,
        errors: u32,
        deep_work: f64,
        cpm: f64,
    ) {
        let conn = self.conn.lock();
        let _ = conn.execute(
            "INSERT INTO hourly_metrics(ts,fatigue,switches,errors,deep_work_score,keystrokes_per_min) VALUES(?1,?2,?3,?4,?5,?6)",
            params![ts, fatigue, switches, errors, deep_work, cpm],
        );
    }

    pub fn record_error(&self, ts: i64, source: &str, category: ErrorCategory, message: Option<&str>) {
        let conn = self.conn.lock();
        let _ = conn.execute(
            "INSERT INTO error_events(ts,source,category,message) VALUES(?1,?2,?3,?4)",
            params![ts, source, category.as_str(), message],
        );
    }

    pub fn save_journal(&self, ts: i64, text: &str) -> i64 {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO journal_entries(ts,text) VALUES(?1,?2)",
            params![ts, text],
        )
        .expect("journal insert");
        conn.last_insert_rowid()
    }

    pub fn journal_entries(&self, limit: u32) -> Vec<JournalEntry> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare("SELECT id, ts, text FROM journal_entries ORDER BY ts DESC LIMIT ?1")
            .expect("journal query");
        stmt.query_map(params![limit], |r| {
            Ok(JournalEntry {
                id: r.get(0)?,
                ts: r.get(1)?,
                text: r.get(2)?,
            })
        })
        .expect("journal map")
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn upsert_screen_time(
        &self,
        ide: u64,
        browser: u64,
        comm: u64,
        other: u64,
    ) {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let conn = self.conn.lock();
        let _ = conn.execute(
            "INSERT INTO screen_time_daily(date,ide_secs,browser_secs,communication_secs,other_secs)
             VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(date) DO UPDATE SET
               ide_secs=excluded.ide_secs,
               browser_secs=excluded.browser_secs,
               communication_secs=excluded.communication_secs,
               other_secs=excluded.other_secs",
            params![date, ide as i64, browser as i64, comm as i64, other as i64],
        );
    }

    pub fn start_break(&self, ts: i64, kind: &str) -> i64 {
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO break_sessions(ts_start,kind) VALUES(?1,?2)",
            params![ts, kind],
        )
        .expect("break insert");
        conn.last_insert_rowid()
    }

    pub fn end_break(&self, id: i64, ts: i64) {
        let conn = self.conn.lock();
        let _ = conn.execute(
            "UPDATE break_sessions SET ts_end = ?1 WHERE id = ?2",
            params![ts, id],
        );
    }

    pub fn update_baseline(&self, metrics: &BaselineMetrics) {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        for (metric, value) in [
            ("fatigue", metrics.avg_fatigue),
            ("switches", metrics.avg_switches),
            ("errors", metrics.avg_errors),
            ("cpm", metrics.avg_keystrokes_per_min),
            ("deep_work", metrics.avg_deep_work),
        ] {
            let _ = conn.execute(
                "INSERT INTO baseline(metric,value,updated_at) VALUES(?1,?2,?3)
                 ON CONFLICT(metric) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at",
                params![metric, value, now],
            );
        }
    }

    pub fn load_baseline(&self) -> Option<BaselineMetrics> {
        let conn = self.conn.lock();
        let mut out = BaselineMetrics::default();
        let mut found = false;
        if let Ok(mut stmt) = conn.prepare("SELECT metric, value FROM baseline") {
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, f64>(1)?)))
                .ok()?;
            for row in rows.flatten() {
                found = true;
                match row.0.as_str() {
                    "fatigue" => out.avg_fatigue = row.1,
                    "switches" => out.avg_switches = row.1,
                    "errors" => out.avg_errors = row.1,
                    "cpm" => out.avg_keystrokes_per_min = row.1,
                    "deep_work" => out.avg_deep_work = row.1,
                    _ => {}
                }
            }
        }
        if found { Some(out) } else { None }
    }

    pub fn trend(&self, period: TrendPeriod) -> Vec<TrendBucket> {
        let days = match period {
            TrendPeriod::Weekly => 7,
            TrendPeriod::Monthly => 30,
        };
        let cutoff = Local::now().timestamp() - (days as i64 * 86400);
        let conn = self.conn.lock();
        let sql = match period {
            TrendPeriod::Weekly => {
                "SELECT date(ts,'unixepoch','localtime') as d,
                        AVG(fatigue), AVG(switches), AVG(errors), AVG(deep_work_score)
                 FROM hourly_metrics WHERE ts >= ?1 GROUP BY d ORDER BY d"
            }
            TrendPeriod::Monthly => {
                "SELECT strftime('%Y-%W', ts, 'unixepoch', 'localtime') as d,
                        AVG(fatigue), AVG(switches), AVG(errors), AVG(deep_work_score)
                 FROM hourly_metrics WHERE ts >= ?1 GROUP BY d ORDER BY d"
            }
        };
        let mut stmt = conn.prepare(sql).expect("trend sql");
        stmt.query_map(params![cutoff], |r| {
            Ok(TrendBucket {
                label: r.get(0)?,
                avg_fatigue: r.get(1)?,
                avg_switches: r.get(2)?,
                avg_errors: r.get(3)?,
                avg_deep_work: r.get(4)?,
            })
        })
        .expect("trend map")
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn error_breakdown(&self, since_ts: i64) -> Vec<(ErrorCategory, u32)> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT category, COUNT(*) FROM error_events WHERE ts >= ?1 GROUP BY category",
            )
            .expect("error breakdown");
        stmt.query_map(params![since_ts], |r| {
            let cat_str: String = r.get(0)?;
            let count: u32 = r.get(1)?;
            Ok((ErrorCategory::from_str(&cat_str), count))
        })
        .expect("breakdown map")
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn compute_baseline_from_history(&self) -> Option<BaselineMetrics> {
        let cutoff = Local::now().timestamp() - (14 * 86400);
        let conn = self.conn.lock();
        conn.query_row(
            "SELECT AVG(fatigue), AVG(switches), AVG(errors), AVG(keystrokes_per_min), AVG(deep_work_score)
             FROM hourly_metrics WHERE ts >= ?1",
            params![cutoff],
            |r| {
                Ok(BaselineMetrics {
                    avg_fatigue: r.get(0)?,
                    avg_switches: r.get(1)?,
                    avg_errors: r.get(2)?,
                    avg_keystrokes_per_min: r.get(3)?,
                    avg_deep_work: r.get(4)?,
                })
            },
        )
        .ok()
    }

    pub fn apply_retention_policy(&self) {
        let days = self.retention_days();
        let cutoff = Local::now().timestamp() - (days as i64 * 86400);
        let conn = self.conn.lock();
        let _ = conn.execute("DELETE FROM hourly_metrics WHERE ts < ?1", params![cutoff]);
        let _ = conn.execute("DELETE FROM error_events WHERE ts < ?1", params![cutoff]);
        let _ = conn.execute("DELETE FROM journal_entries WHERE ts < ?1", params![cutoff]);
        let _ = conn.execute("DELETE FROM break_sessions WHERE ts_start < ?1", params![cutoff]);
        // Keep screen_time_daily for same retention window
        let date_cutoff = (Local::now() - chrono::Duration::days(days as i64))
            .format("%Y-%m-%d")
            .to_string();
        let _ = conn.execute(
            "DELETE FROM screen_time_daily WHERE date < ?1",
            params![date_cutoff],
        );
    }
}
