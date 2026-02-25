# tui-moodle

A terminal UI for Moodle built with Bun + Ink.

## Features

- Secure login with username/password
- Cross-platform secure password storage (macOS Keychain, Linux libsecret, Windows DPAPI)
- Courses list view with keyboard navigation
- Cached course fallback when Moodle API is unavailable
- Settings/help modal with shortcut overview (`?`)

## Requirements

- [Bun](https://bun.sh/) (v1.3+ recommended)
- A Moodle instance with web services enabled
- Valid Moodle account credentials

## Install

```bash
bun install
```

## Run

```bash
bun run index.tsx
```

Or through the bin entry:

```bash
bun run bin/moodle.js
```

## Controls

- `?` open/close settings modal
- `q` quit
- `l` logout
- `r` refresh courses
- `↑` / `↓` move selection
- `PageUp` / `PageDown` jump
- `Home` / `End` jump to start/end

## Tests

```bash
bun test
bunx tsc --noEmit
```

## Local Debug Script

Fetch courses using saved credentials:

```bash
bun run scripts/fetch-courses.ts
```

## Config Storage

- Config file: `~/.config/tui-moodle/config.json`
- Cache file: `~/.config/tui-moodle/cache.json`
- Windows fallback secret store: `~/.config/tui-moodle/secrets.json`

## Push Checklist

1. Confirm no `node_modules` is tracked (`git status`).
2. Run `bun test` and `bunx tsc --noEmit`.
3. Commit all files.
4. Add remote and push:

```bash
git remote add origin <your-github-repo-url>
git branch -M main
git push -u origin main
```