# Cooldown v0.2

Local-first cognitive load and burnout tracking for developers.

## New in v0.2

| Feature | Location |
|---------|----------|
| Weekly / monthly trend reports | **Trends** tab |
| Deep work score | **Insights** tab + Cognitive Map |
| Error type breakdown (build/lint/test/…) | **Insights** tab |
| Graduated alerts (60 hint / 75 warning / 90 critical) | Toast + popup |
| Contextual break suggestions | Notification popup |
| Focus mode (25/50 min) | **Settings** tab |
| SQLite persistence + retention policy | Backend + Settings |
| Sensitive window filtering | Backend (auto) |
| System tray + colored icon | Tray (green/yellow/red) |
| Mini widget window | Tray → Toggle Widget |
| Dark / light theme | Header + Settings |
| Micro-journal archive | **Journal** tab |
| Personal baseline (14-day) | **Insights** tab |
| Anomaly detection | **Insights** tab |
| Proactive break suggestions | **Insights** tab |
| Plugin registry (vscode/terminal/powershell) | Settings |
| Benchmark mode (memory/CPU/uptime) | Settings |
| Screen lock detection | Backend (Windows + macOS) |
| PowerShell hook | `integrations/shell/cooldown-hook.ps1` |

## Quick Start

**Windows / Linux / macOS**

```bash
npm install
npm run icons      # generate PNG, ICO, ICNS (required for macOS build)
npm run tauri dev
```

### macOS

See **[docs/MACOS.md](docs/MACOS.md)** for full setup: Xcode CLI tools, Accessibility + Input Monitoring permissions, Zsh hook, and `.dmg` build.

```bash
npm run tauri build   # → src-tauri/target/release/bundle/macos/Cooldown.app
```

## Shell Hooks

**Zsh (macOS / Linux)** — add to `~/.zshrc` (see `integrations/shell/cooldown-hook.zsh`):

```bash
cooldown_report() { local ec=$?; [ $ec -ne 0 ] && curl -sf -X POST http://127.0.0.1:9876/event -H 'Content-Type: application/json' -d "{\"source\":\"terminal\",\"exit_code\":$ec}" >/dev/null 2>&1; }
precmd_functions+=(cooldown_report)
```

**PowerShell (Windows)** — add to `$PROFILE`:

```powershell
function global:cooldown_report { if ($LASTEXITCODE -ne 0) { curl.exe -sf -X POST http://127.0.0.1:9876/event -H 'Content-Type: application/json' -d "{`"source`":`"powershell`",`"plugin`":`"powershell`",`"exit_code`":$LASTEXITCODE}" 2>$null | Out-Null } }
function prompt { cooldown_report; "PS $($PWD)> " }
```

## Alert Levels

- **≥ 60** — subtle hint toast (suppressed in focus mode)
- **≥ 75** — break popup with suggestion + micro-journal
- **≥ 90** — critical mandatory break (cannot snooze in UI)

Focus mode suppresses alerts below 90.
