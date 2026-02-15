use adw::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};
use gtk::prelude::*;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/lock_view.ui")]
    pub struct LockView {
        #[template_child]
        pub unlock_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub password_entry: TemplateChild<gtk::PasswordEntry>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LockView {
        const NAME: &'static str = "LockView";
        type Type = super::LockView;
        type ParentType = adw::Bin;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LockView {}
    impl WidgetImpl for LockView {}
    impl BinImpl for LockView {}
}

glib::wrapper! {
    pub struct LockView(ObjectSubclass<imp::LockView>)
        @extends gtk::Widget, adw::Bin,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl LockView {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn password_text(&self) -> String {
        self.imp().password_entry.text().to_string()
    }

    pub fn clear_password(&self) {
        self.imp().password_entry.set_text("");
        self.imp().password_entry.remove_css_class("error");
    }

    pub fn show_error(&self) {
        let entry = &self.imp().password_entry;
        entry.add_css_class("error");
        // Select all text so user can easily retype
        entry.select_region(0, -1);
        entry.grab_focus();
    }
}
