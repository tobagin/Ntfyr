# Ntfyr

A native ntfy.sh client for the GNOME Desktop.

![Ntfyr Application](data/screenshots/1.png)

## 🎉 Version 0.1.0 - Initial Release

**Ntfyr 0.1.0** brings the power of ntfy.sh to your Linux desktop with a seamless, native experience.

### ✨ Key Features

- **🚀 Native Rust Backend**: Built with Rust, GTK4, and Libadwaita for speed and stability
- **📨 Push Notifications**: Instant notifications from ntfy.sh or self-hosted servers
- **🔄 Background Service**: Runs reliably in the background via system portals
- **📂 Attachments**: View images and download files directly within the app
- **🧩 Multiple Servers**: Group subscriptions by server for better organization
- **🛡️ Privacy Focused**: Full support for self-hosted instances

### 🆕 What's New in 0.1.0

- **Server Grouping**: Subscriptions are visually grouped by their server.
- **Unified Preferences**: Manage accounts and servers in a single, cohesive view.
- **System Tray**: Improved tray integration with window toggling.
- **Markdown**: Rich text rendering for notification messages.

For detailed release notes and version history, see [CHANGELOG.md](CHANGELOG.md).

## Features

### Core Features
- **Native Desktop Integration**: Uses GTK4 and Libadwaita for a perfect GNOME fit.
- **Unified Server Management**: Manage subscriptions from `ntfy.sh` and custom servers in one list.
- **Persistent Connection**: Daemonized backend ensures you never miss a notification.

### User Experience
- **Action Buttons**: Interact with notifications (Open Link, HTTP actions).
- **Shortcuts**: Keyboard shortcuts for quick navigation (`Ctrl+,`, `F1`, `Ctrl+N`).
- **System Tray**: Quick access and unread status indication.
- **Dark Mode**: Fully supports system-wide dark theme preference.

### Privacy & Customization
- **Self-Hosted Support**: Connect to your own ntfy server instances.
- **Local History**: Notifications are cached locally for offline viewing.
- **No Telemetry**: Your data stays on your machine.

## Installation

### Flatpak (Recommended)

#### From Flathub
```bash
flatpak install flathub io.github.tobagin.Ntfyr
```

#### Development Version
```bash
# Clone the repository
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Build and install development version
flatpak-builder --user --install --force-clean build packaging/io.github.tobagin.Ntfyr.Devel.yml
flatpak run io.github.tobagin.Ntfyr.Devel
```

## Usage

### Basic Usage

Launch Ntfyr from your applications menu or run:
```bash
flatpak run io.github.tobagin.Ntfyr
```

1.  Click the **+** button to add a subscription.
2.  Enter the topic component (e.g., `mytopic`).
3.  (Optional) Select or add a custom server.

### Sending Notifications

You can test Ntfyr using `curl`:

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Keyboard Shortcuts

- `Ctrl+,` - Open Preferences
- `Ctrl+Q` - Quit Application
- `F1` - Show Shortcuts Help
- `Ctrl+N` - Subscribe to new topic
- `Ctrl+F` - Search notifications

## Architecture

Ntfyr is built using modern GNOME technologies:

- **Rust**: For memory safety and performance.
- **GTK4 / Libadwaita**: For the user interface.
- **ntfy-daemon**: Custom Rust daemon for handling notification streams.
- **SQLite**: Local storage for notification history.

## Privacy & Security

Ntfyr is designated to respect your privacy:

- **Sandboxed**: Distributed as a Flatpak with strict permissions.
- **Local Data**: All history and configuration is stored locally.
- **Open Source**: Code is fully available for audit.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

- Reporting Bugs: [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- Discussions: [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## License

Ntfyr is licensed under the [GPL-3.0-or-later](LICENSE).

## Acknowledgments

- **ntfy.sh**: For the amazing notification platform.
- **GNOME**: For the GTK toolkit.
- **Rust**: For the language.

## Screenshots

| Main Window | Subscription View |
|-------------|-------------------|
| ![Main Window](data/screenshots/1.png) | ![Subscription](data/screenshots/2.png) |

---

**Ntfyr** - A native ntfy.sh client for the GNOME Desktop.

