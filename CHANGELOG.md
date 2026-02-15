# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

