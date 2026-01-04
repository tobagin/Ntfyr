use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use ntfy_daemon::models::Account;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate, Default)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/account_dialog.ui")]
    pub struct NtfyrAccountDialog {
        #[template_child]
        pub server_row: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub username_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub password_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub save_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NtfyrAccountDialog {
        const NAME: &'static str = "NtfyrAccountDialog";
        type Type = super::NtfyrAccountDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NtfyrAccountDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("save").build()]);
            SIGNALS.as_ref()
        }
    }

    impl WidgetImpl for NtfyrAccountDialog {}
    impl AdwDialogImpl for NtfyrAccountDialog {}

    #[gtk::template_callbacks]
    impl NtfyrAccountDialog {
        #[template_callback]
        fn on_cancel_clicked(&self) {
            self.obj().close();
        }

        #[template_callback]
        fn on_save_clicked(&self) {
            self.obj().emit_by_name::<()>("save", &[]);
            self.obj().close();
        }
    }
}

glib::wrapper! {
    pub struct NtfyrAccountDialog(ObjectSubclass<imp::NtfyrAccountDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl NtfyrAccountDialog {
    pub fn new() -> Self {
        let obj: Self = glib::Object::builder().build();
        
        // Populate servers
        let settings = gio::Settings::new(crate::config::APP_ID);
        let custom_servers = settings.strv("custom-servers");
        
        let model = gtk::StringList::new(&[]);
        model.append("https://ntfy.sh");
        for server in custom_servers {
            model.append(&server);
        }
        
        obj.imp().server_row.set_model(Some(&model));
        
        obj
    }

    pub fn set_account(&self, account: &Account) {
        let imp = self.imp();
        
        // Select correct server
        if let Some(model) = imp.server_row.model().and_downcast::<gtk::StringList>() {
             for i in 0..model.n_items() {
                  if let Some(s) = model.string(i) {
                       if s == account.server {
                            imp.server_row.set_selected(i);
                            break;
                       }
                  }
             }
        }
        
        imp.username_entry.set_text(&account.username);
        // We don't have the password in the Account struct usually, should we?
        // ntfy-daemon/src/models.rs says:
        // pub struct Account { pub server: String, pub username: String, }
        // The password is in the keyring.
        
        // For editing, we might need a way to pass the password if we want to show it,
        // but often passwords aren't shown during edit.
        
        // If we are editing, we probably want the "Add" button to say "Save".
        imp.save_btn.set_label("Save");
        self.set_title("Edit Account");
        
        // Server should probably be read-only when editing? 
        // With combo row, we can just disable it.
        imp.server_row.set_sensitive(false);
    }

    pub fn account_data(&self) -> (String, String, String) {
        let imp = self.imp();
        
        let selected = imp.server_row.selected();
        let server = if let Some(model) = imp.server_row.model().and_downcast::<gtk::StringList>() {
             model.string(selected).unwrap_or_else(|| "https://ntfy.sh".into())
        } else {
             "https://ntfy.sh".into()
        };

        (
            server.into(),
            imp.username_entry.text().to_string(),
            imp.password_entry.text().to_string(),
        )
    }
}
