# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

