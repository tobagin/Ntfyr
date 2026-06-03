//! Application locale and gettext setup.

use gettextrs::gettext;
use gtk::gio;
use gtk::prelude::*;

use crate::config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR};

/// Relaunch this app (Flatpak or native .desktop) and quit the current instance.
pub fn restart_application() {
    let Some(app) = gio::Application::default() else {
        return;
    };
    let Some(app_id) = app.application_id() else {
        app.quit();
        return;
    };
    match std::process::Command::new("gtk-launch")
        .arg(app_id.as_str())
        .spawn()
    {
        Ok(_) => app.quit(),
        Err(e) => {
            tracing::warn!("gtk-launch failed for {app_id}: {e}; keeping current instance running");
        }
    }
}

pub const LANG_SYSTEM: &str = "system";

/// Language codes stored in GSettings (`interface-language`).
pub const LANGUAGE_CODES: &[&str] = &["system", "en", "ru", "de", "es", "fr"];

/// Native names for the language picker (not translated), except the system entry.
pub fn language_labels() -> Vec<String> {
    vec![
        gettext("System default").to_string(),
        "English".to_string(),
        "Русский".to_string(),
        "Deutsch".to_string(),
        "Español".to_string(),
        "Français".to_string(),
    ]
}

pub fn init() {
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)
        .expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let settings = gio::Settings::new(APP_ID);
    let code = settings.string("interface-language");
    apply_language_code(&code);
}

/// Selects the gettext catalog via `LANGUAGE`.
///
/// The Flatpak runtime only ships a handful of libc locales (mostly `C` and `en_*`),
/// so we must not call `setlocale` with names like `ru_RU.UTF-8` — that panics at runtime.
pub fn apply_language_code(code: &str) {
    // SAFETY: called from the GTK main thread when the user changes language in settings.
    unsafe {
        match code {
            LANG_SYSTEM => std::env::remove_var("LANGUAGE"),
            "en" => std::env::set_var("LANGUAGE", "en"),
            "ru" => std::env::set_var("LANGUAGE", "ru"),
            "de" => std::env::set_var("LANGUAGE", "de"),
            "es" => std::env::set_var("LANGUAGE", "es"),
            "fr" => std::env::set_var("LANGUAGE", "fr"),
            _ => std::env::remove_var("LANGUAGE"),
        }
    }
}

pub fn selected_language_index(settings: &gio::Settings) -> u32 {
    let current = settings.string("interface-language");
    LANGUAGE_CODES
        .iter()
        .position(|c| *c == current.as_str())
        .unwrap_or(0) as u32
}
