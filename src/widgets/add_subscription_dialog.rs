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
    pub sub_btn: gtk::Button,
}
mod imp {
    pub use super::*;
    #[derive(Debug, Default)]
    pub struct AddSubscriptionDialog {
        pub widgets: RefCell<Widgets>,
        pub server_url: OnceCell<String>,
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
    pub fn new(server_url: String) -> Self {
        let this: Self = glib::Object::builder().build();
        this.imp().server_url.set(server_url).unwrap();
        this.build_ui();
        this
    }
    fn build_ui(&self) {
        let imp = self.imp();
        let obj = self.clone();
        let server_url = imp.server_url.get().unwrap();
        
        obj.set_title("Subscribe To Topic");

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
                        set_label: &format!("Subscribing to topic on {}", server_url),
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_halign: gtk::Align::Center,
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

        let debounced_error_check = {
            let db = crate::async_utils::Debouncer::new();
            let objc = obj.clone();
            move || {
                db.call(std::time::Duration::from_millis(100), move || {
                    objc.check_errors()
                });
            }
        };

        let f = debounced_error_check.clone();
        topic_entry
            .delegate()
            .unwrap()
            .connect_changed(move |_| f.clone()());

        // Initial check
        debounced_error_check();
        
        // Mock server widget for struct compatibility, using RefCell
        // Since we removed them from UI, we still need them in struct if we keep struct same?
        // Wait, I can update the struct definition in replacement content too!
        // But the struct definition is at top of file, so let's update that separate or included?
        // The targeted range (EndLine: 278) covers 'imp' module end but NOT 'Widgets' struct definition at line 12.
        // So I must update Widgets struct separately or accept unused fields if I just change the build_ui logic.
        // Actually, let's redefine Widgets in imp logic or update it first.
        
        // For now, I will populate the struct with dummy widgets or just remove them from struct in a separate call?
        // Let's update `Widgets` struct first or here? 
        // I cannot easily update line 12-18 and 22-278 in one go if I don't replace whole file.
        // Let's replace the whole file content for safety and correctness.
        
        imp.widgets.replace(Widgets {
            topic_entry,
            sub_btn,
        });

        obj.set_content_width(400);
        obj.set_child(Some(&toolbar_view));
    }
    
    pub fn subscription(&self) -> Result<models::Subscription, ntfy_daemon::Error> {
        let w = { self.imp().widgets.borrow().clone() };
        let server = self.imp().server_url.get().unwrap();
        
        models::Subscription::builder(w.topic_entry.text().to_string())
            .server(server.clone())
            .build()
    }
    
    fn check_errors(&self) {
        let w = { self.imp().widgets.borrow().clone() };
        let sub = self.subscription();

        w.topic_entry.remove_css_class("error");
        w.sub_btn.set_sensitive(true);

        if let Err(ntfy_daemon::Error::InvalidSubscription(errs)) = sub {
            w.sub_btn.set_sensitive(false);
            for e in errs {
                if let ntfy_daemon::Error::InvalidTopic(_) = e {
                     w.topic_entry.add_css_class("error");
                }
            }
        }
    }
    fn emit_subscribe_request(&self) {
        self.emit_by_name::<()>("subscribe-request", &[]);
    }
}
