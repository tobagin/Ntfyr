use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::prelude::*;
use gtk::{gio, glib};

mod imp {
    use ntfy_daemon::NtfyHandle;

    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/preferences.ui")]
    pub struct NtfyrPreferences {
        #[template_child]
        pub startup_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub sort_descending_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub startup_background_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub app_lock_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub auto_lock_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub auto_lock_timeout: TemplateChild<adw::SpinRow>,
        #[template_child]
        pub show_default_server_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub change_password_row: TemplateChild<adw::ActionRow>,
        pub notifier: OnceCell<NtfyHandle>,
    }

    impl Default for NtfyrPreferences {
        fn default() -> Self {
            let this = Self {
                startup_switch: Default::default(),
                sort_descending_switch: Default::default(),
                startup_background_switch: Default::default(),
                app_lock_switch: Default::default(),
                auto_lock_switch: Default::default(),
                auto_lock_timeout: Default::default(),
                show_default_server_switch: Default::default(),
                change_password_row: Default::default(),

                notifier: Default::default(),
            };

            this
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NtfyrPreferences {
        const NAME: &'static str = "NtfyrPreferences";
        type Type = super::NtfyrPreferences;
        type ParentType = adw::PreferencesDialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NtfyrPreferences {}

    impl WidgetImpl for NtfyrPreferences {}
    impl AdwDialogImpl for NtfyrPreferences {}
    impl PreferencesDialogImpl for NtfyrPreferences {}
}

glib::wrapper! {
    pub struct NtfyrPreferences(ObjectSubclass<imp::NtfyrPreferences>)
        @extends gtk::Widget, adw::Dialog, adw::PreferencesDialog,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl NtfyrPreferences {
    pub fn new(notifier: ntfy_daemon::NtfyHandle) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp()
            .notifier
            .set(notifier)
            .map_err(|_| "notifier")
            .unwrap();

        let settings = gio::Settings::new(crate::config::APP_ID);
        settings
            .bind("run-on-startup", &*obj.imp().startup_switch, "active")
            .build();
        settings.connect_changed(Some("run-on-startup"), move |settings, _| {
            let enabled = settings.boolean("run-on-startup");
            if let Some(app) = gio::Application::default() {
                app.activate_action("sync-autostart", Some(&enabled.to_variant()));
            } else {
                tracing::warn!("Failed to get default application to sync autostart.");
            }
        });
        settings
            .bind("sort-descending", &*obj.imp().sort_descending_switch, "active")
            .build();
        settings
            .bind("start-in-background", &*obj.imp().startup_background_switch, "active")
            .build();
        settings
            .bind("app-lock-enabled", &*obj.imp().app_lock_switch, "active")
            .build();
        settings
            .bind("show-default-server", &*obj.imp().show_default_server_switch, "active")
            .flags(gio::SettingsBindFlags::GET)
            .build();

        let settings_clone = settings.clone();
        let obj_clone = obj.clone();
        obj.imp().show_default_server_switch.connect_active_notify(move |switch| {
             let is_active = switch.is_active();
             let settings = &settings_clone;
             
             // If the switch state matches the settings, it might be a sync from settings (via bind)
             // or a no-op. But since we are GET-bound only, the switch update comes from settings.
             // We only care if user INTERACTED (or switch changed) to a value DIFFERENT from settings?
             // Actually, if settings is TRUE, and user clicks OFF. switch becomes FALSE. settings is TRUE.
             // Handler runs.
             
             // However, checking against settings value is racy or complex.
             // Easier: Just implement the logic.
             // If we set_boolean(true) when it's already true, it's fine.
             
             if is_active {
                 let _ = settings.set_boolean("show-default-server", true);
             } else {
                 // Disabling - check for topics
                 let switch = switch.clone();
                 let obj = obj_clone.clone();
                 let settings = settings.clone();
                 
                 glib::MainContext::default().spawn_local(async move {
                      let notifier = obj.imp().notifier.get().unwrap();
                      let list = notifier.list_subscriptions().await.unwrap_or_default();
                      
                      let mut has_topics = false;
                      for sub_handle in list {
                          let model = sub_handle.model().await;
                          if model.server == "https://ntfy.sh" {
                              has_topics = true;
                              break;
                          }
                      }
                      
                      if has_topics {
                           let dialog = adw::MessageDialog::builder()
                               .heading("Disable Default Server?")
                               .body("You have active topics on ntfy.sh. Disabling the default server will unsubscribe and remove these topics. This action cannot be undone.")
                               .modal(true)
                               .build();
                           
                           dialog.add_response("cancel", "Cancel");
                           dialog.add_response("disable", "Disable & Purge");
                           dialog.set_response_appearance("disable", adw::ResponseAppearance::Destructive);
                           
                           if let Some(root) = obj.root().and_downcast::<gtk::Window>() {
                               dialog.set_transient_for(Some(&root));
                           }
                           dialog.present();
                           
                           let response = dialog.choose_future().await;
                           if response == "disable" {
                                if let Some(app) = gio::Application::default() {
                                     app.activate_action("app.purge-default-server", None);
                                }
                                let _ = settings.set_boolean("show-default-server", false);
                           } else {
                                // Revert switch
                                switch.set_active(true);
                           }
                      } else {
                           let _ = settings.set_boolean("show-default-server", false);
                      }
                 });
             }
        });
        settings
            .bind("auto-lock-enabled", &*obj.imp().auto_lock_switch, "active")
            .build();
        
        // Timeout is in seconds in GSchema, but UI shows minutes. 
        // We'll need a custom mapping or just simple property binding if we change GSchema to minutes? 
        // No, GSchema is usually int. Let's assume we bind it directly for now, but lock-timeout is seconds.
        // Wait, SpinRow shows a value. If GSchema is seconds (e.g. 300), SpinRow 1-60 will be wrong.
        // I should probably change the GSchema to be minutes for simplicity, or use a map_get/set.
        // Let's change GSchema to minutes since it's an int. 300 seconds = 5 minutes.
        // Or better, keep it as seconds but use map_get/set.
        // For simplicity and standard practice, let's use seconds in backend but map to minutes in UI.
        // Actually, for this iteration, let's just make the GSchema 'lock-timeout-minutes' or similar?
        // No, 'lock-timeout' in seconds is better for granularity if needed later.
        // I will use `bind_with_mapping`.
        
        // Manual binding for lock-timeout to handle seconds <-> minutes conversion
        let row = obj.imp().auto_lock_timeout.clone();
        let settings_clone = settings.clone();

        // Initial sync
        let secs = settings.int("lock-timeout");
        row.set_value((secs as f64 / 60.0).round());

        // UI -> Settings
        row.connect_notify_local(Some("value"), move |row, _| {
            let mins = row.value();
            let secs = (mins * 60.0) as i32;
            // Prevent feedback loop? Settings dedupes usually.
            if settings_clone.int("lock-timeout") != secs {
                let _ = settings_clone.set_int("lock-timeout", secs);
            }
        });

        // Settings -> UI
        let row_clone = obj.imp().auto_lock_timeout.clone();
        settings.connect_changed(Some("lock-timeout"), move |settings, _| {
            let secs = settings.int("lock-timeout");
            let mins = (secs as f64 / 60.0).round();
            if (row_clone.value() - mins).abs() > f64::EPSILON {
                row_clone.set_value(mins);
            }
        });

        obj.imp().change_password_row.connect_activated(move |row| {
             let parent_window = row.root().and_then(|w| w.downcast::<gtk::Window>().ok());
 
             glib::MainContext::default().spawn_local(async move {
                 let has_pass = crate::secrets::has_password().await.unwrap_or(false);
                 
                 let dialog = adw::MessageDialog::builder()
                     .heading(if has_pass { "Change Password" } else { "Set App Lock Password" })
                     .body(if has_pass { "Enter your current password and a new password." } else { "Enter a new password to secure the application." })
                     .modal(true)
                     .build();
                 
                 if let Some(window) = &parent_window {
                     dialog.set_transient_for(Some(window));
                 }
                 
                 dialog.add_response("cancel", "Cancel");
                 // Note: "Save" is now a custom button to prevent auto-closing
                 
                 let content_box = gtk::Box::builder()
                     .orientation(gtk::Orientation::Vertical)
                     .spacing(12)
                     .build();

                 let list = adw::PreferencesGroup::new();
                 content_box.append(&list);
                 
                 let current_entry = if has_pass {
                     let e = adw::PasswordEntryRow::builder()
                         .title("Current Password")
                         .activates_default(true)
                         .build();
                     list.add(&e);
                     Some(e)
                 } else {
                     None
                 };
 
                 let new_entry = adw::PasswordEntryRow::builder()
                     .title("New Password")
                     .activates_default(true)
                     .build();
                 list.add(&new_entry);
 
                 let confirm_entry = adw::PasswordEntryRow::builder()
                     .title("Confirm Password")
                     .activates_default(true)
                     .build();
                 list.add(&confirm_entry);

                 let error_label = gtk::Label::builder()
                     .css_classes(["error", "caption"])
                     .halign(gtk::Align::Center)
                     .visible(false)
                     .margin_bottom(12)
                     .build();
                 content_box.append(&error_label);

                 let save_button = gtk::Button::builder()
                     .label("Save")
                     .css_classes(["suggested-action", "pill"])
                     .margin_top(12)
                     .margin_bottom(12)
                     .halign(gtk::Align::Center)
                     .width_request(120)
                     .build();
                 content_box.append(&save_button);
 
                 dialog.set_extra_child(Some(&content_box));
                 
                 let d = dialog.clone();
                 // Clone widgets for the closure
                 let current_entry_c = current_entry.clone();
                 let new_entry_c = new_entry.clone();
                 let confirm_entry_c = confirm_entry.clone();
                 let error_label_c = error_label.clone();
                 
                 save_button.connect_clicked(move |_| {
                     let d = d.clone();
                     let current_entry = current_entry_c.clone();
                     let new_entry = new_entry_c.clone();
                     let confirm_entry = confirm_entry_c.clone();
                     let error_label = error_label_c.clone();
                     
                     glib::MainContext::default().spawn_local(async move {
                         let new_pass = new_entry.text().to_string();
                         let confirm_pass = confirm_entry.text().to_string();

                         // Visual Reset
                         new_entry.remove_css_class("error");
                         confirm_entry.remove_css_class("error");
                         if let Some(c) = &current_entry {
                             c.remove_css_class("error");
                         }
                         error_label.set_visible(false);

                         // Validate confirmation
                         if new_pass != confirm_pass {
                             error_label.set_text("Passwords do not match");
                             error_label.set_visible(true);
                             new_entry.add_css_class("error");
                             confirm_entry.add_css_class("error");
                             return;
                         }

                         if new_pass.is_empty() {
                             error_label.set_text("Empty password not allowed");
                             error_label.set_visible(true);
                             new_entry.add_css_class("error");
                             return;
                         }

                         if let Some(curr) = current_entry {
                             let curr_pass = curr.text();
                             let stored = crate::secrets::get_password().await.unwrap_or(None);
                             if let Some(stored_pass) = stored {
                                 if curr_pass != stored_pass {
                                     error_label.set_text("Current password incorrect");
                                     error_label.set_visible(true);
                                     curr.add_css_class("error");
                                     curr.grab_focus();
                                     return;
                                 }
                             }
                         }

                         // Valid
                         if let Err(e) = crate::secrets::store_password(&new_pass).await {
                             tracing::error!("Failed to store password: {}", e);
                             error_label.set_text(&format!("Error: {}", e));
                             error_label.set_visible(true);
                         } else {
                             tracing::info!("Password set successfully");
                             d.close();
                         }
                     });
                 });

                 // Helper to trigger save on enter
                 if let Some(e) = &current_entry {
                     let b = save_button.clone();
                     e.connect_apply(move |_| { b.activate(); });
                 }
                 let b = save_button.clone();
                 new_entry.connect_apply(move |_| { b.activate(); });
                 let b = save_button.clone();
                 confirm_entry.connect_apply(move |_| { b.activate(); });
                 
                 dialog.present();
             });
        });

        obj
    }


}
