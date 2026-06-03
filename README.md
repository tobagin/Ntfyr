# Ntfyr

**Languages:** [English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md)

A native [ntfy.sh](https://ntfy.sh/) client for the GNOME desktop. Built with Rust, GTK4, and Libadwaita; distributed as a Flatpak.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## Latest release: 0.6.1

Recent highlights (see [CHANGELOG.md](CHANGELOG.md) for the full history):

- **UI language selector** — choose the interface language in Preferences (system, English US/UK, Russian, German, Spanish, French, Portuguese PT/BR); restart applies translations app-wide.
- **Notification history on subscribe** — new subscriptions load the server topic cache without replaying cleared messages or spamming desktop alerts.
- **Reliable background mode** — launcher reactivation always shows the window; tray and portal integration improved across GNOME and KDE.
- **Self-hosted friendly** — HTTPS private servers, stable desktop notification IDs, atomic clear, and robust keyring fallbacks when the Secret portal is unavailable.
- **0.6.1** — dependency maintenance release (updated Rust crates for GTK 0.22 / ashpd 0.13 / reqwest 0.13 stack).

Install the latest stable build from [Flathub](https://flathub.org/en/apps/io.github.tobagin.Ntfyr) or build from source below.

## Features

### Core
- **Native desktop integration** — GTK4 + Libadwaita, GNOME Platform 50.
- **Push notifications** — subscribe to topics on `ntfy.sh` or self-hosted servers.
- **Background daemon** — in-process backend keeps SSE connections alive and delivers desktop notifications.
- **Local history** — messages cached in SQLite for offline browsing and search.
- **Multiple servers** — group subscriptions by server; hide or show the default `ntfy.sh` entry.

### Notifications & content
- **End-to-end encryption** — send and receive encrypted messages.
- **Attachments** — view images and download files in-app.
- **Action buttons** — open links and run HTTP actions from notifications.
- **Filters** — per-subscription filter rules.

### Privacy & security
- **App lock** — optional PIN/biometric lock on startup (via system keyring).
- **Self-hosted support** — connect to private ntfy instances over HTTPS.
- **Sandboxed Flatpak** — strict permissions; no telemetry.
- **Local data only** — settings and history stay on your machine.

### Desktop UX
- **System tray** — quick access and unread indication (themed symbolic icon on dark panels).
- **Keyboard shortcuts** — `Ctrl+N` new topic, `Ctrl+F` search, `Ctrl+,` preferences, `F1` help.
- **Preferences** — customizable date/time format, autostart, background launch, window state.
- **Dark mode** — follows the system theme.

### Translations

Interface available in English, Russian, German, Spanish, French, and Portuguese (European and Brazilian variants in the app). More languages can be added via gettext — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Building from source

Requires Flatpak, `flatpak-builder`, and the GNOME 50 SDK/runtime (installed automatically by the build script from Flathub).

```bash
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Production build (io.github.tobagin.Ntfyr)
./build.sh

# Development build (io.github.tobagin.Ntfyr.Devel, debug profile, pre-commit hook)
./build.sh --dev
```

Each build installs into the user Flatpak repo at `~/repo` and registers the app locally. Run:

```bash
flatpak run io.github.tobagin.Ntfyr          # production
flatpak run io.github.tobagin.Ntfyr.Devel    # development
```

For faster iteration without Flatpak (host packages `gtk4-devel`, `libadwaita-devel` required):

```bash
cargo check   # type check
cargo clippy  # lint
cargo run     # run from source
```

See [CLAUDE.md](CLAUDE.md) for architecture notes and [CONTRIBUTING.md](CONTRIBUTING.md) for the full development workflow.

## Usage

Launch Ntfyr from the applications menu or run:

```bash
flatpak run io.github.tobagin.Ntfyr
```

1. Click **+** to add a subscription.
2. Enter the topic name (e.g. `alerts`).
3. Optionally pick a custom server or add one in Preferences.

Send a test notification:

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+N` | Subscribe to new topic |
| `Ctrl+F` | Search notifications |
| `Ctrl+,` | Open Preferences |
| `Ctrl+Q` | Quit |
| `F1` | Show keyboard shortcuts |

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

- **Bug reports:** [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- **Questions & ideas:** [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## License

Ntfyr is licensed under [GPL-3.0-or-later](LICENSE).

## Acknowledgments

- [ntfy.sh](https://ntfy.sh/) — the notification platform this client talks to.
- [GNOME](https://www.gnome.org/) — GTK4 and Libadwaita.
- [Rust](https://www.rust-lang.org/) — application and daemon implementation.

## Screenshots

| Main window | Topics | Preferences |
|-------------|--------|-------------|
| ![Main window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |
