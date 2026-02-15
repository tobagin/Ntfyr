# Ntfyr

A native ntfy.sh client for the GNOME Desktop.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## 🎉 Version 0.4.0 - App Locking & Privacy

**Ntfyr 0.4.0** adds App Locking and enhanced privacy controls.

### ✨ Key Features

- **🚀 Native and Fast**: Built with Rust and GTK4 for a smooth, native experience.
- **📨 Push Notifications**: Instant notifications from ntfy.sh or self-hosted servers.
- **🔐 End-to-End Encryption**: Send and receive encrypted messages securely.
- **🛡️ App Locking**: Protect your messages with a PIN code.
- **🔄 Background Service**: Runs reliably in the background via system portals.
- **📂 Attachments**: View images and download files directly within the app.
- **🧩 Multiple Servers**: Group subscriptions by server for better organizational.
- **🛡️ Privacy Focused**: Full support for self-hosted instances.

### 🆕 What's New in 0.4.0

- **App Lock**: Secure access with a PIN code.
- **Auto-Lock**: Lock app after inactivity.
- **Privacy & Security**: Hide notification content and verify security settings.
- **Secrets Management**: Robust secret storage using libsecret.

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

## Building from Source

```bash
# Clone the repository
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Build and install development version
./build.sh --dev
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

| Main Window | Topics View | Preferences |
|-------------|-------------|-------------|
| ![Main Window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |

---

**Ntfyr** - A native ntfy.sh client for the GNOME Desktop.

