# Ntfyr

**Языки:** [English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md)

Нативный клиент [ntfy.sh](https://ntfy.sh/) для рабочего стола GNOME. Написан на Rust с GTK4 и Libadwaita; распространяется как Flatpak.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## Последний релиз: 0.6.1

Кратко о недавних изменениях (полная история — в [CHANGELOG.md](CHANGELOG.md)):

- **Выбор языка интерфейса** — в настройках можно выбрать язык (системный, English US/UK, русский, немецкий, испанский, французский, португальский PT/BR); после перезапуска переводы применяются ко всему приложению.
- **История при подписке** — новые подписки загружают кэш темы с сервера без повторной доставки очищенных сообщений и лишних desktop-уведомлений.
- **Надёжный фоновый режим** — повторный запуск из меню всегда показывает окно; улучшена интеграция с треем и порталами в GNOME и KDE.
- **Self-hosted** — HTTPS на частных серверах, стабильные ID desktop-уведомлений, атомарная очистка и устойчивые fallback'и keyring, если Secret portal недоступен.
- **0.6.1** — технический релиз с обновлением Rust-зависимостей (GTK 0.22 / ashpd 0.13 / reqwest 0.13).

Установите стабильную сборку с [Flathub](https://flathub.org/en/apps/io.github.tobagin.Ntfyr) или соберите из исходников (ниже).

## Возможности

### Основное
- **Нативная интеграция** — GTK4 + Libadwaita, GNOME Platform 50.
- **Push-уведомления** — подписки на темы `ntfy.sh` или собственных серверов.
- **Фоновый демон** — встроенный backend держит SSE-соединения и доставляет desktop-уведомления.
- **Локальная история** — сообщения в SQLite для просмотра офлайн и поиска.
- **Несколько серверов** — группировка подписок по серверу; можно скрыть или показать `ntfy.sh` по умолчанию.

### Уведомления и контент
- **Сквозное шифрование** — отправка и приём зашифрованных сообщений.
- **Вложения** — просмотр изображений и скачивание файлов в приложении.
- **Кнопки действий** — открытие ссылок и HTTP-действия из уведомлений.
- **Фильтры** — правила фильтрации для каждой подписки.

### Конфиденциальность и безопасность
- **Блокировка приложения** — опциональный PIN/биометрия при запуске (через системный keyring).
- **Self-hosted** — подключение к частным экземплярам ntfy по HTTPS.
- **Песочница Flatpak** — строгие разрешения; телеметрии нет.
- **Только локальные данные** — настройки и история остаются на вашем компьютере.

### Интерфейс
- **Системный трей** — быстрый доступ и индикация непрочитанного (символическая иконка на тёмных панелях).
- **Горячие клавиши** — `Ctrl+N` новая тема, `Ctrl+F` поиск, `Ctrl+,` настройки, `F1` справка.
- **Настройки** — формат даты/времени, автозапуск, запуск в фоне, состояние окна.
- **Тёмная тема** — следует системной теме.

### Переводы

Интерфейс доступен на английском, русском, немецком, испанском, французском и португальском (в приложении — европейский и бразильский варианты). Новые языки добавляются через gettext — см. [CONTRIBUTING.md](CONTRIBUTING.md).

## Сборка из исходников

Нужны Flatpak, `flatpak-builder` и SDK/runtime GNOME 50 (скрипт сборки подтянет их с Flathub).

```bash
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Production (io.github.tobagin.Ntfyr)
./build.sh

# Development (io.github.tobagin.Ntfyr.Devel, debug, pre-commit hook)
./build.sh --dev
```

Сборка устанавливается в пользовательский Flatpak-репозиторий `~/repo`. Запуск:

```bash
flatpak run io.github.tobagin.Ntfyr          # production
flatpak run io.github.tobagin.Ntfyr.Devel    # development
```

Быстрая итерация без Flatpak (нужны пакеты `gtk4-devel`, `libadwaita-devel`):

```bash
cargo check   # проверка типов
cargo clippy  # линтер
cargo run     # запуск из исходников
```

Архитектура — в [CLAUDE.md](CLAUDE.md), процесс разработки — в [CONTRIBUTING.md](CONTRIBUTING.md).

## Использование

Запустите Ntfyr из меню приложений или выполните:

```bash
flatpak run io.github.tobagin.Ntfyr
```

1. Нажмите **+**, чтобы добавить подписку.
2. Введите имя темы (например, `alerts`).
3. При необходимости выберите свой сервер или добавьте его в настройках.

Тестовое уведомление:

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Горячие клавиши

| Сочетание | Действие |
|-----------|----------|
| `Ctrl+N` | Подписаться на новую тему |
| `Ctrl+F` | Поиск по уведомлениям |
| `Ctrl+,` | Настройки |
| `Ctrl+Q` | Выход |
| `F1` | Справка по клавишам |

## Участие в разработке

Вклад приветствуется! Перед pull request прочитайте [CONTRIBUTING.md](CONTRIBUTING.md).

- **Ошибки:** [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- **Вопросы и идеи:** [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## Лицензия

Ntfyr распространяется под [GPL-3.0-or-later](LICENSE).

## Благодарности

- [ntfy.sh](https://ntfy.sh/) — платформа уведомлений.
- [GNOME](https://www.gnome.org/) — GTK4 и Libadwaita.
- [Rust](https://www.rust-lang.org/) — реализация приложения и демона.

## Скриншоты

| Главное окно | Темы | Настройки |
|--------------|------|-----------|
| ![Main window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |
