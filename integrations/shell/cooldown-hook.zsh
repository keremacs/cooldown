cooldown_report() { local ec=$?; [ $ec -ne 0 ] && curl -sf -X POST http://127.0.0.1:9876/event -H 'Content-Type: application/json' -d "{\"source\":\"terminal\",\"exit_code\":$ec}" >/dev/null 2>&1; }
precmd_functions+=(cooldown_report)  # zsh — for bash use: trap 'cooldown_report' DEBUG
