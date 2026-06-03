# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.0] - 2026-06-03

### ✨ Added

- **UI language selector (PR #18)**: Choose the interface language in Preferences, independent of the system locale. Changing it prompts for a restart to re-render all strings.
- **New translations**: European Portuguese (pt_PT), Brazilian Portuguese (pt_BR), and British English (en_GB), alongside the existing Russian, German, Spanish, and French. The English entry is split into US (source) and UK variants.

### 🐛 Fixed

- **Missing notification history on first subscription (PR #17)**: New subscriptions now load the server's topic cache (`since=0`) while still marking pre-existing messages as read, so history appears without spamming desktop notifications. A separate `listen_since` cursor keeps cleared notifications from being replayed after reconnect.
- **Window not shown on launcher reactivation (PR #16)**: Launching the app while a background instance is already running now always presents the window; `start-in-background` only suppresses it on first launch.

### 🔧 Changed

- **Packaging (PR #18)**: The custom-LOCALEDIR catalog copy is derived from the installed `.mo` set instead of a hardcoded language list, so future languages are bundled automatically.

## [0.5.4] - 2026-05-10

### 🐛 Fixed

- **Startup crash on missing Secret portal (issue #12)**: The keyring no longer panics when `org.freedesktop.portal.Secret` is unavailable or unusable. It now falls back to the host Secret Service over DBus, and finally to a non-persistent in-memory store, so the app keeps launching.
- **Add account fails with "Password (secret from portal) too short: 0" (issue #14)**: Same fallback path covers the case where the portal returns an unusable master key — account creation retries via Secret Service.
- **Tray icon invisible on dark panels (issue #13)**: Symbolic tray SVGs now use the KDE `ColorScheme-Text` / `ColorScheme-Highlight` idiom and `currentColor`, so the tray icon is themed by the panel on KDE Breeze Dark and other dark themes.
- **Notification replay after Clear (PR #15)**: Clearing notifications now atomically bumps `read_until` and deletes cached messages in a single transaction, so reconnecting no longer replays the server-side topic cache from epoch.
- **Inflated launcher badge counters (PR #15)**: Desktop portal notifications use a stable per-(server, topic) id, so new alerts replace the previous portal notification instead of accumulating shell badge entries.
- **Empty topic clear errored (PR #15)**: `delete_messages` no longer fails when no rows match.

### 🔧 Changed

- **Desktop file `Exec` (PR #15)**: Uses the absolute `@bindir@/ntfyr` path so launchers that drop `~/.local/bin` from `PATH` still start the app.

## [0.5.3] - 2026-03-28

### 🐛 Fixed

- **HTTPS connections to private servers**: Fixed a regression where subscribing to self-hosted ntfy servers via HTTPS would fail with "invalid URL, scheme is not http". Caused by a missing `rustls` feature when updating the `reqwest` dependency from 0.12 to 0.13.

## [0.5.2] - 2026-03-23

### 🐛 Fixed

- **Build**: Removed unused capnproto module from Flatpak manifests (was never a required dependency).

## [0.5.1] - 2026-03-23

### 🐛 Fixed

- **Build**: Fixed Flatpak build failure by updating Cargo.lock and regenerating cargo-sources.json to match declared crate versions.

## [0.5.0] - 2026-03-23

### ✨ New Features

- **Date/Time Format**: Customizable timestamp format for message rows via Preferences (ISO, European, US, time-only, short).

### 🐛 Fixed

- **Window Focus**: Re-activating the app now always brings the window to the foreground (issue #7).
- **Tray Icon**: System tray now shows a full-color icon on KDE dark themes instead of an invisible symbolic icon (issue #9).

### 🔧 Changed

- **Runtime**: Updated Flatpak runtime to GNOME 50.
- **Dependencies**: Updated all Rust crates to latest versions (gtk4 0.11, libadwaita 0.9, ashpd 0.13, rand 0.10, oo7 0.6, rusqlite 0.39, reqwest 0.13, and more).
- **Build**: Build script now uses a shared local Flatpak repo (`~/repo`) to avoid stale build artifacts.

## [0.4.1] - 2026-02-15

### Fixed
- **AppStream**: Fixed metadata validation issues by removing HTML tags from release description.

## [0.4.0] - 2026-02-15

### Added
- **App Lock**: Secure your notifications with an application lock code.
- **Auto-Lock**: Automatically lock the application after a period of inactivity.
- **Privacy Mode**: Hide notification content in system notifications when locked.
- **Secrets Management**: Enhanced security for storing sensitive data using libsecret.

### Changed
- **Settings Redesign**: Reorganized settings into logical sections (General, Appearance, Privacy).
- **Security Check**: Application now verifies security configuration on startup.

## [0.3.0] - 2026-01-25

### Added

- **End-to-End Encryption**: Full support for sending and receiving encrypted messages
  - Automatic decryption of incoming encrypted notifications
  - New "Encrypt Message" toggle in Advanced Message Dialog
  - Encryption keys stored securely in system keyring via libsecret
  - New "Encryption Key" field in Subscription Info dialog

### Changed

- Updated Flatpak manifest with `org.freedesktop.secrets` permission for keyring access

## [0.2.1] - 2026-01-12

### Changed
- Improved metadata validation (summary, description, branding coverage)
- Simplified README and added Flathub/Ko-Fi badges
- Improved build instructions to use `build.sh`

## [0.2.0] - 2026-01-10

### Added

- **Filter Rules**: Added ability to filter notifications based on rules.
- **Filter Dialog**: New dialog to create and manage filter rules.

### Changed

- **Unified Dialogs**: Refactored "Add Server" and "Add Account" dialogs to match the "Add Topic" aesthetic.
- **Server Actions**: Improved server actions menu with `GtkMenuButton` and better styling.
- **UI Polish**: Various visual improvements to dialogs and menus.

### Fixed

- **Muted Icon**: Muted subscriptions now correctly show a muted icon in the topic list.
- **Database Migrations**: Fixed issues with database migrations.


## [0.1.2] - 2026-01-06

### Changed

- **Portal-based Notifications**: Refactored notification system to use XDG Desktop Portal (`ashpd`) instead of direct D-Bus communication with `org.freedesktop.Notifications`. This improves sandboxing and follows Flatpak best practices.

### Removed

- Removed `--talk-name=org.freedesktop.Notifications` D-Bus permission from Flatpak manifests as it's no longer needed with portal-based notifications.

## [0.1.1]

### Fixed

- Fixed flatpak build failure by switching `capnproto` source to official tarball.
- Fixed offline build failure by correctly setting `CARGO_HOME` in `meson.build`.

## [0.1.0] - 2026-01-04

### Added
- **Custom Servers**: Added support for custom ntfy servers in Preferences.
- **Server Grouping**: Subscriptions are now grouped by server in the side panel.
- **Unified Preferences**: Merged default server selection into the main server list.
- **Enhanced About Dialog**: Added credits, links, and legal info.
- **Markdown Support**: Messages now support Markdown rendering.
- **Timezone Conversion**: Message timestamps are converted to local time.
- **Message Sorting**: Added option to sort messages by date.
- **Shortcuts**: Added keyboard shortcuts for Preferences (`Ctrl+,`) and About (`F1`).
- **Autostart**: Implemented reliable background autostart using XDG Portal.
- **Tray Icon**: Enhanced system tray integration with window toggling.
- **Mobile Navigation**: Improved navigation flow on mobile devices.

### Changed
- **UI Improvements**: Polished various UI elements, including tooltips and centered action buttons.
- **Account Dialog**: Refined the "Add Account" dialog and account list.
- **Shortcuts Dialog**: Upgraded to `Adw.ShortcutsDialog` for better UX.
- **Dependencies**: Updated `gettext-rs` and other dependencies for reliable Flatpak builds.

### Fixed
- **Notification Flooding**: Fixed issue where old notifications re-appeared on new subscriptions.
- **Subscription Crash**: Resolved crash when opening "Subscription Info".
- **Libadwaita API**: Fixed compatibility issues with Libadwaita 1.6+.
- **Message Clearing**: Message input now properly clears after sending.
- **Compilation Warnings**: Cleaned up unused code and warnings.

