use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;

use gtk::gio;
use gtk::glib;

use crate::error::*;

mod imp {
    pub use super::*;
    #[derive(Debug, Default, glib::Properties, gtk::CompositeTemplate)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/subscription_info_dialog.ui")]
    #[properties(wrapper_type = super::SubscriptionInfoDialog)]
    pub struct SubscriptionInfoDialog {
        #[property(get, construct_only)]
        pub subscription: RefCell<Option<crate::subscription::Subscription>>,
        #[template_child]
        pub display_name_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub muted_switch_row: TemplateChild<adw::SwitchRow>,
        
        // Schedule
        #[template_child]
        pub schedule_enabled_switch: TemplateChild<adw::SwitchRow>,
        #[template_child]
        pub schedule_start_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub schedule_end_entry: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub schedule_days_box: TemplateChild<gtk::Box>,

        // Rules
        #[template_child]
        pub rules_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub add_rule_btn: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SubscriptionInfoDialog {
        const NAME: &'static str = "SubscriptionInfoDialog";
        type Type = super::SubscriptionInfoDialog;
        type ParentType = adw::Dialog;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }
    #[glib::derived_properties]
    impl ObjectImpl for SubscriptionInfoDialog {
        fn constructed(&self) {
            self.parent_constructed();
            let this = self.obj().clone();

            let sub = this.subscription().unwrap();
            self.display_name_entry
                .set_text(&sub.display_name());
            self.muted_switch_row
                .set_active(sub.muted());
            
            // Init Schedule
            this.init_schedule_ui(&sub);
             // Init Rules
            this.init_rules_ui(&sub);

            let debouncer = crate::async_utils::Debouncer::new();
            self.display_name_entry.connect_changed({
                 let this = this.clone();
                 let debouncer = debouncer.clone();
                 move |entry| {
                    let entry = entry.clone();
                    let this = this.clone();
                    debouncer.call(std::time::Duration::from_millis(500), move || {
                        this.update_display_name(&entry);
                    })
                }
            });
            
            // Schedule Signals
            let this_weak = this.downgrade();
            self.schedule_enabled_switch.connect_active_notify(move |_| {
                if let Some(this) = this_weak.upgrade() {
                    this.update_schedule();
                }
            });
            let this_weak = this.downgrade();
            let debouncer_clone = debouncer.clone();
            self.schedule_start_entry.connect_changed(move |_| {
                let Some(this) = this_weak.upgrade() else { return; };
                let debouncer = debouncer_clone.clone();
                debouncer.call(std::time::Duration::from_millis(500), move || {
                    this.update_schedule();
                });
            });
            let this_weak = this.downgrade();
            let debouncer_clone = debouncer.clone();
            self.schedule_end_entry.connect_changed(move |_| {
                let Some(this) = this_weak.upgrade() else { return; };
                let debouncer = debouncer_clone.clone();
                debouncer.call(std::time::Duration::from_millis(500), move || {
                    this.update_schedule();
                });
            });
            
            // Bind day toggles
            let mut i = self.schedule_days_box.first_child();
            while let Some(child) = i {
                if let Some(btn) = child.downcast_ref::<gtk::ToggleButton>() {
                     let this_weak = this.downgrade();
                     btn.connect_toggled(move |_| {
                         if let Some(this) = this_weak.upgrade() {
                             this.update_schedule();
                         }
                     });
                }
                i = child.next_sibling();
            }

            // Rules Signals
            let this_weak = this.downgrade();
            self.add_rule_btn.connect_clicked(move |_| {
               if let Some(this) = this_weak.upgrade() {
                   this.show_add_rule_dialog(); 
               }
            });
            let this = self.obj().clone();
            self.muted_switch_row.connect_active_notify({
                move |switch| {
                    this.update_muted(switch);
                }
            });
        }
    }
    impl WidgetImpl for SubscriptionInfoDialog {}
    impl AdwDialogImpl for SubscriptionInfoDialog {}
}

glib::wrapper! {
    pub struct SubscriptionInfoDialog(ObjectSubclass<imp::SubscriptionInfoDialog>)
        @extends gtk::Widget, adw::Dialog,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

impl SubscriptionInfoDialog {
    pub fn new(subscription: crate::subscription::Subscription) -> Self {
        let this = glib::Object::builder()
            .property("subscription", subscription)
            .build();
        this
    }
    fn update_display_name(&self, entry: &impl IsA<gtk::Editable>) {
        if let Some(sub) = self.subscription() {
            let entry = entry.clone();
            self.error_boundary().spawn(async move {
                let res = sub.set_display_name(entry.text().to_string()).await;
                res
            });
        }
    }
    fn update_muted(&self, switch: &adw::SwitchRow) {
        if let Some(sub) = self.subscription() {
            let switch = switch.clone();
            self.error_boundary()
                .spawn(async move { sub.set_muted(switch.is_active()).await })
        }
    }

    // Map UI index (0=Mon...6=Sun) to Model day (0=Sun...6=Sat)
    fn ui_idx_to_model_day(idx: i32) -> u8 {
        ((idx + 1) % 7) as u8
    }

    fn model_day_to_ui_idx(day: u8) -> i32 {
        ((day + 6) % 7) as i32
    }

    fn init_schedule_ui(&self, sub: &crate::subscription::Subscription) {
        let imp = self.imp();
        if let Some(schedule) = sub.get_schedule() {
            imp.schedule_enabled_switch.set_active(true);
            imp.schedule_start_entry.set_text(&schedule.start_time);
            imp.schedule_end_entry.set_text(&schedule.end_time);
            
            let mut i = imp.schedule_days_box.first_child();
            let mut ui_idx = 0;
            while let Some(child) = i {
                if let Some(btn) = child.downcast_ref::<gtk::ToggleButton>() {
                     let model_day = Self::ui_idx_to_model_day(ui_idx);
                     btn.set_active(schedule.days.contains(&model_day));
                     ui_idx += 1;
                }
                i = child.next_sibling();
            }
        } else {
             imp.schedule_enabled_switch.set_active(false);
        }
    }

    fn update_schedule(&self) {
        let imp = self.imp();
        let enabled = imp.schedule_enabled_switch.is_active();
        let sub = self.subscription().unwrap();

        if !enabled {
            self.error_boundary().spawn(async move {
                sub.set_schedule(None).await
            });
            return;
        }

        let start = imp.schedule_start_entry.text();
        let end = imp.schedule_end_entry.text();
        
        let mut days = vec![];
        let mut i = imp.schedule_days_box.first_child();
        let mut ui_idx = 0;
        while let Some(child) = i {
            if let Some(btn) = child.downcast_ref::<gtk::ToggleButton>() {
                    if btn.is_active() {
                        days.push(Self::ui_idx_to_model_day(ui_idx));
                    }
                    ui_idx += 1;
            }
            i = child.next_sibling();
        }

        let schedule = ntfy_daemon::models::Schedule {
            start_time: start.to_string(),
            end_time: end.to_string(),
            days,
        };

        self.error_boundary().spawn(async move {
            sub.set_schedule(Some(schedule)).await
        });
    }

    fn init_rules_ui(&self, sub: &crate::subscription::Subscription) {
        let imp = self.imp();
        // Clear all rows from the ListBox
        let mut row = imp.rules_list.row_at_index(0);
        while let Some(r) = row {
            imp.rules_list.remove(&r);
            row = imp.rules_list.row_at_index(0);
        }

        if let Some(rules) = sub.get_rules() {
             for rule in rules {
                 self.add_rule_row(&rule);
             }
        }
    }

    fn add_rule_row(&self, rule: &ntfy_daemon::models::FilterRule) {
        let imp = self.imp();
        let row = adw::ActionRow::builder()
            .title(&rule.name)
            .subtitle(format!("Regex: {} -> Action: {:?}", rule.regex, rule.action))
            .build();
        
        // Add delete button
        let btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .valign(gtk::Align::Center)
            .css_classes(vec!["flat"])
            .build();
        
        let rule_clone = rule.clone();
        let this_weak = self.downgrade();
        btn.connect_clicked(move |_| {
             if let Some(this) = this_weak.upgrade() {
                this.delete_rule(&rule_clone);
             }
        });

        row.add_suffix(&btn);
        imp.rules_list.append(&row);
    }
    
    fn delete_rule(&self, rule_to_delete: &ntfy_daemon::models::FilterRule) {
         let sub = self.subscription().unwrap();
         if let Some(mut rules) = sub.get_rules() {
             rules.retain(|r| r.regex != rule_to_delete.regex || r.name != rule_to_delete.name);
             let this = self.clone();
             let sub_clone = sub.clone();
             self.error_boundary().spawn(async move {
                 let _ = sub_clone.set_rules(Some(rules)).await;
                 this.init_rules_ui(&sub_clone);
                 Ok::<(), anyhow::Error>(())
             });
         }
    }

    fn show_add_rule_dialog(&self) {
        let dialog = crate::widgets::filter_rule_dialog::FilterRuleDialog::new();
        let this_weak = self.downgrade();
        
        dialog.connect_closed(move |d| {
            if let Some(this) = this_weak.upgrade() {
                if let Some(rule) = d.get_rule() {
                     let sub = this.subscription().unwrap();
                     let mut rules = sub.get_rules().unwrap_or_default();
                     rules.push(rule);
                     
                     let sub_clone = sub.clone();
                     let this_clone = this.clone();
                     this.error_boundary().spawn(async move {
                         let _ = sub_clone.set_rules(Some(rules)).await;
                         this_clone.init_rules_ui(&sub_clone);
                         Ok::<(), anyhow::Error>(())
                     });
                }
            }
        });
        
        dialog.present(Some(self));
    }
        
// Old add_rule removed as it's now handled in the closure
}
