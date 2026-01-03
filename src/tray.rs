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
        // We assume we have icons named after the APP_ID and APP_ID-new-message-symbolic?
        // User said: "we have two tray icons to use in the icon folder, one to identify when there is new notifications and the standard."
        // Karere uses: format!("{}-new-message-symbolic", app_id)
        // I'll stick to that convention for now, or check file names later.
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
                            // We can use helper methods providing we expose them or use actions
                            // Since we are in the same crate, we can cast to NtfyrApplication
                            // But controlling window directly might be cleaner via actions?
                            // Karere uses manual window control.
                            // NtfyrApplication has ensure_window_present.
                            // It doesn't seem to have a public "hide" method but we can use actions.
                            // Or we can implement it here.
                            // NtfyrApplication has `window` RefCell<WeakRef<NtfyrWindow>>
                            // access it is restricted (imp module).
                            // BUT NtfyrApplication is a public struct in application.rs
                            // The fields are in `imp` struct.
                            // We can use `app.activate_action("window.close", None)` to hide?
                            // No, "window.close" (Control+W) usually closes the window which *might* mean hide if it's a daemon?
                            // In `NtfyrApplication::setup_gactions`:
                            // action_quit -> closes window and quits.
                            // There is no explicit "hide" action.
                            // However, standard GtkApplicationWindow behavior: if you close the last window, and application has hold count, it keeps running.
                            // So `window.close()` is effectively "Hide" if the daemon is running.
                            // BUT if we want to "Show", we need `ensure_window_present()`.
                            // NtfyrApplication methods like `ensure_window_present` are private (fn ensure_window_present).
                            // Wait, `ensure_window_present` is not `pub`.
                            // I should probably expose a "toggle-window" action.
                            // Or make `ensure_window_present` public.
                            // Let's invoke an action "app.toggle-window" which I should add?
                            // Or use "activate" which calls `ensure_window_present`.
                            // `app.activate()` calls `ensure_window_present()`.
                            // So to SHOW, we can call `app.activate()`.
                            // To HIDE (close window), we can find the window and close it?
                            // application.rs: `action_quit` closes window:
                            // `if let Some(win) = app.imp().window.borrow().upgrade() { win.close(); }`
                            // So I can implement a "toggle-window" action in application.rs that does exactly this.
                            // Does it toggle? No.
                            // So I should add an action "toggle-window".
                            app.activate_action("toggle-window", None);
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
