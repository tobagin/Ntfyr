# Ntfyr

**Sprachen:** [English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md)

Ein nativer [ntfy.sh](https://ntfy.sh/)-Client für den GNOME-Desktop. Entwickelt mit Rust, GTK4 und Libadwaita; als Flatpak verfügbar.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## Aktuelles Release: 0.6.1

Auszug der neuesten Änderungen (vollständige Historie in [CHANGELOG.md](CHANGELOG.md)):

- **Sprachauswahl in der Oberfläche** — Sprache in den Einstellungen wählen (System, Englisch US/UK, Russisch, Deutsch, Spanisch, Französisch, Portugiesisch PT/BR); nach Neustart gelten Übersetzungen app-weit.
- **Benachrichtigungsverlauf bei neuer Subscription** — neue Abonnements laden den Server-Topic-Cache, ohne gelöschte Nachrichten erneut zu liefern oder Desktop-Benachrichtigungen zu spamen.
- **Zuverlässiger Hintergrundbetrieb** — erneutes Starten aus dem Launcher zeigt immer das Fenster; verbesserte Tray- und Portal-Integration unter GNOME und KDE.
- **Self-Hosting** — HTTPS für private Server, stabile Desktop-Benachrichtigungs-IDs, atomisches Leeren und robuste Keyring-Fallbacks, wenn das Secret-Portal nicht verfügbar ist.
- **0.6.1** — Wartungsrelease mit aktualisierten Rust-Abhängigkeiten (GTK 0.22 / ashpd 0.13 / reqwest 0.13).

Installieren Sie die stabile Version über [Flathub](https://flathub.org/en/apps/io.github.tobagin.Ntfyr) oder bauen Sie aus den Quellen (unten).

## Funktionen

### Kern
- **Native Desktop-Integration** — GTK4 + Libadwaita, GNOME Platform 50.
- **Push-Benachrichtigungen** — Topics auf `ntfy.sh` oder selbst gehosteten Servern abonnieren.
- **Hintergrund-Daemon** — In-Process-Backend hält SSE-Verbindungen und liefert Desktop-Benachrichtigungen.
- **Lokaler Verlauf** — Nachrichten in SQLite für Offline-Ansicht und Suche.
- **Mehrere Server** — Abonnements nach Server gruppiert; Standard-`ntfy.sh`-Eintrag ein- oder ausblendbar.

### Benachrichtigungen & Inhalte
- **Ende-zu-Ende-Verschlüsselung** — verschlüsselte Nachrichten senden und empfangen.
- **Anhänge** — Bilder anzeigen und Dateien in der App herunterladen.
- **Aktionsschaltflächen** — Links öffnen und HTTP-Aktionen aus Benachrichtigungen ausführen.
- **Filter** — Filterregeln pro Abonnement.

### Datenschutz & Sicherheit
- **App-Sperre** — optional PIN/Biometrie beim Start (über System-Keyring).
- **Self-Hosting** — private ntfy-Instanzen per HTTPS.
- **Flatpak-Sandbox** — strenge Berechtigungen; keine Telemetrie.
- **Nur lokale Daten** — Einstellungen und Verlauf bleiben auf Ihrem Rechner.

### Desktop-UX
- **System-Tray** — Schnellzugriff und Ungelesen-Anzeige (symbolisches Icon auf dunklen Panels).
- **Tastenkürzel** — `Ctrl+N` neues Topic, `Ctrl+F` Suche, `Ctrl+,` Einstellungen, `F1` Hilfe.
- **Einstellungen** — Datums-/Zeitformat, Autostart, Hintergrundstart, Fensterzustand.
- **Dark Mode** — folgt dem Systemthema.

### Übersetzungen

Oberfläche verfügbar auf Englisch, Russisch, Deutsch, Spanisch, Französisch und Portugiesisch (europäische und brasilianische Varianten in der App). Weitere Sprachen über gettext — siehe [CONTRIBUTING.md](CONTRIBUTING.md).

## Aus Quellen bauen

Erfordert Flatpak, `flatpak-builder` und GNOME-50-SDK/Runtime (wird vom Build-Skript automatisch von Flathub installiert).

```bash
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Production (io.github.tobagin.Ntfyr)
./build.sh

# Development (io.github.tobagin.Ntfyr.Devel, Debug-Profil, Pre-Commit-Hook)
./build.sh --dev
```

Jeder Build wird im Benutzer-Flatpak-Repo `~/repo` installiert. Start:

```bash
flatpak run io.github.tobagin.Ntfyr          # production
flatpak run io.github.tobagin.Ntfyr.Devel    # development
```

Schnellere Iteration ohne Flatpak (Host-Pakete `gtk4-devel`, `libadwaita-devel` erforderlich):

```bash
cargo check   # Typprüfung
cargo clippy  # Lint
cargo run     # aus Quellen starten
```

Architektur: [CLAUDE.md](CLAUDE.md), Entwicklungsworkflow: [CONTRIBUTING.md](CONTRIBUTING.md).

## Verwendung

Ntfyr aus dem Anwendungsmenü starten oder:

```bash
flatpak run io.github.tobagin.Ntfyr
```

1. **+** klicken, um ein Abonnement hinzuzufügen.
2. Topic-Namen eingeben (z. B. `alerts`).
3. Optional einen eigenen Server wählen oder in den Einstellungen hinzufügen.

Testbenachrichtigung senden:

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Tastenkürzel

| Kürzel | Aktion |
|--------|--------|
| `Ctrl+N` | Neues Topic abonnieren |
| `Ctrl+F` | Benachrichtigungen durchsuchen |
| `Ctrl+,` | Einstellungen öffnen |
| `Ctrl+Q` | Beenden |
| `F1` | Tastenkürzel anzeigen |

## Mitwirken

Beiträge sind willkommen! Bitte [CONTRIBUTING.md](CONTRIBUTING.md) lesen, bevor Sie einen Pull Request öffnen.

- **Fehler:** [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- **Fragen & Ideen:** [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## Lizenz

Ntfyr steht unter [GPL-3.0-or-later](LICENSE).

## Danksagungen

- [ntfy.sh](https://ntfy.sh/) — die Benachrichtigungsplattform.
- [GNOME](https://www.gnome.org/) — GTK4 und Libadwaita.
- [Rust](https://www.rust-lang.org/) — Implementierung von App und Daemon.

## Screenshots

| Hauptfenster | Topics | Einstellungen |
|--------------|--------|---------------|
| ![Main window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |
