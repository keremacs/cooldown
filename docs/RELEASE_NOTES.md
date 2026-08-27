# Cooldown Release Notes

## v0.2.0 — Developer Wellness Update

**Release date:** August 2026

Cooldown v0.2 is a major step forward: persistent analytics, smarter break coaching, deep IDE/terminal integration, and cross-platform support for Windows and macOS.

---

### Highlights

- **Live Fatigue Score (0–100)** — Combines context switches, error density, and typing cadence into a real-time burnout indicator.
- **Local-first architecture** — All data stays on your device in SQLite. No cloud, no accounts, no telemetry.
- **IDE & terminal integration** — VS Code, Cursor, PowerShell, and Zsh hooks feed errors and failed commands into the score.
- **macOS support** — Native `.app` / `.dmg` builds with screen lock detection, menu bar tray, and Mac app recognition.

---

### New Features

#### Dashboard & Analytics
- **Cognitive Map** — Hourly fatigue visualization with zone coloring (Flow / Distraction / Burnout).
- **Screen Time** — Tracks time across IDEs, browsers, communication apps, and other categories.
- **Trends tab** — Weekly and monthly fatigue trend reports.
- **Insights tab** — Deep work score, error type breakdown, personal 14-day baseline, anomaly detection, and proactive break suggestions.
- **Journal tab** — Micro-journal archive for break-time notes.

#### Smart Notifications
- **Graduated alerts** — Hint at 60, break popup at 75, critical alert at 90.
- **Contextual break suggestions** — Recommendations based on current fatigue zone and activity.
- **Smart snooze** — Defer reminders without losing track of rising fatigue.
- **Focus mode** — 25/50-minute sessions that suppress non-critical alerts (warnings below 90).

#### Privacy & Performance
- **Sensitive window filtering** — Auto-redacts banking, password managers, and private browsing windows.
- **Data retention policy** — Configurable 30–365 day retention in Settings.
- **Benchmark mode** — Self-monitoring for RAM (~80–130 MB), CPU, and uptime.
- **Screen lock detection** — Tracks real break time when your screen is locked (Windows + macOS).

#### UI & System Integration
- **System tray / menu bar** — Color-coded icon (green / yellow / red) reflecting live fatigue.
- **Mini widget** — Draggable always-on-top widget with minimize and close controls.
- **Dark & soft light themes** — Comfortable viewing in any environment.
- **Close-to-tray** — Closing the main window hides to tray instead of quitting.

#### Integrations
- **VS Code / Cursor extension** — Captures diagnostics and failed tasks.
- **Zsh hook** — Reports failed terminal commands (macOS / Linux).
- **PowerShell hook** — Reports failed commands on Windows.
- **Plugin registry** — Tracks event sources (vscode, terminal, powershell) in Settings.

---

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Windows 10/11 | ✅ Supported | Full feature set, tray icon, screen lock detection |
| macOS 10.15+ | ✅ Supported | Requires Accessibility + Input Monitoring permissions |
| Linux | ⚠️ Experimental | Core app runs; tray and permissions vary by desktop environment |

---

### Technical Stack

- **Backend:** Tauri 2 + Rust (axum HTTP server, rusqlite, rdev, sysinfo)
- **Frontend:** React + TypeScript + Tailwind CSS + Recharts
- **Storage:** Local SQLite database in app data directory
- **Event API:** `POST http://127.0.0.1:9876/event`

---

### Upgrade Notes

- First launch after upgrade creates the SQLite schema automatically.
- Run `npm run icons` before building on macOS to generate `icon.icns`.
- Grant **Accessibility** and **Input Monitoring** permissions on macOS for window tracking and keyboard tempo.
- See [MACOS.md](./MACOS.md) for full macOS setup instructions.

---

### Known Limitations

- Keyboard tracking measures **cadence only** — keystroke content is never recorded.
- Screen time polling adds ~0.1–0.3% CPU overhead on top of baseline usage.
- Linux desktop integration (tray, permissions) is not fully tested across all distros.
- macOS builds require a Mac; cross-compilation from Windows is not supported.

---

### Documentation

| Document | Description |
|----------|-------------|
| [README.md](../README.md) | Quick start and feature overview |
| [MACOS.md](./MACOS.md) | macOS build and permissions guide |
| [cooldown-overview.html](./cooldown-overview.html) | Full product overview (PDF-ready) |
| [cooldown-simple-overview.html](./cooldown-simple-overview.html) | Plain-language overview for everyone |

---

**Less burnout. Smarter breaks. Better code.**
