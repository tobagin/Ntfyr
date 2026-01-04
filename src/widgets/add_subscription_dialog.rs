use std::cell::OnceCell;
use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::gio;
use gtk::glib;
use ntfy_daemon::models;
use once_cell::sync::Lazy;

#[derive(Default, Debug, Clone)]
pub struct Widgets {
    pub topic_entry: adw::EntryRow,
    pub server_combo: adw::ComboRow,
    pub server_entry: adw::EntryRow,
    pub sub_btn: gtk::Button,
}
mod imp {
    pub use super::*;
    #[derive(Debug, Default)]
    pub struct AddSubscriptionDialog {
        pub widgets: RefCell<Widgets>,
        pub init_custom_server: OnceCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddSubscriptionDialog {
        const NAME: &'static str = "AddSubscriptionDialog";
        type Type = super::AddSubscriptionDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("default.activate", None, |this, _, _| {
                this.emit_subscribe_request();
            });
        }
    }

    impl ObjectImpl for AddSubscriptionDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("subscribe-request").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for AddSubscriptionDialog {}
    impl AdwDialogImpl for AddSubscriptionDialog {}
}

glib::wrapper! {
    pub struct AddSubscriptionDialog(ObjectSubclass<imp::AddSubscriptionDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl AddSubscriptionDialog {
    pub fn new(custom_server: Option<String>) -> Self {
        let this: Self = glib::Object::builder().build();
        if let Some(s) = custom_server {
            if s != ntfy_daemon::models::DEFAULT_SERVER {
                this.imp().init_custom_server.set(s).unwrap();
            }
        }
        this.build_ui();
        this
    }
    fn build_ui(&self) {
        let imp = self.imp();
        let obj = self.clone();
        obj.set_title("Subscribe To Topic");

        let settings = gio::Settings::new(crate::config::APP_ID);
        let custom_servers = settings.strv("custom-servers");
        let default_server = settings.string("default-server");

        let model = gtk::StringList::new(&[]);
        model.append("https://ntfy.sh");
        for server in &custom_servers {
            model.append(server);
        }
        model.append("Custom...");

        let mut selected_idx = 0;
        if let Some(s) = imp.init_custom_server.get() {
            // If initialized with a custom server (e.g. from deep link?), try to find it
            // If not found, select "Custom..." and fill entry later? 
            // Logic: if provided custom server is in list, select it. Else select "Custom..."
             let mut found = false;
             for i in 0..model.n_items() {
                  if let Some(item) = model.string(i) {
                       if item == *s {
                            selected_idx = i;
                            found = true;
                            break;
                       }
                  }
             }
             if !found {
                  selected_idx = model.n_items() - 1; // Custom...
             }
        } else {
             // Default to preference
             for i in 0..model.n_items() {
                  if let Some(item) = model.string(i) {
                       if item == default_server {
                            selected_idx = i;
                            break;
                       }
                  }
             }
        }


        relm4_macros::view! {
            toolbar_view = adw::ToolbarView {
                add_top_bar: &adw::HeaderBar::new(),
                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_end: 12,
                    set_margin_start: 12,
                    set_margin_top: 12,
                    set_margin_bottom: 12,
                    append = &gtk::Label {
                        add_css_class: "dim-label",
                        set_label: "Topics may not be password-protected, so choose a name that's not easy to guess. \
                            Once subscribed, you can PUT/POST notifications.",
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_wrap_mode: gtk::pango::WrapMode::WordChar
                    },
                    append = &gtk::ListBox {
                        add_css_class: "boxed-list",
                        append: topic_entry = &adw::EntryRow {
                            set_title: "Topic",
                            set_activates_default: true,
                            add_suffix = &gtk::Button {
                                set_icon_name: "dice3-symbolic",
                                set_tooltip_text: Some("Generate name"),
                                set_valign: gtk::Align::Center,
                                add_css_class: "flat",
                                connect_clicked[topic_entry] => move |_| {
                                    use rand::distributions::Alphanumeric;
                                    use rand::{thread_rng, Rng};
                                    let mut rng = thread_rng();
                                    let chars: String = (0..10).map(|_| rng.sample(Alphanumeric) as char).collect();
                                    topic_entry.set_text(&chars);
                                }
                            }
                        },
                        append: server_combo = &adw::ComboRow {
                            set_title: "Server",
                            set_model: Some(&model),
                            set_selected: selected_idx,
                        },
                        append: server_entry = &adw::EntryRow {
                            set_title: "Custom Server URL",
                            set_visible: false, // Initially hidden, logic below updates it
                            set_text: imp.init_custom_server.get().map(|x| x.as_str()).unwrap_or(""),
                        }
                    },
                    append: sub_btn = &gtk::Button {
                        set_label: "Subscribe",
                        add_css_class: "suggested-action",
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        set_sensitive: false,
                        connect_clicked[obj] => move |_| {
                            obj.emit_subscribe_request();
                        }
                    }
                },
            },
        }

        // Logic to toggle server_entry visibility
        let combo = server_combo.clone();
        let entry = server_entry.clone();
        let combo_notify = move || {
             let model = combo.model().and_downcast::<gtk::StringList>().unwrap();
             let selected = combo.selected();
             let is_custom = selected == model.n_items() - 1; // Last item is "Custom..."
             entry.set_visible(is_custom);
        };
        // Run once
        combo_notify();
        // Connect signal
        let f = combo_notify.clone();
        server_combo.connect_selected_notify(move |_| f());


        let debounced_error_check = {
            let db = crate::async_utils::Debouncer::new();
            let objc = obj.clone();
            move || {
                db.call(std::time::Duration::from_millis(500), move || {
                    objc.check_errors()
                });
            }
        };

        let f = debounced_error_check.clone();
        topic_entry
            .delegate()
            .unwrap()
            .connect_changed(move |_| f.clone()());
        let f = debounced_error_check.clone();
        server_entry
            .delegate()
            .unwrap()
            .connect_changed(move |_| f.clone()());
        let f = debounced_error_check.clone();
        server_combo.connect_selected_notify(move |_| f.clone()());

        imp.widgets.replace(Widgets {
            topic_entry,
            server_combo,
            server_entry,
            sub_btn,
        });

        obj.set_content_width(480);
        obj.set_child(Some(&toolbar_view));
    }
    pub fn subscription(&self) -> Result<models::Subscription, ntfy_daemon::Error> {
        let w = { self.imp().widgets.borrow().clone() };
        let mut sub = models::Subscription::builder(w.topic_entry.text().to_string());
        
        // Get selected server from combo
        if let Some(model) = w.server_combo.model().and_downcast::<gtk::StringList>() {
             let selected = w.server_combo.selected();
             // Last item is "Custom..."
             if selected == model.n_items() - 1 {
                  sub = sub.server(w.server_entry.text().to_string());
             } else if let Some(s) = model.string(selected) {
                  // Explicitly set server even if default, to be safe
                  sub = sub.server(s.to_string());
             }
        }
        
        sub.build()
    }
    fn check_errors(&self) {
        let w = { self.imp().widgets.borrow().clone() };
        let sub = self.subscription();

        w.server_entry.remove_css_class("error");
        w.server_combo.remove_css_class("error");
        w.topic_entry.remove_css_class("error");
        w.sub_btn.set_sensitive(true);

        if let Err(ntfy_daemon::Error::InvalidSubscription(errs)) = sub {
            w.sub_btn.set_sensitive(false);
            for e in errs {
                match e {
                    ntfy_daemon::Error::InvalidTopic(_) => {
                        w.topic_entry.add_css_class("error");
                    }
                    ntfy_daemon::Error::InvalidServer(_) => {
                         if let Some(model) = w.server_combo.model().and_downcast::<gtk::StringList>() {
                              if w.server_combo.selected() == model.n_items() - 1 {
                                   w.server_entry.add_css_class("error");
                              } else {
                                   w.server_combo.add_css_class("error");
                              }
                         }
                    }
                    _ => {}
                }
            }
        }
    }
    fn emit_subscribe_request(&self) {
        self.emit_by_name::<()>("subscribe-request", &[]);
    }
}
