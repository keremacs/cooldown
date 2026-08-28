# Cooldown git post-commit hook (PowerShell) — records commits as activity.
# Install: Copy-Item integrations/git/post-commit.ps1 .git/hooks/post-commit -Force
#          (Git for Windows runs hooks via sh; use post-commit with bash, or configure manually.)

$msg = (git log -1 --pretty=%s 2>$null)
if ($msg.Length -gt 200) { $msg = $msg.Substring(0, 200) }
$body = @{
    source = "git"
    plugin = "git"
    event  = "git_commit"
    message = $msg
} | ConvertTo-Json -Compress

try {
    Invoke-RestMethod -Uri "http://127.0.0.1:9876/event" -Method Post -ContentType "application/json" -Body $body -TimeoutSec 2 | Out-Null
} catch {
    # Cooldown may not be running — ignore
}
