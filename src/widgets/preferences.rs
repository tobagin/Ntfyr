use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
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
        pub notifier: OnceCell<NtfyHandle>,
    }

    impl Default for NtfyrPreferences {
        fn default() -> Self {
            let this = Self {
                startup_switch: Default::default(),
                sort_descending_switch: Default::default(),
                startup_background_switch: Default::default(),

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
        obj
    }


}
