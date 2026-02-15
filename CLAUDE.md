# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ntfyr is a native GNOME desktop client for [ntfy.sh](https://ntfy.sh/) push notifications. Built with Rust, GTK4, and Libadwaita. Distributed as a Flatpak. License: GPL-3.0-or-later.

## Build & Development Commands

### Flatpak Build (full build, required for UI/resource changes)
```bash
# Development build
./build.sh --dev
flatpak run io.github.tobagin.Ntfyr.Devel

# Production build
./build.sh
```

### Cargo (quick iteration, requires gtk4-devel and libadwaita-devel on host)
```bash
cargo check          # Type check
cargo clippy         # Lint
cargo fmt --all      # Format
cargo run            # Build and run
```

### Pre-commit Hook
Installed automatically in development builds. Runs `cargo fmt --all -- --check`. Fix with `cargo fmt --all`.

## Architecture

### Two-Crate Workspace
- **`ntfyr`** (root) — GTK4/Libadwaita frontend application
- **`ntfy-daemon`** (`ntfy-daemon/`) — Background daemon handling network, database, and notification delivery

Communication between frontend and daemon uses async channels (`async-channel`). The daemon is spawned in-process via `NtfyrApplication::ensure_rpc_running()`.

### Frontend (src/)
- `main.rs` — Entry point, i18n setup, GResource loading
- `application.rs` — `NtfyrApplication` (extends `AdwApplication`), manages G-Actions, window lifecycle, tray, autostart portal
- `subscription.rs` — `Subscription` GObject wrapping daemon's `SubscriptionHandle`
- `secrets.rs` — App lock password storage via system keyring (`oo7`)
- `tray.rs` — System tray via `ksni` (KDE StatusNotifier)
- `config.rs.in` — Template for build-time constants (`APP_ID`, `VERSION`, etc.), processed by Meson into `config.rs`
- `widgets/` — All UI components, each paired with a Blueprint `.blp` file in `data/resources/ui/`

### Daemon (ntfy-daemon/src/)
- `ntfy.rs` — `NtfyHandle` RPC interface; daemon startup and subscription management
- `listener.rs` — SSE event streaming from ntfy servers with retry logic
- `subscription.rs` — Per-subscription lifecycle: listener task, filtering, read tracking
- `http_client.rs` — HTTP requests to ntfy servers
- `credentials.rs` / `keys.rs` — Auth and encryption key storage via system keyring
- `models.rs` — Core data structures (`ReceivedMessage`, `OutgoingMessage`, `FilterRule`, etc.)
- `message_repo/` — SQLite persistence layer

### Build System (Meson + Cargo)
Meson orchestrates the full build: Blueprint compilation → GResource bundling → Cargo build. Two profiles controlled by `meson_options.txt`:
- `default` — Release mode with LTO
- `development` — Debug mode, installs git hooks

### UI Layer
Blueprint `.blp` files in `data/resources/ui/` are compiled to GtkBuilder XML by `blueprint-compiler`, then bundled into a GResource. Each widget in `src/widgets/` uses `#[template]` to load its corresponding UI file.

### Settings
GSettings schema in `data/io.github.tobagin.Ntfyr.gschema.xml.in`. Access via `gio::Settings::new(APP_ID)`. Bind to widgets with `settings.bind("key", &widget, "property")`.

## Adding a New Widget

1. Create `data/resources/ui/my_widget.blp`
2. Create `src/widgets/my_widget.rs` (GObject subclass with `#[template]`)
3. Add `mod my_widget;` and `pub use` to `src/widgets/mod.rs`
4. Add the `.blp` file to the `blueprints` target in `data/resources/meson.build`
5. Add the compiled `.ui` file to `data/resources/resources.gresource.xml.in`

## Code Style

- `cargo fmt` (rustfmt) — enforced by pre-commit hook
- `cargo clippy` — used for linting
- Blueprint UI filenames: `kebab-case` or `snake_case` (match existing)
- Translations: wrap user-visible strings with `gettext("...")`
- Rust edition 2024
