function global:cooldown_report { if ($LASTEXITCODE -ne 0) { curl.exe -sf -X POST http://127.0.0.1:9876/event -H 'Content-Type: application/json' -d "{`"source`":`"powershell`",`"plugin`":`"powershell`",`"exit_code`":$LASTEXITCODE}" 2>$null | Out-Null } }
function prompt { cooldown_report; "PS $($PWD)> " }
