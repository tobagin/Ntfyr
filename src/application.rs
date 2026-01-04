use std::cell::Cell;
use std::pin::Pin;
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;
use futures::stream::Stream;
use gtk::{gdk, gio, glib};
use ntfy_daemon::models;
use ntfy_daemon::NtfyHandle;
use tracing::{debug, error, info, warn};

use crate::config::{APP_ID, PKGDATADIR, PROFILE, VERSION};
use crate::widgets::*;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::tray;

mod imp {
    use std::cell::RefCell;

    use glib::WeakRef;
    use once_cell::sync::OnceCell;

    use super::*;

    #[derive(Default)]
    pub struct NtfyrApplication {
        pub window: RefCell<WeakRef<NtfyrWindow>>,
        pub hold_guard: OnceCell<gio::ApplicationHoldGuard>,
        pub ntfy: OnceCell<NtfyHandle>,
        pub tray: OnceCell<ksni::Handle<tray::NtfyrTray>>,
        pub tray_visible: Arc<AtomicBool>,
        pub tray_has_unread: Arc<AtomicBool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NtfyrApplication {
        const NAME: &'static str = "NtfyrApplication";
        type Type = super::NtfyrApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for NtfyrApplication {}

    impl ApplicationImpl for NtfyrApplication {
        fn activate(&self) {
            debug!("AdwApplication<NtfyrApplication>::activate");
            self.parent_activate();
            
            let app = self.obj();
            let settings = gio::Settings::new(crate::config::APP_ID);
            let start_in_background = settings.boolean("start-in-background");
            
            // If the window already exists, it means the app was already running and 
            // the user is activating it again (e.g. clicking the icon). In that case, 
            // we should always present the window.
            let has_window = app.imp().window.borrow().upgrade().is_some();
            
            if !start_in_background || has_window {
                app.ensure_window_present();
            } else {
                debug!("Starting in background as requested by preferences");
                // We still need to ensure RPC is running if it's the first activation
                if self.hold_guard.get().is_none() {
                    app.ensure_rpc_running();
                }
            }
        }

        fn startup(&self) {
            debug!("AdwApplication<NtfyrApplication>::startup");
            self.parent_startup();
            let app = self.obj();

            // Set icons for shell
            gtk::Window::set_default_icon_name(APP_ID);

            // Spawn tray
            let visible = app.imp().tray_visible.clone();
            let has_unread = app.imp().tray_has_unread.clone();
            if let Ok(handle) = crate::tray::spawn_tray(visible, has_unread) {
                app.imp().tray.set(handle).ok();
            } else {
                 warn!("Failed to spawn tray icon");
            }

            app.setup_css();
            app.setup_gactions();
            app.setup_accels();
            app.setup_autostart();
            
            // Karere-style background portal request at startup
            let settings = gio::Settings::new(APP_ID);
            let autostart_enabled = settings.boolean("run-on-startup");
            
            crate::async_utils::RUNTIME.spawn(async move {
                if let Err(e) = super::NtfyrApplication::run_in_background(None, autostart_enabled).await {
                    warn!("Failed to request background permission at startup: {}", e);
                }
            });
        }

        fn command_line(&self, command_line: &gio::ApplicationCommandLine) -> glib::ExitCode {
            debug!("AdwApplication<NtfyrApplication>::command_line");
            let arguments = command_line.arguments();
            let is_daemon = arguments.get(1).map(|x| x.to_str()) == Some(Some("--daemon"));
            let app = self.obj();

            if self.hold_guard.get().is_none() {
                app.ensure_rpc_running();
            }

            let settings = gio::Settings::new(crate::config::APP_ID);
            let autostart = settings.boolean("run-on-startup");
            crate::async_utils::RUNTIME.spawn(async move {
                if let Err(e) = super::NtfyrApplication::run_in_background(None, autostart).await {
                    warn!(error = %e, "couldn't request running in background from portal");
                }
            });

            if is_daemon {
                return glib::ExitCode::SUCCESS;
            }

            let start_in_background = settings.boolean("start-in-background");
            let has_window = app.imp().window.borrow().upgrade().is_some();

            if !start_in_background || has_window {
                app.ensure_window_present();
            } else {
                debug!("Starting in background from command line as requested by preferences");
            }

            glib::ExitCode::SUCCESS
        }
    }

    impl GtkApplicationImpl for NtfyrApplication {}
    impl AdwApplicationImpl for NtfyrApplication {}
}

glib::wrapper! {
    pub struct NtfyrApplication(ObjectSubclass<imp::NtfyrApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionMap, gio::ActionGroup;
}

impl NtfyrApplication {
    fn ensure_window_present(&self) {
        if let Some(window) = self.imp().window.borrow().upgrade() {
            window.present();
            return;
        }
        self.build_window();
        self.main_window().present();
    }

    fn main_window(&self) -> NtfyrWindow {
        self.imp().window.borrow().upgrade().unwrap()
    }

    fn setup_gactions(&self) {
        // Quit
        let action_quit = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| {
                // This is needed to trigger the delete event and saving the window state
                if let Some(win) = app.imp().window.borrow().upgrade() {
                    let _ = win.save_window_size();
                    win.close();
                }
                app.quit();
                std::process::exit(0);
            })
            .build();

        // About
        let action_about = gio::ActionEntry::builder("about")
            .activate(|app: &Self, _, _| {
                app.show_about_dialog();
            })
            .build();

        let action_preferences = gio::ActionEntry::builder("preferences")
            .activate(|app: &Self, _, _| {
                app.show_preferences();
            })
            .build();

        let message_action = gio::ActionEntry::builder("message-action")
            .parameter_type(Some(&glib::VariantTy::STRING))
            .activate(|app: &Self, _, params| {
                let Some(params) = params else {
                    return;
                };
                let Some(s) = params.str() else {
                    warn!("action is not a string");
                    return;
                };
                let Ok(action) = serde_json::from_str(s) else {
                    error!("invalid action json");
                    return;
                };
                app.handle_message_action(action);
            })
            .build();
        let action_shortcuts = gio::ActionEntry::builder("shortcuts")
            .activate(|app: &Self, _, _| {
                app.show_shortcuts();
            })
            .build();

        self.add_action_entries([
            action_quit,
            action_about,
            action_shortcuts,
            action_preferences,
            message_action,
        ]);
        
        let action_toggle_window = gio::ActionEntry::builder("toggle-window")
            .activate(move |app: &Self, _, _| {
                if let Some(win) = app.imp().window.borrow().upgrade() {
                    if win.is_visible() {
                         win.set_visible(false);
                    } else {
                         win.present();
                    }
                } else {
                    app.ensure_window_present();
                }
            })
            .build();
        self.add_action_entries([action_toggle_window]);
    }

    fn handle_message_action(&self, action: models::Action) {
        match action {
            models::Action::View { url, .. } => {
                gtk::UriLauncher::builder().uri(url.clone()).build().launch(
                    gtk::Window::NONE,
                    gio::Cancellable::NONE,
                    |_| {},
                );
            }
            models::Action::Http {
                method,
                url,
                body,
                headers,
                ..
            } => {
                gio::spawn_blocking(move || {
                    let agent = ureq::Agent::new_with_config(
                        Default::default()
                    );
                    
                    macro_rules! set_headers {
                        ($req:expr) => {{
                            let mut r = $req;
                            for (k, v) in headers.iter() {
                                r = r.header(k, v);
                            }
                            r
                        }}
                    }

                   let res = match method.as_str() {
                        "GET" => set_headers!(agent.get(url.as_str())).call(),
                        "POST" => set_headers!(agent.post(url.as_str())).send(body.as_bytes()),
                        "PUT" => set_headers!(agent.put(url.as_str())).send(body.as_bytes()),
                        "DELETE" => set_headers!(agent.delete(url.as_str())).call(),
                        "HEAD" => set_headers!(agent.head(url.as_str())).call(),
                        "PATCH" => set_headers!(agent.patch(url.as_str())).send(body.as_bytes()),
                        "OPTIONS" => set_headers!(agent.options(url.as_str())).call(),
                        "TRACE" => set_headers!(agent.trace(url.as_str())).call(),
                        _ => set_headers!(agent.get(url.as_str())).call(),
                    };
                    match res {
                        Err(e) => {
                            error!(error = ?e, "Error sending request");
                        }
                        Ok(_) => {}
                    }
                });
            }
            _ => {}
        }
    }

    // Sets up keyboard shortcuts
    fn setup_accels(&self) {
        self.set_accels_for_action("app.quit", &["<Control>q"]);
        self.set_accels_for_action("window.close", &["<Control>w"]);
        self.set_accels_for_action("app.shortcuts", &["<Control>question"]);
        self.set_accels_for_action("app.preferences", &["<Control>comma"]);
        self.set_accels_for_action("app.about", &["F1"]);
        self.set_accels_for_action("win.add-topic", &["<Control>n"]);
        self.set_accels_for_action("win.search", &["<Control>f"]);
        self.set_accels_for_action("win.clear-notifications", &["<Control><Shift>k"]);
    }

    fn setup_css(&self) {
        let provider = gtk::CssProvider::new();
        provider.load_from_resource("/io/github/tobagin/Ntfyr/style.css");
        if let Some(display) = gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    fn show_about_dialog(&self) {
        let dialog = adw::AboutDialog::from_appdata(
            "/io/github/tobagin/Ntfyr/io.github.tobagin.Ntfyr.metainfo.xml",
            None,
        );
        
        dialog.add_link("Support Questions", "https://github.com/tobagin/Ntfyr/discussions");
        
        dialog.add_credit_section(Some("Developers"), &["Tobagin", "Ranfdev"]);
        dialog.add_credit_section(Some("Designers"), &["Tobagin"]);
        dialog.add_credit_section(Some("Acknowledgements"), &["GTK4", "Libadwaita", "ntfy-rs", "gettext-rs"]);
        
        dialog.set_copyright("© 2019-2026 The Ntfyr Team");
        dialog.set_license_type(gtk::License::Gpl30);

        if let Some(w) = self.imp().window.borrow().upgrade() {
            dialog.present(Some(&w));
        }
    }

    fn show_shortcuts(&self) {
        let builder = gtk::Builder::from_resource("/io/github/tobagin/Ntfyr/gtk/help-overlay.ui");
        let dialog: adw::ShortcutsDialog = builder.object("help_overlay")
            .expect("shortcuts.ui MUST have help_overlay object");
        if let Some(w) = self.imp().window.borrow().upgrade() {
            dialog.present(Some(&w));
        }
    }

    fn show_preferences(&self) {
        let win = crate::widgets::NtfyrPreferences::new(
            self.main_window().imp().notifier.get().unwrap().clone(),
        );
        win.present(Some(&self.main_window()));
    }

    pub fn run(&self) -> glib::ExitCode {
        info!(app_id = %APP_ID, version = %VERSION, profile = %PROFILE, datadir = %PKGDATADIR, "running");

        glib::ExitCode::from(self.run_with_args(&std::env::args().collect::<Vec<_>>()))
    }
    
    fn setup_autostart(&self) {
        let settings = gio::Settings::new(crate::config::APP_ID);
        
        let app = self.clone();
        settings.connect_changed(Some("run-on-startup"), move |_, _| {
            debug!("Run on startup setting changed");
            let app = app.clone();
            
            // We need to get the window from the main thread
            let identifier = if let Some(win) = app.imp().window.borrow().upgrade() {
                // from_native is async and needs a reactor. 
                // We use a temporary runtime on the main thread here.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async move {
                    match ashpd::WindowIdentifier::from_native(&win).await {
                        Some(id) => Some(id),
                        None => {
                            warn!("Failed to get window identifier");
                            None
                        }
                    }
                })
            } else {
                None
            };

            // Convert identifier to string to be Send
            let identifier_str = identifier.map(|id| id.to_string());

            let settings = gio::Settings::new(crate::config::APP_ID);
            let autostart = settings.boolean("run-on-startup");

            // Run the portal request in a background thread to avoid blocking and provide a reactor for zbus
            crate::async_utils::RUNTIME.spawn(async move {
                info!("Calling run_in_background from background thread");
                if let Err(e) = Self::run_in_background(identifier_str, autostart).await {
                     warn!("Failed to update autostart portal: {}", e);
                } else {
                    info!("Autostart portal updated");
                }
            });
        });
    }

    #[allow(dead_code)]
    fn update_autostart_file(&self, _enable: bool) -> std::io::Result<()> {
        // Legacy method, not used in the portal-based autostart pattern
        Ok(())
    }


    async fn run_in_background(identifier: Option<String>, autostart: bool) -> anyhow::Result<()> {
        info!(autostart_request = autostart, "Initiating background portal request via zbus");

        let connection = zbus::Connection::session().await?;
        let proxy = zbus::Proxy::new(
            &connection,
            "org.freedesktop.portal.Desktop",
            "/org/freedesktop/portal/desktop",
            "org.freedesktop.portal.Background",
        )
        .await?;

        let mut options = std::collections::HashMap::new();
        options.insert("reason", zbus::zvariant::Value::from("Receive notifications in the background"));
        options.insert("autostart", zbus::zvariant::Value::from(autostart));
        // Note: portal expects "dbus-activatable" with a hyphen
        options.insert("commandline", zbus::zvariant::Value::from(vec!["ntfyr", "--daemon"]));
        options.insert("dbus-activatable", zbus::zvariant::Value::from(false));

        let parent_window = identifier.unwrap_or_default();

        let request_path: zbus::zvariant::OwnedObjectPath = proxy
            .call_method("RequestBackground", &(&parent_window, &options))
            .await?
            .body()
            .deserialize()?;

        info!(request_path = %request_path, "Background portal request initiated");

        // Karere pattern: also set status to ensure it appears in GNOME Background Apps
        let mut status_options = std::collections::HashMap::new();
        status_options.insert("message", zbus::zvariant::Value::from("Running in background"));
        
        if let Err(e) = proxy.call_method("SetStatus", &(status_options)).await {
            warn!("Failed to set background status: {}", e);
        } else {
            debug!("Background status set successfully");
        }

        Ok(())
    }



    fn ensure_rpc_running(&self) {
        let dbpath = glib::user_data_dir().join("io.github.tobagin.Ntfyr.sqlite");
        info!(database_path = %dbpath.display());

        // Here I'm sending notifications to the desktop environment and listening for network changes.
        // This should have been inside ntfy-daemon, but using portals from another thread causes the error
        // `Invalid client serial` and it's broken.
        // Until https://github.com/flatpak/xdg-dbus-proxy/issues/46 is solved, I have to handle these things
        // in the main thread. Uff.

        let (s, r) = async_channel::unbounded::<models::Notification>();

        let app = self.clone();
        glib::MainContext::ref_thread_default().spawn_local(async move {
            while let Ok(n) = r.recv().await {
                let gio_notif = gio::Notification::new(&n.title);
                gio_notif.set_body(Some(&n.body));

                let action_name = |a| {
                    let json = serde_json::to_string(a).unwrap();
                    gio::Action::print_detailed_name("app.message-action", Some(&json.into()))
                };
                for a in n.actions.iter() {
                    match a {
                        models::Action::View { label, .. } => {
                            gio_notif.add_button(&label, &action_name(a))
                        }
                        models::Action::Http { label, .. } => {
                            gio_notif.add_button(&label, &action_name(a))
                        }
                        _ => {}
                    }
                }

                app.send_notification(None, &gio_notif);
                app.set_unread(true);
            }
        });
        struct Proxies {
            notification: async_channel::Sender<models::Notification>,
        }
        impl models::NotificationProxy for Proxies {
            fn send(&self, n: models::Notification) -> anyhow::Result<()> {
                self.notification.send_blocking(n)?;
                Ok(())
            }
        }
        impl models::NetworkMonitorProxy for Proxies {
            fn listen(&self) -> Pin<Box<dyn Stream<Item = ()>>> {
                let (tx, rx) = async_channel::bounded(1);
                let prev_available = Rc::new(Cell::new(false));

                gio::NetworkMonitor::default().connect_network_changed(move |_, available| {
                    if available && !prev_available.get() {
                        if let Err(e) = tx.send_blocking(()) {
                            warn!(error = %e);
                        }
                    }
                    prev_available.replace(available);
                });

                Box::pin(rx)
            }
        }
        let proxies = std::sync::Arc::new(Proxies { notification: s });
        let ntfy = ntfy_daemon::start(dbpath.to_str().unwrap(), proxies.clone(), proxies).unwrap();
        self.imp()
            .ntfy
            .set(ntfy)
            .or(Err(anyhow::anyhow!("failed setting ntfy")))
            .unwrap();
        self.imp().hold_guard.set(self.hold()).unwrap();
    }

    fn build_window(&self) {
        let ntfy = self.imp().ntfy.get().unwrap();

        let window = NtfyrWindow::new(self, ntfy.clone());
        
        let visible = self.imp().tray_visible.clone();
        let app = self.clone();
        window.connect_notify_local(Some("visible"), move |win, _| {
            let is_visible = win.is_visible();
            visible.store(is_visible, Ordering::Relaxed);
            if is_visible {
                app.set_unread(false);
            }
            app.update_tray();
        });
        // Sync initial state
        self.imp().tray_visible.store(window.is_visible(), Ordering::Relaxed);
        
        *self.imp().window.borrow_mut() = window.downgrade();
    }
    fn set_unread(&self, unread: bool) {
        let imp = self.imp();
        if imp.tray_has_unread.load(Ordering::Relaxed) != unread {
            imp.tray_has_unread.store(unread, Ordering::Relaxed);
            self.update_tray();
        }
    }

    fn update_tray(&self) {
        if let Some(handle) = self.imp().tray.get() {
            let handle = handle.clone();
            crate::async_utils::RUNTIME.spawn(async move {
                let _ = handle.update(|_| {}).await;
            });
        }
    }
}

impl Default for NtfyrApplication {
    fn default() -> Self {
        glib::Object::builder()
            .property("application-id", APP_ID)
            .property("flags", gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .property("resource-base-path", "/io/github/tobagin/Ntfyr/")
            .build()
    }
}
