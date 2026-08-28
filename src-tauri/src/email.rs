//! Optional SMTP email for weekly reports (opt-in, local-first).

use std::path::PathBuf;

use tauri::AppHandle;
use tauri::Manager;

use crate::db::Database;
use crate::reports;

pub fn reports_dir(app: &AppHandle) -> PathBuf {
    let dir = app.path().app_data_dir().expect("app data dir");
    let reports = dir.join("reports");
    std::fs::create_dir_all(&reports).ok();
    reports
}

pub fn save_weekly_report(app: &AppHandle, db: &Database) -> Result<PathBuf, String> {
    let html = reports::build_weekly_html(db);
    let name = format!(
        "cooldown-weekly-{}.html",
        chrono::Local::now().format("%Y-%m-%d")
    );
    let path = reports_dir(app).join(name);
    std::fs::write(&path, html).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn send_weekly_email(app: &AppHandle, db: &Database) -> Result<String, String> {
    let to = db.email_to().ok_or("Email recipient not configured")?;
    let smtp_host = db
        .smtp_host()
        .ok_or("SMTP host not configured in Settings")?;
    let smtp_port = db.smtp_port();
    let smtp_user = db.smtp_user();
    let smtp_pass = db.smtp_password();

    let path = save_weekly_report(app, db)?;
    let html = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let subject = format!(
        "Cooldown Weekly Report — {}",
        chrono::Local::now().format("%Y-%m-%d")
    );

    send_smtp(
        &smtp_host,
        smtp_port,
        smtp_user.as_deref(),
        smtp_pass.as_deref(),
        &to,
        &subject,
        &html,
    )?;

    db.set_last_weekly_report_date(&chrono::Local::now().format("%Y-%m-%d").to_string());
    Ok(format!("Weekly report sent to {to}"))
}

fn send_smtp(
    host: &str,
    port: u16,
    user: Option<&str>,
    pass: Option<&str>,
    to: &str,
    subject: &str,
    html: &str,
) -> Result<(), String> {
    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{Message, SmtpTransport, Transport};

    let from = user.unwrap_or("cooldown@localhost");
    let email = Message::builder()
        .from(from.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .to(to.parse().map_err(|e: lettre::address::AddressError| e.to_string())?)
        .subject(subject)
        .header(ContentType::TEXT_HTML)
        .body(html.to_string())
        .map_err(|e| e.to_string())?;

    let mut builder = SmtpTransport::relay(host).map_err(|e| e.to_string())?;
    builder = builder.port(port);

    let mailer = if let (Some(u), Some(p)) = (user, pass) {
        builder
            .credentials(Credentials::new(u.to_string(), p.to_string()))
            .build()
    } else {
        builder.build()
    };

    mailer.send(&email).map_err(|e| e.to_string())?;
    Ok(())
}
