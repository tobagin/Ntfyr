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
        pub username_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub password_entry: TemplateChild<adw::PasswordEntryRow>,
        #[template_child]
        pub save_btn: TemplateChild<gtk::Button>,
        pub server_url: once_cell::sync::OnceCell<String>,
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
    pub fn new(server_url: String) -> Self {
        let obj: Self = glib::Object::builder().build();
        obj.imp().server_url.set(server_url).unwrap();
        obj
    }

    pub fn set_account(&self, account: &Account) {
        let imp = self.imp();
        
        // Server context is already set via new() or we should verify matches?
        // Ideally set_account is used when editing existing account. 
        // If we want to support editing, we might need to ensure server matches or update it?
        // But for now, we assume dialog is opened for a specific server context.
        
        imp.username_entry.set_text(&account.username);
        imp.save_btn.set_label("Save");
        self.set_title("Edit Account");
    }

    pub fn account_data(&self) -> (String, String, String) {
        let imp = self.imp();
        
        let server = imp.server_url.get().map(|s| s.as_str()).unwrap_or("https://ntfy.sh");

        (
            server.into(),
            imp.username_entry.text().to_string(),
            imp.password_entry.text().to_string(),
        )
    }
}
