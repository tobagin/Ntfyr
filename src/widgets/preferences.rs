use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};
use tracing::debug;

use crate::error::*;
use super::NtfyrAccountDialog;

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
        pub add_server_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub added_servers: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub add_account_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub added_accounts: TemplateChild<gtk::ListBox>,
        pub notifier: OnceCell<NtfyHandle>,
    }

    impl Default for NtfyrPreferences {
        fn default() -> Self {
            let this = Self {
                startup_switch: Default::default(),
                sort_descending_switch: Default::default(),
                startup_background_switch: Default::default(),
                add_server_btn: Default::default(),
                added_servers: Default::default(),
                add_account_btn: Default::default(),
                added_accounts: Default::default(),

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
        settings
            .bind("sort-descending", &*obj.imp().sort_descending_switch, "active")
            .build();
        settings
            .bind("start-in-background", &*obj.imp().startup_background_switch, "active")
            .build();

        // Server Logic
        let this = obj.clone();
        settings.connect_changed(Some("custom-servers"), move |_, _| {
            let this = this.clone();
            glib::MainContext::default().spawn_local(async move {
                this.update_servers_ui();
            });
        });

        
        let this = obj.clone();
        settings.connect_changed(Some("default-server"), move |_, _| {
             let this = this.clone();
             glib::MainContext::default().spawn_local(async move {
                 this.update_servers_ui();
             });
        });


        // Initial update
        let this = obj.clone();
        glib::MainContext::default().spawn_local(async move {
            this.update_servers_ui();
        });

        let this = obj.clone();
        obj.imp().add_server_btn.connect_clicked(move |btn| {
            let this = this.clone();
             btn.error_boundary().spawn(async move {
                  this.on_add_server_clicked().await
             });
        });

        // settings.connect_changed("run-on-startup") handled in application.rs

        let this = obj.clone();
        obj.imp().add_account_btn.connect_clicked(move |btn| {
            let this = this.clone();
            btn.error_boundary()
                .spawn(async move { this.on_add_account_clicked().await });
        });
        let this = obj.clone();
        obj.imp()
            .added_accounts
            .error_boundary()
            .spawn(async move { this.show_accounts().await });
        obj
    }

    pub async fn show_accounts(&self) -> anyhow::Result<()> {
        debug!("show_accounts: starting");
        let imp = self.imp();
        let accounts = imp.notifier.get().unwrap().list_accounts().await?;
        debug!("show_accounts: accounts found: {}", accounts.len());

        while let Some(child) = imp.added_accounts.last_child() {
            imp.added_accounts.remove(&child);
        }

        if accounts.is_empty() {
            let row = adw::ActionRow::builder()
                .title("No accounts configured")
                .subtitle("Add an account to receive private notifications")
                .build();
            let icon = gtk::Image::builder()
                .icon_name("user-available-symbolic")
                .pixel_size(32)
                .margin_end(12)
                .build();
            row.add_prefix(&icon);
            imp.added_accounts.append(&row);
        } else {
            for a in accounts {
                let row = adw::ActionRow::builder()
                    .title(&a.server)
                    .subtitle(&a.username)
                    .build();
                row.add_css_class("property");

                // Icon
                let icon = gtk::Image::builder()
                    .icon_name("avatar-default-symbolic")
                    .build();
                row.add_prefix(&icon);


                // Details button
                row.add_suffix(&{
                    let btn = gtk::Button::builder()
                        .icon_name("dialog-information-symbolic")
                        .tooltip_text("View Details")
                        .valign(gtk::Align::Center)
                        .build();
                    btn.add_css_class("flat");
                    let server = a.server.clone();
                    let username = a.username.clone();
                    let this = self.clone();
                    btn.connect_clicked(move |_| {
                        let dialog = adw::AlertDialog::builder()
                            .heading("Account Details")
                            .body(format!("Server: {}\nUsername: {}", server, username))
                            .build();
                        dialog.add_response("ok", "OK");
                        dialog.present(Some(&this));
                    });
                    btn
                });

                // Edit button
                row.add_suffix(&{
                    let btn = gtk::Button::builder()
                        .icon_name("document-edit-symbolic")
                        .tooltip_text("Edit Account")
                        .valign(gtk::Align::Center)
                        .build();
                    btn.add_css_class("flat");
                    let this = self.clone();
                    let a = a.clone();
                    btn.connect_clicked(move |btn| {
                        let this = this.clone();
                        let a = a.clone();
                        btn.error_boundary()
                            .spawn(async move { this.on_edit_account_clicked(&a).await });
                    });
                    btn
                });

                // Remove button
                row.add_suffix(&{
                    let btn = gtk::Button::builder()
                        .icon_name("user-trash-symbolic")
                        .tooltip_text("Remove Account")
                        .valign(gtk::Align::Center)
                        .build();
                    btn.add_css_class("flat");
                    btn.add_css_class("error");
                    let this = self.clone();
                    let a = a.clone();
                    btn.connect_clicked(move |_| {
                         let this = this.clone();
                         let a = a.clone();
                         let dialog = adw::AlertDialog::builder()
                            .heading("Remove Account?")
                            .body(format!("Are you sure you want to remove the account for {}?", a.server))
                            .build();
                         dialog.add_response("cancel", "Cancel");
                         dialog.add_response("remove", "Remove");
                         dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
                         dialog.set_default_response(Some("cancel"));
                         dialog.set_close_response("cancel");
                         
                         let this_clone = this.clone();
                         dialog.choose(Some(&this), gio::Cancellable::NONE, move |result| {
                             if result == "remove" {
                                  glib::MainContext::default().spawn_local(async move {
                                       let _ = this_clone.remove_account(&a.server).await;
                                  });
                             }
                         });
                    });
                    btn
                });
                imp.added_accounts.append(&row);
            }
        }
        Ok(())
    }

    pub async fn on_add_account_clicked(&self) -> anyhow::Result<()> {
        let dialog = NtfyrAccountDialog::new();
        dialog.set_title("Add Account");

        let (sender, receiver) = async_channel::bounded(1);
        dialog.connect_closure(
            "save",
            false,
            glib::closure_local!(move |_dialog: NtfyrAccountDialog| {
                let _ = sender.send_blocking(());
            }),
        );

        dialog.present(Some(self));

        if let Ok(()) = receiver.recv().await {
            let (server, username, password) = dialog.account_data();
            let n = self.imp().notifier.get().unwrap();
            n.add_account(&server, &username, &password).await?;
            self.show_accounts().await?;
        }

        Ok(())
    }

    pub async fn on_edit_account_clicked(&self, account: &ntfy_daemon::models::Account) -> anyhow::Result<()> {
        let dialog = NtfyrAccountDialog::new();
        dialog.set_account(account);

        let (sender, receiver) = async_channel::bounded(1);
        dialog.connect_closure(
            "save",
            false,
            glib::closure_local!(move |_dialog: NtfyrAccountDialog| {
                let _ = sender.send_blocking(());
            }),
        );

        dialog.present(Some(self));

        if let Ok(()) = receiver.recv().await {
            let (server, username, password) = dialog.account_data();
            let n = self.imp().notifier.get().unwrap();
            // ntfy-daemon might not have an "update_account" but add_account
            // with same server should overwrite or we remove and add.
            // Let's check ntfy-daemon/src/ntfy.rs if possible or just try add_account.
            n.add_account(&server, &username, &password).await?;
            self.show_accounts().await?;
        }

        Ok(())
    }

    pub async fn remove_account(&self, server: &str) -> anyhow::Result<()> {
        self.imp()
            .notifier
            .get()
            .unwrap()
            .remove_account(server)
            .await?;
        self.show_accounts().await?;
        Ok(())
    }
    pub fn update_servers_ui(&self) {
        let settings = gio::Settings::new(crate::config::APP_ID);
        let custom_servers = settings.strv("custom-servers");
        let default_server = settings.string("default-server");
        let imp = self.imp();

        // Update ListBox
        while let Some(child) = imp.added_servers.last_child() {
            imp.added_servers.remove(&child);
        }
        
        // We'll use this group to group the radio buttons
        let mut group: Option<gtk::CheckButton> = None;

        // Add Default Server (ntfy.sh)
        {
            let row = adw::ActionRow::builder()
                  .title("https://ntfy.sh")
                  .build();
            
            // Radio Button
            let check = gtk::CheckButton::builder()
                .active(default_server == "https://ntfy.sh")
                .valign(gtk::Align::Center)
                .build();
            if let Some(g) = &group {
                check.set_group(Some(g));
            } else {
                group = Some(check.clone());
            }
            
            check.connect_toggled(move |btn| {
                 if btn.is_active() {
                      let settings = gio::Settings::new(crate::config::APP_ID);
                      let _ = settings.set_string("default-server", "https://ntfy.sh");
                 }
            });
            row.add_prefix(&check);
            
            let icon = gtk::Image::builder()
                .icon_name("io.github.tobagin.Ntfyr-ntfy-symbolic")
                .build();
            row.add_prefix(&icon);
             
            imp.added_servers.append(&row);
        }

        for server in custom_servers {
             let row = adw::ActionRow::builder()
                  .title(&*server)
                  .build();

             // Radio Button
             let check = gtk::CheckButton::builder()
                 .active(default_server == server)
                 .valign(gtk::Align::Center)
                 .build();
             if let Some(g) = &group {
                 check.set_group(Some(g));
             }
             
             let server_clone = server.clone();
             check.connect_toggled(move |btn| {
                  if btn.is_active() {
                       let settings = gio::Settings::new(crate::config::APP_ID);
                       let _ = settings.set_string("default-server", &server_clone);
                  }
             });
             row.add_prefix(&check);

             let icon = gtk::Image::builder()
                .icon_name("network-server-symbolic")
                .build();
             row.add_prefix(&icon);
             
             let btn = gtk::Button::builder()
                  .icon_name("user-trash-symbolic")
                  .tooltip_text("Remove Server")
                  .valign(gtk::Align::Center)
                  .build();
             btn.add_css_class("flat");
             btn.add_css_class("error");
             
             let this = self.clone();
             let s = server.clone();
             btn.connect_clicked(move |_| {
                  let this = this.clone();
                  let s = s.clone();
                  let dialog = adw::AlertDialog::builder()
                       .heading("Remove Server?")
                       .body(format!("Are you sure you want to remove {}?", s))
                       .build();
                  dialog.add_response("cancel", "Cancel");
                  dialog.add_response("remove", "Remove");
                  dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
                  dialog.set_default_response(Some("cancel"));
                  dialog.set_close_response("cancel");

                  let this_clone = this.clone();
                  dialog.choose(Some(&this), gio::Cancellable::NONE, move |result| {
                       if result == "remove" {
                            glib::MainContext::default().spawn_local(async move {
                                 this_clone.remove_server(&s);
                            });
                       }
                  });
             });
             
             row.add_suffix(&btn);
             imp.added_servers.append(&row);
        }
    }
    pub async fn on_add_server_clicked(&self) -> anyhow::Result<()> {
         let dialog = adw::AlertDialog::builder()
              .heading("Add Server")
              .body("Enter the URL of the custom server")
              .build();
         
         let entry = gtk::Entry::builder()
              .placeholder_text("https://example.com")
              .activates_default(true)
              .build();
         
         dialog.set_extra_child(Some(&entry));
         dialog.add_response("cancel", "Cancel");
         dialog.add_response("add", "Add");
         dialog.set_response_appearance("add", adw::ResponseAppearance::Suggested);
         dialog.set_default_response(Some("add"));
         dialog.set_close_response("cancel");

         let result = dialog.choose_future(Some(self)).await;
         
         if result == "add" {
              let text = entry.text();
              if !text.is_empty() {
                   let settings = gio::Settings::new(crate::config::APP_ID);
                   let mut servers: Vec<String> = settings.strv("custom-servers").into_iter().map(|s| s.to_string()).collect();
                   if !servers.contains(&text.to_string()) && text != "https://ntfy.sh" {
                        servers.push(text.to_string());
                        let _ = settings.set_strv("custom-servers", servers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice());
                   }
              }
         }
         Ok(())
    }

    pub fn remove_server(&self, server: &str) {
         let settings = gio::Settings::new(crate::config::APP_ID);
         let mut servers: Vec<String> = settings.strv("custom-servers").into_iter().map(|s| s.to_string()).collect();
         servers.retain(|s| s != server);
         let _ = settings.set_strv("custom-servers", servers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice());
    }
}
