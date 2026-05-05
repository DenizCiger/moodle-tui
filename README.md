# moodle-tui

Terminal UI for Moodle, written in Rust with [ratatui](https://github.com/ratatui/ratatui).
Port of the Bun + Ink [`moodle-tui`](../moodle-tui) reference app.

## Run

```sh
cargo run            # production
cargo run -- --demo  # offline demo data, no network, no disk writes
```

Config & cache live in `~/.config/tui-moodle/` (override via `TUI_MOODLE_CONFIG_DIR`).
The Rust port reads & writes the same JSON layout as the TS version, so existing creds carry over.

## Install (npm wrapper, after first GitHub release)

```sh
npm install -g moodle-tui
moodle
```

## Demo via Docker + ttyd

```sh
docker build -f Dockerfile.demo -t moodle-tui-demo .
docker run --rm -p 7681:7681 moodle-tui-demo
# open http://localhost:7681
```

## Develop

```sh
cargo check
cargo test
cargo build --release
```

## Status

This is the Rust port. UI fidelity is in progress — the spine (login, dashboard list,
course page, finder/modal scaffolds, settings/help) is in place; tree expand/collapse,
detailed assignment modal contents, and the polished overlay lists are still being
ported from the TS source one screen at a time.
