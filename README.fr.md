# Ntfyr

**Langues :** [English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md)

Client natif [ntfy.sh](https://ntfy.sh/) pour le bureau GNOME. Développé en Rust avec GTK4 et Libadwaita ; distribué en Flatpak.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## Dernière version : 0.6.1

Points récents (historique complet dans [CHANGELOG.md](CHANGELOG.md)) :

- **Sélecteur de langue de l’interface** — choisissez la langue dans les Préférences (système, anglais US/UK, russe, allemand, espagnol, français, portugais PT/BR) ; un redémarrage applique les traductions dans toute l’application.
- **Historique à l’abonnement** — les nouveaux abonnements chargent le cache du topic serveur sans renvoyer les messages effacés ni saturer les alertes bureau.
- **Mode arrière-plan fiable** — une réactivation depuis le lanceur affiche toujours la fenêtre ; intégration barre système et portails améliorée sous GNOME et KDE.
- **Self-hosted** — serveurs privés HTTPS, identifiants stables des notifications bureau, effacement atomique et replis keyring robustes si le portail Secret est indisponible.
- **0.6.1** — version de maintenance avec dépendances Rust mises à jour (GTK 0.22 / ashpd 0.13 / reqwest 0.13).

Installez la version stable depuis [Flathub](https://flathub.org/en/apps/io.github.tobagin.Ntfyr) ou compilez depuis les sources (ci-dessous).

## Fonctionnalités

### Cœur
- **Intégration bureau native** — GTK4 + Libadwaita, GNOME Platform 50.
- **Notifications push** — abonnez-vous aux topics sur `ntfy.sh` ou serveurs auto-hébergés.
- **Démon en arrière-plan** — backend in-process maintient les connexions SSE et livre les notifications bureau.
- **Historique local** — messages en SQLite pour consultation hors ligne et recherche.
- **Plusieurs serveurs** — abonnements groupés par serveur ; masquer ou afficher l’entrée `ntfy.sh` par défaut.

### Notifications et contenu
- **Chiffrement de bout en bout** — envoyer et recevoir des messages chiffrés.
- **Pièces jointes** — afficher des images et télécharger des fichiers dans l’app.
- **Boutons d’action** — ouvrir des liens et exécuter des actions HTTP depuis les notifications.
- **Filtres** — règles de filtrage par abonnement.

### Confidentialité et sécurité
- **Verrouillage de l’app** — PIN/biométrie optionnel au démarrage (via keyring système).
- **Self-hosted** — connexion à des instances ntfy privées en HTTPS.
- **Flatpak sandboxé** — permissions strictes ; pas de télémétrie.
- **Données locales uniquement** — réglages et historique restent sur votre machine.

### Expérience bureau
- **Barre système** — accès rapide et indicateur de non-lus (icône symbolique sur panneaux sombres).
- **Raccourcis clavier** — `Ctrl+N` nouveau topic, `Ctrl+F` recherche, `Ctrl+,` préférences, `F1` aide.
- **Préférences** — format date/heure, démarrage automatique, lancement en arrière-plan, état de la fenêtre.
- **Mode sombre** — suit le thème système.

### Traductions

Interface disponible en anglais, russe, allemand, espagnol, français et portugais (variantes européenne et brésilienne dans l’app). D’autres langues via gettext — voir [CONTRIBUTING.md](CONTRIBUTING.md).

## Compiler depuis les sources

Nécessite Flatpak, `flatpak-builder` et le SDK/runtime GNOME 50 (installés automatiquement par le script depuis Flathub).

```bash
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Production (io.github.tobagin.Ntfyr)
./build.sh

# Développement (io.github.tobagin.Ntfyr.Devel, profil debug, hook pre-commit)
./build.sh --dev
```

Chaque compilation s’installe dans le dépôt Flatpak utilisateur `~/repo`. Lancer :

```bash
flatpak run io.github.tobagin.Ntfyr          # production
flatpak run io.github.tobagin.Ntfyr.Devel    # development
```

Itération rapide sans Flatpak (paquets hôte `gtk4-devel`, `libadwaita-devel` requis) :

```bash
cargo check   # vérification de types
cargo clippy  # lint
cargo run     # exécuter depuis les sources
```

Architecture : [CLAUDE.md](CLAUDE.md) ; workflow de développement : [CONTRIBUTING.md](CONTRIBUTING.md).

## Utilisation

Lancez Ntfyr depuis le menu applications ou exécutez :

```bash
flatpak run io.github.tobagin.Ntfyr
```

1. Cliquez sur **+** pour ajouter un abonnement.
2. Saisissez le nom du topic (p. ex. `alerts`).
3. Choisissez éventuellement un serveur personnalisé ou ajoutez-en un dans les Préférences.

Envoyer une notification de test :

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Raccourcis clavier

| Raccourci | Action |
|-----------|--------|
| `Ctrl+N` | S’abonner à un nouveau topic |
| `Ctrl+F` | Rechercher dans les notifications |
| `Ctrl+,` | Ouvrir les Préférences |
| `Ctrl+Q` | Quitter |
| `F1` | Afficher les raccourcis |

## Contribuer

Les contributions sont les bienvenues ! Lisez [CONTRIBUTING.md](CONTRIBUTING.md) avant d’ouvrir une pull request.

- **Bugs :** [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- **Questions et idées :** [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## Licence

Ntfyr est sous [GPL-3.0-or-later](LICENSE).

## Remerciements

- [ntfy.sh](https://ntfy.sh/) — la plateforme de notifications.
- [GNOME](https://www.gnome.org/) — GTK4 et Libadwaita.
- [Rust](https://www.rust-lang.org/) — implémentation de l’application et du démon.

## Captures d’écran

| Fenêtre principale | Topics | Préférences |
|--------------------|--------|-------------|
| ![Main window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |
