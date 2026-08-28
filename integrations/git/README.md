# Git integration

Cooldown tracks git commits as **activity** (not errors). Each commit is sent to the local HTTP endpoint on port `9876`.

## Install (per repository)

**Unix / Git Bash:**

```bash
cp integrations/git/post-commit .git/hooks/post-commit
chmod +x .git/hooks/post-commit
```

**Windows (PowerShell helper):**

```powershell
Copy-Item integrations/git/post-commit.ps1 .git/hooks/post-commit.ps1
```

For Git for Windows, the shell hook (`post-commit`) is preferred — copy it to `.git/hooks/post-commit`.

## Payload

```json
{
  "source": "git",
  "plugin": "git",
  "event": "git_commit",
  "message": "feat: add pomodoro timer"
}
```

Commits appear in the daily summary and weekly email report. Cooldown must be running for hooks to deliver events.
