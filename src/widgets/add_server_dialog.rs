use std::cell::RefCell;
use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::subclass::Signal;
use gtk::gio;
use gtk::glib;
use once_cell::sync::Lazy;

#[derive(Default, Debug, Clone)]
pub struct Widgets {
    pub server_entry: adw::EntryRow,
    pub add_btn: gtk::Button,
}

mod imp {
    pub use super::*;
    #[derive(Debug, Default)]
    pub struct AddServerDialog {
        pub widgets: RefCell<Widgets>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AddServerDialog {
        const NAME: &'static str = "AddServerDialog";
        type Type = super::AddServerDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.install_action("default.activate", None, |this, _, _| {
                this.emit_add_request();
            });
        }
    }

    impl ObjectImpl for AddServerDialog {
        fn signals() -> &'static [Signal] {
            static SIGNALS: Lazy<Vec<Signal>> =
                Lazy::new(|| vec![Signal::builder("add-request").build()]);
            SIGNALS.as_ref()
        }
    }
    impl WidgetImpl for AddServerDialog {}
    impl AdwDialogImpl for AddServerDialog {}
}

glib::wrapper! {
    pub struct AddServerDialog(ObjectSubclass<imp::AddServerDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl AddServerDialog {
    pub fn new() -> Self {
        let this: Self = glib::Object::builder().build();
        this.build_ui();
        this
    }

    fn build_ui(&self) {
        let imp = self.imp();
        let obj = self.clone();
        obj.set_title("Add Server");

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
                        set_label: "Enter the URL of your self-hosted ntfy server.",
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_halign: gtk::Align::Center,
                    },
                    append = &gtk::ListBox {
                        add_css_class: "boxed-list",
                        append: server_entry = &adw::EntryRow {
                            set_title: "Server URL",
                            set_activates_default: true,
                            set_input_purpose: gtk::InputPurpose::Url,
                        },
                    },
                    append: add_btn = &gtk::Button {
                        set_label: "Add Server",
                        add_css_class: "suggested-action",
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        set_sensitive: false,
                        connect_clicked[obj] => move |_| {
                            obj.emit_add_request();
                        }
                    }
                },
            },
        }

        let debounced_check = {
            let db = crate::async_utils::Debouncer::new();
            let objc = obj.clone();
            move || {
                db.call(std::time::Duration::from_millis(100), move || {
                    objc.check_input()
                });
            }
        };

        let f = debounced_check.clone();
        server_entry
            .delegate()
            .unwrap()
            .connect_changed(move |_| f.clone()());
        
        // Initial check
        debounced_check();

        imp.widgets.replace(Widgets {
            server_entry,
            add_btn,
        });

        obj.set_content_width(400);
        obj.set_child(Some(&toolbar_view));
    }

    pub fn server_url(&self) -> String {
        self.imp().widgets.borrow().server_entry.text().to_string()
    }

    fn check_input(&self) {
        let w = self.imp().widgets.borrow();
        let text = w.server_entry.text();
        let is_valid = text.starts_with("http://") || text.starts_with("https://");
        
        w.add_btn.set_sensitive(is_valid);
        if !text.is_empty() && !is_valid {
             w.server_entry.add_css_class("error");
        } else {
             w.server_entry.remove_css_class("error");
        }
    }

    fn emit_add_request(&self) {
        self.emit_by_name::<()>("add-request", &[]);
    }
}
