# Ntfyr

**Idiomas:** [English](README.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md)

Cliente nativo de [ntfy.sh](https://ntfy.sh/) para el escritorio GNOME. Desarrollado con Rust, GTK4 y Libadwaita; distribuido como Flatpak.

<div align="center">

![Ntfyr Application](data/screenshots/main-window.png)

<a href="https://flathub.org/en/apps/io.github.tobagin.Ntfyr"><img src="https://flathub.org/api/badge" height="110" alt="Get it on Flathub"></a>
<a href="https://ko-fi.com/tobagin"><img src="data/kofi_button.png" height="82" alt="Support me on Ko-Fi"></a>

</div>

## Última versión: 0.6.2

Resumen de cambios recientes (historial completo en [CHANGELOG.md](CHANGELOG.md)):

- **0.6.2 — versión de seguridad** — correcciones de una auditoría completa: escape de URLs en enlaces Markdown, restricción de las URLs de acción a http(s) con confirmación, comprobaciones del bloqueo de la app en tiempo constante y un daemon reforzado frente a datos malformados del servidor.
- **Selector de idioma de la interfaz** — elige el idioma en Preferencias (sistema, inglés US/UK, ruso, alemán, español, francés, portugués PT/BR); al reiniciar se aplican las traducciones en toda la app.
- **Historial al suscribirse** — las suscripciones nuevas cargan la caché del tema del servidor sin reenviar mensajes borrados ni saturar con alertas de escritorio.
- **Modo en segundo plano fiable** — reactivar desde el lanzador siempre muestra la ventana; mejor integración con la bandeja y portales en GNOME y KDE.
- **Self-hosted** — servidores privados HTTPS, IDs estables de notificaciones de escritorio, borrado atómico y respaldos robustos del keyring si el portal Secret no está disponible.
- **0.6.1** — versión de mantenimiento con dependencias Rust actualizadas (GTK 0.22 / ashpd 0.13 / reqwest 0.13).

Instala la versión estable desde [Flathub](https://flathub.org/en/apps/io.github.tobagin.Ntfyr) o compila desde el código fuente (abajo).

## Características

### Núcleo
- **Integración nativa** — GTK4 + Libadwaita, GNOME Platform 50.
- **Notificaciones push** — suscríbete a temas en `ntfy.sh` o servidores propios.
- **Daemon en segundo plano** — backend en proceso mantiene conexiones SSE y entrega notificaciones de escritorio.
- **Historial local** — mensajes en SQLite para consulta sin conexión y búsqueda.
- **Varios servidores** — suscripciones agrupadas por servidor; ocultar o mostrar `ntfy.sh` por defecto.

### Notificaciones y contenido
- **Cifrado de extremo a extremo** — enviar y recibir mensajes cifrados.
- **Adjuntos** — ver imágenes y descargar archivos en la app.
- **Botones de acción** — abrir enlaces y ejecutar acciones HTTP desde notificaciones.
- **Filtros** — reglas de filtrado por suscripción.

### Privacidad y seguridad
- **Bloqueo de la app** — PIN/biometría opcional al iniciar (mediante keyring del sistema).
- **Self-hosted** — conectar a instancias ntfy privadas por HTTPS.
- **Flatpak aislado** — permisos estrictos; sin telemetría.
- **Solo datos locales** — ajustes e historial permanecen en tu equipo.

### Experiencia de escritorio
- **Bandeja del sistema** — acceso rápido e indicador de no leídos (icono simbólico en paneles oscuros).
- **Atajos de teclado** — `Ctrl+N` nuevo tema, `Ctrl+F` buscar, `Ctrl+,` preferencias, `F1` ayuda.
- **Preferencias** — formato de fecha/hora, inicio automático, arranque en segundo plano, estado de ventana.
- **Modo oscuro** — sigue el tema del sistema.

### Traducciones

Interfaz disponible en inglés, ruso, alemán, español, francés y portugués (variantes europea y brasileña en la app). Más idiomas vía gettext — consulta [CONTRIBUTING.md](CONTRIBUTING.md).

## Compilar desde el código fuente

Requiere Flatpak, `flatpak-builder` y SDK/runtime GNOME 50 (el script de compilación los instala desde Flathub).

```bash
git clone https://github.com/tobagin/Ntfyr.git
cd Ntfyr

# Producción (io.github.tobagin.Ntfyr)
./build.sh

# Desarrollo (io.github.tobagin.Ntfyr.Devel, perfil debug, hook pre-commit)
./build.sh --dev
```

Cada compilación se instala en el repositorio Flatpak de usuario `~/repo`. Ejecutar:

```bash
flatpak run io.github.tobagin.Ntfyr          # production
flatpak run io.github.tobagin.Ntfyr.Devel    # development
```

Iteración rápida sin Flatpak (requiere paquetes `gtk4-devel`, `libadwaita-devel`):

```bash
cargo check   # comprobación de tipos
cargo clippy  # lint
cargo run     # ejecutar desde fuentes
```

Arquitectura: [CLAUDE.md](CLAUDE.md); flujo de desarrollo: [CONTRIBUTING.md](CONTRIBUTING.md).

## Uso

Inicia Ntfyr desde el menú de aplicaciones o ejecuta:

```bash
flatpak run io.github.tobagin.Ntfyr
```

1. Pulsa **+** para añadir una suscripción.
2. Introduce el nombre del tema (p. ej. `alerts`).
3. Opcionalmente elige un servidor personalizado o añádelo en Preferencias.

Enviar una notificación de prueba:

```bash
curl -d "Hello from CLI" ntfy.sh/mytopic
```

### Atajos de teclado

| Atajo | Acción |
|-------|--------|
| `Ctrl+N` | Suscribirse a un tema nuevo |
| `Ctrl+F` | Buscar notificaciones |
| `Ctrl+,` | Abrir Preferencias |
| `Ctrl+Q` | Salir |
| `F1` | Mostrar atajos de teclado |

## Contribuir

¡Las contribuciones son bienvenidas! Lee [CONTRIBUTING.md](CONTRIBUTING.md) antes de abrir un pull request.

- **Errores:** [GitHub Issues](https://github.com/tobagin/Ntfyr/issues)
- **Preguntas e ideas:** [GitHub Discussions](https://github.com/tobagin/Ntfyr/discussions)

## Licencia

Ntfyr está bajo [GPL-3.0-or-later](LICENSE).

## Agradecimientos

- [ntfy.sh](https://ntfy.sh/) — la plataforma de notificaciones.
- [GNOME](https://www.gnome.org/) — GTK4 y Libadwaita.
- [Rust](https://www.rust-lang.org/) — implementación de la app y el daemon.

## Capturas de pantalla

| Ventana principal | Temas | Preferencias |
|-------------------|-------|--------------|
| ![Main window](data/screenshots/main-window.png) | ![Topics](data/screenshots/topics.png) | ![Preferences](data/screenshots/preferences.png) |
