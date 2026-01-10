use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;
use ntfy_daemon::models::{FilterRule, FilterAction};

mod imp {
    use super::*;

    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/filter_rule_dialog.ui")]
    #[properties(wrapper_type = super::FilterRuleDialog)]
    pub struct FilterRuleDialog {
        #[template_child]
        pub name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub regex_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub action_combo: TemplateChild<adw::ComboRow>,
        #[template_child]
        pub add_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub cancel_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FilterRuleDialog {
        const NAME: &'static str = "FilterRuleDialog";
        type Type = super::FilterRuleDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[glib::derived_properties]
    impl ObjectImpl for FilterRuleDialog {
        fn constructed(&self) {
            self.parent_constructed();
            let this = self.obj();
            
            // Connect signals
            let this_weak = this.downgrade();
            self.add_btn.connect_clicked(move |_| {
                if let Some(this) = this_weak.upgrade() {
                    this.emit_rule_added();
                    this.close();
                }
            });
            
            let this_weak = this.downgrade();
            self.cancel_btn.connect_clicked(move |_| {
                if let Some(this) = this_weak.upgrade() {
                    this.close();
                }
            });
        }
    }
    impl WidgetImpl for FilterRuleDialog {}
    impl AdwDialogImpl for FilterRuleDialog {}
}

glib::wrapper! {
    pub struct FilterRuleDialog(ObjectSubclass<imp::FilterRuleDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl FilterRuleDialog {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
    
    pub fn get_rule(&self) -> Option<FilterRule> {
        let imp = self.imp();
        let name = imp.name_entry.text().to_string();
        let regex = imp.regex_entry.text().to_string();
        
        if name.is_empty() || regex.is_empty() {
            return None;
        }

        let selected = imp.action_combo.selected();
        let action = match selected {
            0 => FilterAction::Mute,
            1 => FilterAction::Discard,
            2 => FilterAction::MarkRead,
            _ => FilterAction::Mute,
        };

        Some(FilterRule {
            name,
            regex,
            action,
        })
    }
    
    fn emit_rule_added(&self) {
        // Since we don't have a formal GSignal for this yet, we can use a closure/callback pattern
        // or standard GAction. For simplicity, we assume the caller will connect to "closed" 
        // and check get_rule(), but "closed" fires on cancel too.
        // Better: Caller passes a callback or we define a signal. 
        // Let's use simple GAction approach for now: Caller connects to button? No, template child is private.
        // We will expose a helper or signal.
        // For minimal implementation: We'll stick to `add_btn` signal being internal, 
        // and we simply expose an "response" signal? adw::Dialog has "closed".
        // Let's emit a proper signal "rule-added" using `full-glib-signals` would be best but requires more boilerplate.
        // Alternative: Pass a callback.
    }
}
