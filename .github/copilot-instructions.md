# Copilot Instructions — Television (tv)

## Project Overview

Television is a fast, portable TUI fuzzy finder written in Rust (edition 2024, MSRV 1.90). The binary is `tv`. It uses **tokio** (multi-thread) for async, **ratatui** + **crossterm** for the terminal UI, and a forked **nucleo** crate (`alexpasmantier/nucleo`, branch `television`) for fuzzy matching.

## Architecture

```
main.rs          CLI parsing (clap), config loading, launches App
  └─ App         Event loop + action dispatch (television/app.rs)
      ├─ Television   Core state: channel, picker, previewer, mode (television/television.rs)
      │   ├─ Channel<P: EntryProcessor>   Spawns source command, feeds lines into Matcher (channels/channel.rs)
      │   ├─ Picker<T>                    List selection state backed by ratatui ListState (picker.rs)
      │   ├─ Previewer                    Async preview pipeline with cache + timeout (previewer/)
      │   └─ RemoteControl / ActionPicker Channel switcher & action menus (channels/)
      ├─ Render task   Dedicated tokio task running at 60 FPS (render.rs → draw.rs → screen/)
      └─ EventLoop     Crossterm event reader → Event<Key> (event.rs)
```

Key data flow: **Events → Actions → Television state mutations → RenderingTasks → Tui draw**. Communication between components uses `tokio::sync::mpsc` unbounded channels. Actions are defined in `television/action.rs` as a flat enum.

### Configuration Layering

Config merges four layers (see `config/layers.rs`): **base config file** → **channel prototype** → **global CLI flags** → **channel CLI flags**. `ConfigLayers::merge()` produces a `MergedConfig` consumed at runtime. Channels are defined as TOML prototypes in `cable/unix/` (or `cable/windows/`), deserialized into `ChannelPrototype` (`channels/prototypes.rs`).

### Channel System (Cable)

Channels are TOML files describing a source command, optional preview, keybindings, and actions. Example: `cable/unix/files.toml`. The `Cable` struct (`cable.rs`) is a `FxHashMap<String, ChannelPrototype>`. `Channel<P>` is generic over `EntryProcessor` (Plain, Ansi, Display variants in `entry_processor.rs`) which controls how source output is pushed into the nucleo `Matcher`.

### Rendering

Rendering runs on a separate tokio task. `App` sends `RenderingTask::Render(Box<Ctx>)` snapshots; the render loop draws via `draw()` in `draw.rs`, which delegates to `screen/` modules (results, preview, input, status_bar, etc.). The render loop caps at 60 FPS and coalesces multiple render instructions.

## Developer Workflow

All commands use the `just` task runner:

| Task               | Command                                       |
| ------------------ | --------------------------------------------- |
| Setup              | `just setup`                                  |
| Run (debug + logs) | `just run`                                    |
| Run (staging/fast) | `just r`                                      |
| Build release      | `just br`                                     |
| Check              | `just check`                                  |
| Lint + format      | `just fix`                                    |
| Tests              | `just test`                                   |
| Fast tests (local) | `just test-fast` (uses `TV_TEST_DELAY_MS=50`) |

Logs go to `$XDG_DATA_HOME/television/television.log` (or platform equivalent). Enable with `RUST_LOG=debug`.

## Code Conventions

- **Rust edition 2024**, max line width **79 chars** (`rustfmt.toml`).
- Clippy **pedantic** is enabled workspace-wide with specific `allow` overrides in `Cargo.toml` `[workspace.lints.clippy]`.
- Hash maps use `rustc_hash::FxHashMap` / `FxHashSet` throughout, not `std::collections::HashMap`.
- The `#[serde(rename_all = "snake_case")]` convention is used on enums that appear in config/TOML files.
- Unit tests live in `#[cfg(test)] mod tests` at the bottom of source files. Integration tests in `tests/` use a `PtyTester` helper (`tests/common/mod.rs`) that spawns `tv` in a pseudo-terminal for end-to-end assertions.
- The `TV_TEST` env var signals test mode to the TUI layer; `TV_TEST_DELAY_MS` controls PTY test timing.
- Platform-specific deps are split with `[target.'cfg(...)'.dependencies]` (e.g., macOS crossterm uses `use-dev-tty`).

## Key Files & Entry Points

- `television/main.rs` — binary entry, CLI + config bootstrap
- `television/app.rs` — `App` struct, main event/action loop
- `television/television.rs` — `Television` core state machine
- `television/channels/channel.rs` — `Channel<P>` generic channel implementation
- `television/channels/prototypes.rs` — `ChannelPrototype`, `CommandSpec`, `ActionSpec` TOML schema
- `television/config/layers.rs` — `ConfigLayers` / `MergedConfig` config merge logic
- `television/action.rs` — `Action` enum (all application actions)
- `television/event.rs` — `Event`, `Key` types, `EventLoop`
- `television/draw.rs` — top-level draw dispatcher
- `television/screen/` — individual UI component renderers
- `cable/unix/*.toml` — built-in channel definitions
- `build.rs` — generates man pages from clap at build time
