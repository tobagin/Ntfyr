use ksni;
use ksni::TrayMethods;
use std::error::Error;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use gettextrs::gettext;

use crate::config::APP_ID;

pub struct NtfyrTray {
    pub visible: Arc<AtomicBool>,
    pub has_unread: Arc<AtomicBool>,
}

impl ksni::Tray for NtfyrTray {
    fn icon_name(&self) -> String {
        // Symbolic SVGs use the KDE `ColorScheme-Text` / `ColorScheme-Highlight`
        // stylesheet idiom plus `currentColor`, so Plasma recolors them for the
        // active panel theme and GTK inherits the foreground color too.
        if self.has_unread.load(Ordering::Relaxed) {
            format!("{}-new-notification-symbolic", APP_ID)
        } else {
            format!("{}-symbolic", APP_ID)
        }
    }

    fn title(&self) -> String {
        gettext("Ntfyr")
    }

    fn id(&self) -> String {
        APP_ID.to_string()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        glib::MainContext::default().invoke(move || {
            if let Some(app) = gio::Application::default() {
                app.activate_action("toggle-window", None);
            }
        });
    }

    fn category(&self) -> ksni::Category {
        ksni::Category::ApplicationStatus
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: gettext("Ntfyr"),
            description: gettext("Notifications in background"),
            ..Default::default()
        }
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let label = if self.visible.load(Ordering::Relaxed) {
             gettext("Hide Window")
        } else {
             gettext("Show Window")
        };
        vec![
            StandardItem {
                label: label.into(),
                activate: Box::new(|_| {
                    glib::MainContext::default().invoke(move || {
                        if let Some(app) = gio::Application::default() {
                            if let Ok(gtk_app) = app.clone().downcast::<gtk::Application>() {
                                if let Some(window) = gtk_app.windows().first() {
                                    if window.is_visible() {
                                        window.set_visible(false);
                                    } else {
                                        window.present();
                                    }
                                } else {
                                     // Fallback to action if no window found
                                     app.activate_action("toggle-window", None);
                                }
                            }
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: gettext("Quit").into(),
                activate: Box::new(|_| {
                     glib::MainContext::default().invoke(move || {
                        if let Some(app) = gio::Application::default() {
                            app.activate_action("quit", None);
                        }
                     });
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn spawn_tray(visible: Arc<AtomicBool>, has_unread: Arc<AtomicBool>) -> Result<ksni::Handle<NtfyrTray>, Box<dyn Error>> {
    let tray = NtfyrTray { visible, has_unread };
    // Create a new runtime specifically for the tray event loop if strictly needed,
    // but ksni might blocking run on prompt.
    // Karere used `rt.block_on(tray.spawn())`.
    // We can leak the runtime to keep it valid.
    let rt = tokio::runtime::Runtime::new()?;
    let handle = rt.block_on(tray.disable_dbus_name(true).spawn())?;
    std::mem::forget(rt);
    Ok(handle)
}
