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

## Plugins

`moodle-tui` is a vanilla host app. Optional behavior lives in separate plugin
repositories and is installed into the user config directory.

Installed plugins live in:

```text
~/.config/tui-moodle/plugins/<plugin-id>/
```

Each plugin directory must contain:

- `plugin.json` with id, name, version, entry, permissions, settings schema, and contributions
- an executable entry file, such as `plugin.js`

Open Settings (`?`) to manage plugins:

- Use the right Config pane to install a local plugin folder, reload plugins, enable or disable a plugin, uninstall a plugin, and edit schema-declared plugin settings.
- Use the left Keybinds pane to inspect all core and plugin keybinds. Plugin actions can ship default keys, and conflicts are shown instead of silently stealing a key.
- Local installs are linked into the config plugin directory, so editing the source plugin repo updates the installed plugin during development.

Uninstalling a plugin from the TUI removes only the installed link/copy under
`~/.config/tui-moodle/plugins`; it does not delete the source plugin repository.

The Gemini quiz filler is an example external plugin:

```text
../moodle-tui-quiz-ai-extension
```

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

## Local Moodle quiz harness

The local harness keeps Moodle data in Docker volumes so the seeded quiz remains
available after setup.

```sh
docker compose -f docker-compose.local.yml up -d
docker compose -f docker-compose.local.yml exec moodle php /scripts/local_moodle_quiz_seed.php
cargo run
```

Login in the TUI with:

- URL: `http://localhost:8080`
- Username: `student`
- Password: `studentpass`
- Service: `moodle_mobile_app`

Open course `TUI-QUIZ`, select `TUI supported questions quiz`, then press Enter.
The seeded quiz contains true/false, short answer, and numerical questions that
exercise the in-TUI quiz flow. The Moodle admin account is `admin` / `adminpass`.
If Moodle returns `invaliddatarootpermissions`, run:

```sh
docker compose -f docker-compose.local.yml exec --user root moodle sh -lc "chmod -R 0777 /bitnami/moodledata /bitnami/moodle"
```

## Status

This is the Rust port. UI fidelity is in progress — the spine (login, dashboard list,
course page, finder/modal scaffolds, settings/help) is in place; tree expand/collapse,
detailed assignment modal contents, and the polished overlay lists are still being
ported from the TS source one screen at a time.
