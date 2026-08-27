//! Sensitive window title filtering — banking, password managers, etc.

const SENSITIVE_KEYWORDS: &[&str] = &[
    "password", "passwd", "1password", "lastpass", "bitwarden", "keepass", "dashlane",
    "bank", "banking", "garanti", "ziraat", "iş bank", "isbank", "akbank", "yapı kredi",
    "paypal", "stripe dashboard", "vault", "private browsing", "incognito",
    "kimlik", "tc kimlik", "ssn", "credit card",
];

/// Returns true if the window title/app should NOT be tracked or displayed.
pub fn is_sensitive(title: &str, app_name: &str) -> bool {
    let haystack = format!("{} {}", title, app_name).to_lowercase();
    SENSITIVE_KEYWORDS.iter().any(|k| haystack.contains(k))
}

/// Redacted label shown in UI/logs when sensitive.
pub fn redacted_label() -> &'static str {
    "[Protected Window]"
}
