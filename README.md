# Hatchmark

A Windows desktop app that tracks daily actions (water, vape, mint sugar, and
anything else you care about) via global hotkeys bound to a mini 6-key
keyboard. A tiny always-running daemon listens for `F13`–`F24` and writes each
press into SQLite; an on-demand Apple Liquid Glass UI visualizes the data and
manages bindings.

## Architecture

- **`hatchmark-core`** — shared Rust crate: SQLite schema, models, repositories,
  analytics queries, IPC wire types.
- **`hatchmark-daemon`** — Rust binary, always running, ~10–15 MB RAM. Tray icon,
  global hotkey registration (`global-hotkey`), autostart via
  `HKCU\...\Run`, optional toasts, localhost IPC server.
- **`hatchmark-ui`** — Tauri 2 app (Rust backend + Svelte 5 + Tailwind + uPlot).
  Only launched when you double-click the tray icon; fully exits on close, so
  nothing stays in RAM when it's not open. Uses Windows 11 DWM Mica backdrop
  for a real translucent glass window.

Both binaries share a single SQLite DB at
`%APPDATA%\Hatchmark\hatchmark.db`. The UI connects to the daemon's
IPC socket to receive live `increment` / `layer_changed` / `binding_conflict`
events.

## Prerequisites

- Windows 11 (22H2+ for Mica; 10 will still run but without the backdrop).
- Rust 1.78+ via [rustup](https://rustup.rs/).
- Microsoft Visual Studio Build Tools with the "Desktop development with C++"
  workload (for the MSVC linker).
- Node.js 20+ and npm (already required for the UI build).
- Tauri CLI: `cargo install tauri-cli --version "^2"` (only needed for
  `cargo tauri dev`; release builds just use `cargo build`).
- Microsoft Edge WebView2 Runtime (pre-installed on Windows 11).

## Build

```bash
# Rust side
cargo build --release --workspace

# Frontend (the tauri build runs this automatically, but for dev:)
npm --prefix crates/hatchmark-ui/web install
npm --prefix crates/hatchmark-ui/web run build
```

Artifacts land at:
- `target/release/hatchmark-daemon.exe`
- `target/release/hatchmark-ui.exe`

Ship both in the same directory — the daemon locates the UI binary
side-by-side.

## Run

```bash
# Developer dev-mode for the UI:
cd crates/hatchmark-ui
cargo tauri dev

# Release mode, side-by-side with the daemon:
./target/release/hatchmark-daemon.exe
# Double-click the tray icon to launch the UI.
```

## First-run setup

1. Run `hatchmark-daemon.exe`. A tray icon appears.
2. Double-click the tray icon → UI opens.
3. Go to **Settings → Channels** → click "Add channel". Name it (e.g.
   "Water"), pick a color, optionally set `goal` / `limit`.
4. Go to **Settings → Bindings** → click "Bind key" → press F13 on your mini
   keyboard. The binding defaults to incrementing the first channel — you can
   change it in the dropdown.
5. Close the UI. Focus any other app. Press F13. The count increments.
6. Re-open the UI from the tray to see the updated **Today** view.

## Layers

Layers let you swap the meaning of your physical keys (like a keyboard layer).
In **Settings → Layers**, add a second layer. In **Settings → Bindings**, on
the base layer pick a key (say F18) and set it to **Switch → second layer**.
After pressing F18, the remaining five keys now map to the second layer's
bindings until you switch back.

## Data

- **Location:** `%APPDATA%\Hatchmark\`
  - `hatchmark.db` — SQLite (WAL mode).
  - `daemon-state.json` — current IPC port (UI reads this to connect).
  - `log.txt` — daemon log.
- **Backup:** close the daemon from its tray menu, copy `hatchmark.db` somewhere
  safe.
- **Export:** Settings → Data → "Export CSV…" writes one row per event.

## Tests

```bash
cargo test --workspace
```

Integration test `crates/hatchmark-core/tests/analytics_roundtrip.rs` exercises
the end-to-end key → binding → event → analytics path without touching
Windows APIs, so it runs cross-platform.

The manual checklist at
[docs/superpowers/specs/smoke-test-checklist.md](docs/superpowers/specs/smoke-test-checklist.md)
covers Windows-only behavior (tray, Mica, autostart, IPC).

## Uninstall

1. Close the daemon from its tray menu (this also disables autostart if you
   toggled it off first — otherwise remove the run-key manually below).
2. Delete `%APPDATA%\Hatchmark\`.
3. If autostart was left on, remove
   `HKCU\Software\Microsoft\Windows\CurrentVersion\Run\Hatchmark`
   from the registry (Settings → Autostart toggle does this for you when
   enabled/disabled).

## Project layout

```
Hatchmark/
├── Cargo.toml                     # workspace
├── crates/
│   ├── hatchmark-core/              # types, DB, analytics, IPC
│   ├── hatchmark-daemon/            # tray + hotkeys + IPC server
│   └── hatchmark-ui/                # Tauri app
│       ├── src/                   # Tauri Rust backend
│       ├── capabilities/          # Tauri 2 permissions
│       └── web/                   # SvelteKit + Tailwind frontend
└── docs/superpowers/
    ├── specs/                     # design spec + smoke-test checklist
    └── plans/                     # implementation plans 1/2/3
```

## Implementation plans

The project was scoped as three plans, each shipping testable software:

1. [Plan 1 — Daemon & Data Layer](docs/superpowers/plans/2026-04-20-plan-1-daemon-and-data-layer.md)
2. [Plan 2 — UI Foundation](docs/superpowers/plans/2026-04-20-plan-2-ui-foundation.md)
3. [Plan 3 — Analytics & Glass Polish](docs/superpowers/plans/2026-04-20-plan-3-analytics-and-glass-polish.md)

Design spec:
[docs/superpowers/specs/2026-04-20-hatchmark-design.md](docs/superpowers/specs/2026-04-20-hatchmark-design.md).
