use std::cell::Cell;
use std::cell::OnceCell;

use adw::prelude::*;
use adw::subclass::prelude::*;

use gtk::{gio, glib};
use ntfy_daemon::models;
use ntfy_daemon::NtfyHandle;
use tracing::{info, warn};

use crate::application::NtfyrApplication;
use crate::config::{APP_ID, PROFILE};
use crate::error::*;
use anyhow::Result;
use crate::subscription::Status;
use crate::subscription::Subscription;
use crate::widgets::*;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/io/github/tobagin/Ntfyr/ui/window.ui")]
    pub struct NtfyrWindow {
        #[template_child]
        pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub message_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub subscription_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub main_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub lock_view: TemplateChild<LockView>,
        #[template_child]
        pub entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub navigation_split_view: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub subscription_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub subscription_menu_btn: TemplateChild<gtk::MenuButton>,
        pub subscription_list_model: gio::ListStore,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub welcome_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub list_view: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub message_scroll: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub banner: TemplateChild<adw::Banner>,
        #[template_child]
        pub send_btn: TemplateChild<gtk::Button>,
        #[template_child]
        pub code_btn: TemplateChild<gtk::Button>,
        
        // Unified Inbox
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub unified_inbox_view: TemplateChild<adw::ToolbarView>,
        #[template_child]
        pub unified_message_list: TemplateChild<gtk::ListBox>,

        pub notifier: OnceCell<NtfyHandle>,
        pub conn: OnceCell<gio::SocketConnection>,
        pub settings: gio::Settings,
        pub banner_binding: Cell<Option<(Subscription, glib::SignalHandlerId)>>,
        pub subscription_sorter: OnceCell<gtk::CustomSorter>,
        pub subscription_sort_model: OnceCell<gtk::SortListModel>,
        pub last_activity: Cell<std::time::Instant>,
    }

    impl Default for NtfyrWindow {
        fn default() -> Self {
            let this = Self {
                headerbar: Default::default(),
                message_list: Default::default(),
                entry: Default::default(),
                subscription_view: Default::default(),
                navigation_split_view: Default::default(),
                subscription_menu_btn: Default::default(),
                subscription_list: Default::default(),
                main_stack: Default::default(),
                lock_view: Default::default(),
                toast_overlay: Default::default(),
                stack: Default::default(),
                welcome_view: Default::default(),
                list_view: Default::default(),
                message_scroll: Default::default(),
                banner: Default::default(),
               content_stack: Default::default(),
                unified_inbox_view: Default::default(),
                unified_message_list: Default::default(),
                subscription_list_model: gio::ListStore::new::<Subscription>(),
                settings: gio::Settings::new(APP_ID),
                notifier: Default::default(),
                conn: Default::default(),
                banner_binding: Default::default(),
                send_btn: Default::default(),
                code_btn: Default::default(),
                subscription_sorter: Default::default(),
                subscription_sort_model: Default::default(),
                last_activity: Cell::new(std::time::Instant::now()),
            };

            this
        }
    }

    #[gtk::template_callbacks]
    impl NtfyrWindow {
        #[template_callback]
        fn show_add_topic(&self, _btn: &gtk::Button) {
            let this = self.obj().clone();
            let settings = gio::Settings::new(crate::config::APP_ID);
            let default_server = settings.string("default-server");
            let server = this.selected_subscription()
                .map(|x| x.server())
                .unwrap_or(default_server.to_string());
            let dialog = AddSubscriptionDialog::new(server);
            dialog.present(Some(&self.obj().clone()));

            let dc = dialog.clone();
            dialog.connect_local("subscribe-request", true, move |_| {
                let sub = match dc.subscription() {
                    Ok(sub) => sub,
                    Err(e) => {
                        warn!(errors = ?e, "trying to add invalid subscription");
                        return None;
                    }
                };
                this.add_subscription(sub);
                dc.close();
                None
            });
        }
        #[template_callback]
        fn show_add_server(&self, _btn: &gtk::Button) {
            self.obj().on_add_server_clicked();
        }
        #[template_callback]
        fn discover_integrations(&self, _btn: &gtk::Button) {
            gtk::UriLauncher::new("https://docs.ntfy.sh/integrations/").launch(
                Some(&self.obj().clone()),
                gio::Cancellable::NONE,
                |_| {},
            );
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NtfyrWindow {
        const NAME: &'static str = "NtfyrWindow";
        type Type = super::NtfyrWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks();

            klass.install_action("win.unsubscribe", None, |this, _, _| {
                this.unsubscribe();
            });
            klass.install_action("win.show-subscription-info", None, |this, _, _| {
                this.show_subscription_info();
            });
            klass.install_action("win.clear-notifications", None, |this, _, _| {
                this.selected_subscription().map(|sub| {
                    this.error_boundary()
                        .spawn(async move { sub.clear_notifications().await });
                });
            });
            klass.install_action("win.add-topic", None, |this, _, _| {
                this.imp().show_add_topic(&gtk::Button::new());
            });
            klass.install_action("win.search", None, |this, _, _| {
                this.imp().entry.grab_focus();
            });
            //klass.bind_template_instance_callbacks();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for NtfyrWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Setup Lock Screen
            obj.setup_app_lock();
            obj.setup_auto_lock();
        }
    }

    impl WidgetImpl for NtfyrWindow {}
    impl WindowImpl for NtfyrWindow {
        // Save window state on delete event
        fn close_request(&self) -> glib::Propagation {
            if let Err(err) = self.obj().save_window_size() {
                warn!(error = %err, "Failed to save window state");
            }

            // Instead of closing, hide the window
            self.obj().set_visible(false);
            glib::Propagation::Stop
        }
    }

    impl ApplicationWindowImpl for NtfyrWindow {}
    impl AdwApplicationWindowImpl for NtfyrWindow {}
}

glib::wrapper! {
    pub struct NtfyrWindow(ObjectSubclass<imp::NtfyrWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root, gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::ShortcutManager;
}

#[allow(deprecated)]
impl NtfyrWindow {
    pub fn new(app: &NtfyrApplication, notifier: NtfyHandle) -> Self {
        let obj: Self = glib::Object::builder().property("application", app).build();

        if let Err(_) = obj.imp().notifier.set(notifier) {
            panic!("setting notifier for first time");
        };

        // Load latest window state
        obj.load_window_size();
        
        obj.bind_message_list();
        obj.connect_entry_and_send_btn();
        obj.connect_code_btn();
        obj.connect_items_changed();
        obj.connect_settings_changed();
        obj.connect_server_changes();
        obj.selected_subscription_changed(None);
        obj.bind_flag_read();

        obj
    }
    fn connect_entry_and_send_btn(&self) {
        let imp = self.imp();
        let this = self.clone();

        imp.entry.connect_activate(move |_| this.publish_msg());
        let this = self.clone();
        imp.send_btn.connect_clicked(move |_| this.publish_msg());
    }
    fn publish_msg(&self) {
        let entry = self.imp().entry.clone();
        let this = self.clone();

        entry.error_boundary().spawn(async move {
            this.selected_subscription()
                .unwrap()
                .publish_msg(models::OutgoingMessage {
                    message: Some(entry.text().as_str().to_string()),
                    ..models::OutgoingMessage::default()
                }, false)
                .await?;
            entry.set_text("");
            Ok(())
        });
    }
    fn connect_code_btn(&self) {
        let imp = self.imp();
        let this = self.clone();
        imp.code_btn.connect_clicked(move |_| {
            let this = this.clone();
            this.selected_subscription().map(move |sub| {
                AdvancedMessageDialog::new(sub, this.imp().entry.text().to_string())
                    .present(Some(&this))
            });
        });
    }
    fn show_subscription_info(&self) {
        let sub = SubscriptionInfoDialog::new(self.selected_subscription().unwrap());
        sub.present(Some(self));
    }
    fn connect_items_changed(&self) {
        let this = self.clone();
        self.imp()
            .subscription_list_model
            .connect_items_changed(move |list, _, _, _| {
                let imp = this.imp();
                // Content stack: show welcome_view if no topics, otherwise show subscription_view
                if list.n_items() == 0 {
                    imp.content_stack.set_visible_child(&*imp.welcome_view);
                } else {
                    imp.content_stack.set_visible_child(&*imp.subscription_view);
                }
            });
    }

    fn connect_settings_changed(&self) {
        let settings = &self.imp().settings;
        let this = self.clone();
        settings.connect_changed(Some("sort-descending"), move |_, _| {
            this.selected_subscription_changed(this.selected_subscription().as_ref());
        });
    }

    fn connect_server_changes(&self) {
        let settings = &self.imp().settings;
        let this = self.clone();
        settings.connect_changed(Some("custom-servers"), move |_, _| {
            // Trigger a re-sort when servers change
            if let Some(sorter) = this.imp().subscription_sorter.get() {
                sorter.changed(gtk::SorterChange::Different);
            }
            // Rebuild the list to show new/removed servers
            this.rebuild_subscription_list();
        });
        let this = self.clone();
        settings.connect_changed(Some("show-default-server"), move |_, _| {
             this.rebuild_subscription_list();
        });
    }

    fn add_subscription(&self, sub: models::Subscription) {
        let this = self.clone();
        self.error_boundary().spawn(async move {
            let sub = this.notifier().subscribe(&sub.server, &sub.topic).await?;
            let imp = this.imp();

            // Subscription::new will use the pipelined client to retrieve info about the subscription
            let subscription = Subscription::new(sub);
            // We want to still check if there were any errors adding the subscription.

            this.attach_sort_trigger(&subscription);
            
            imp.subscription_list_model.append(&subscription);
            
            // Wait for info to load
            glib::timeout_future_seconds(1).await;
            
            // Rebuild the UI list
            this.rebuild_subscription_list();
            
            // TODO: Select the newly added subscription? 
            // For now let's just ensure it appears.
            
            Ok(())
        });
    }

    fn unsubscribe(&self) {
        let Some(sub) = self.selected_subscription() else {
            return;
        };

        let this = self.clone();
        self.error_boundary().spawn(async move {
            if let Err(e) = this.notifier()
                .unsubscribe(sub.server().as_str(), sub.topic().as_str())
                .await 
            {
                warn!("Failed to unsubscribe from backend: {}", e);
            }

            let imp = this.imp();
            if let Some(i) = imp.subscription_list_model.find(&sub) {
                imp.subscription_list_model.remove(i);
                
                // Rebuild the UI list
                this.rebuild_subscription_list();
                
                // Clear selection if needed
                let n_items = imp.subscription_list_model.n_items();
                if n_items == 0 {
                    this.selected_subscription_changed(None);
                }
            }
            Ok(())
        });
    }

    pub fn purge_default_server_topics(&self) {
        let this = self.clone();
        
        self.error_boundary().spawn(async move {
            info!("Starting purge of default server topics");
            let imp = this.imp();
            let mut to_remove = Vec::new();

            // Collect subscriptions to remove from the UI model
            for i in 0..imp.subscription_list_model.n_items() {
                if let Some(sub) = imp.subscription_list_model.item(i).and_downcast::<Subscription>() {
                    if sub.server() == "https://ntfy.sh" {
                        info!("Found topic to remove: {}", sub.topic());
                        to_remove.push(sub);
                    }
                }
            }

            info!("Purging {} topics from default server", to_remove.len());

            for sub in to_remove {
                info!("Purging topic: {}", sub.topic());
                
                // Get the subscription handle to delete messages
                if let Some(handle) = sub.imp().client.get() {
                    // Delete all messages for this topic
                    if let Err(e) = handle.clear_notifications().await {
                        warn!("Failed to clear notifications for {}: {}", sub.topic(), e);
                    } else {
                        info!("Cleared notifications for {}", sub.topic());
                    }
                }
                
                // Unsubscribe from the daemon (removes from DB and stops listener)
                if let Err(e) = this.notifier()
                    .unsubscribe(sub.server().as_str(), sub.topic().as_str())
                    .await 
                {
                    warn!("Failed to unsubscribe {}: {}", sub.topic(), e);
                } else {
                    info!("Successfully unsubscribed from {}", sub.topic());
                }
                
                // Remove from model if present
                if let Some(i) = imp.subscription_list_model.find(&sub) {
                    info!("Removing {} from model at index {}", sub.topic(), i);
                    imp.subscription_list_model.remove(i);
                } else {
                    warn!("Could not find {} in model to remove", sub.topic());
                }
            }
            
            info!("Purge complete, rebuilding UI");
            
            // Trigger UI rebuild
            this.rebuild_subscription_list();
            
            // Clear selection if needed
            if imp.subscription_list_model.n_items() == 0 {
                this.selected_subscription_changed(None);
            }
            
            info!("Default server topics purged successfully");
            
            Ok::<_, anyhow::Error>(())
        });
    }

    pub fn notifier(&self) -> &NtfyHandle {
        self.imp().notifier.get().unwrap()
    }
    fn selected_subscription(&self) -> Option<Subscription> {
        let imp = self.imp();
        let row = imp.subscription_list.selected_row()?;
        
        // Get topic and server from the row data
        let topic = unsafe { row.data::<String>("topic")?.as_ref().clone() };
        let server = unsafe { row.data::<String>("server")?.as_ref().clone() };
        
        // Find the matching subscription in the model
        if let Some(sort_model) = imp.subscription_sort_model.get() {
            for i in 0..sort_model.n_items() {
                if let Some(sub) = sort_model.item(i).and_downcast::<Subscription>() {
                    if sub.topic() == topic && sub.server() == server {
                        return Some(sub);
                    }
                }
            }
        }
        None
    }
    fn bind_unified_inbox(&self) {
        let imp = self.imp();
        let map_model = gtk::MapListModel::new(Some(imp.subscription_list_model.clone()), |item| {
            let sub = item.downcast_ref::<Subscription>().unwrap();
            println!("UnifiedInbox: Mapping subscription {}", sub.topic());
            sub.imp().messages.clone().upcast::<glib::Object>()
        });
        
        let flatten_model = gtk::FlattenListModel::new(Some(map_model));
        
        let sort_descending = imp.settings.boolean("sort-descending");
        let sorter = gtk::CustomSorter::new(move |a, b| {
                let a = a.downcast_ref::<glib::BoxedAnyObject>().unwrap();
                let a = a.borrow::<models::ReceivedMessage>();
                let b = b.downcast_ref::<glib::BoxedAnyObject>().unwrap();
                let b = b.borrow::<models::ReceivedMessage>();

                let time_a = a.time;
                let time_b = b.time;

                if sort_descending {
                    time_b.cmp(&time_a).into()
                } else {
                    time_a.cmp(&time_b).into()
                }
        });

        let sorter: gtk::Sorter = sorter.upcast(); 
        let sort_model = gtk::SortListModel::new(Some(flatten_model), Some(sorter));
        
        imp.unified_message_list.bind_model(Some(&sort_model), |obj| {
             let b = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
             let msg = b.borrow::<models::ReceivedMessage>();
             MessageRow::new(msg.clone()).upcast()
        });
        
        // Unified inbox selection is handled in subscription_list row_activated
    }

    fn bind_message_list(&self) {
        let imp = self.imp();
        
        self.bind_unified_inbox();

        let sorter = gtk::CustomSorter::new(|a, b| {
            let a = a.downcast_ref::<Subscription>().unwrap();
            let b = b.downcast_ref::<Subscription>().unwrap();

            let server_a = a.server();
            let server_b = b.server();

            let server_cmp = server_a.cmp(&server_b);
            if server_cmp != std::cmp::Ordering::Equal {
                 return server_cmp.into();
            }
            
            a.topic().cmp(&b.topic()).into()
        });
        
        imp.subscription_sorter.set(sorter.clone()).unwrap();

        let sort_model = gtk::SortListModel::new(Some(imp.subscription_list_model.clone()), Some(sorter));
        let _ = imp.subscription_sort_model.set(sort_model.clone());

        // NO header function - we create a flat list with server ActionRows directly
        
        let this = self.clone();
        imp.subscription_list.connect_row_activated(move |_list, row| {
            let imp = this.imp();
            
            // Check if unified inbox row
            let is_inbox = unsafe { row.data::<bool>("unified-inbox").is_some() };
            if is_inbox {
                this.selected_subscription_changed(None);
                imp.content_stack.set_visible_child(&*imp.unified_inbox_view);
                imp.navigation_split_view.set_show_content(true);
                return;
            }
            
            // Check if server row or placeholder - ignore these
            let is_server = unsafe { row.data::<bool>("server-row").is_some() };
            let is_placeholder = unsafe { row.data::<bool>("placeholder").is_some() };
            if is_server || is_placeholder {
                return;
            }
            
            // Topic row - find the subscription by topic+server
            let topic = unsafe { row.data::<String>("topic").map(|s| s.as_ref().clone()) };
            let server = unsafe { row.data::<String>("server").map(|s| s.as_ref().clone()) };
            
            if let (Some(topic), Some(server)) = (topic, server) {
                // Find the subscription in the model
                if let Some(sort_model) = imp.subscription_sort_model.get() {
                    for i in 0..sort_model.n_items() {
                        if let Some(sub) = sort_model.item(i).and_downcast::<Subscription>() {
                            if sub.topic() == topic && sub.server() == server {
                                this.selected_subscription_changed(Some(&sub));
                                imp.content_stack.set_visible_child(&*imp.subscription_view);
                                imp.navigation_split_view.set_show_content(true);
                                return;
                            }
                        }
                    }
                }
            }
        });

        // Initial population (empty servers only at first)
        self.rebuild_subscription_list();
        
        // Load subscriptions asynchronously
        let this = self.clone();
        self.error_boundary().spawn(async move {
            glib::timeout_future_seconds(1).await;
            let list = this.notifier().list_subscriptions().await?;
            for sub in list {
                let sub = Subscription::new(sub);
                this.attach_sort_trigger(&sub);
                this.imp().subscription_list_model.append(&sub);
            }
            // Wait a bit for subscriptions to load their info (async)
            glib::timeout_future_seconds(1).await;
            // Rebuild list with actual subscriptions
            this.rebuild_subscription_list();
            Ok::<_, anyhow::Error>(())
        });
    }

    fn rebuild_subscription_list(&self) {
        let imp = self.imp();
        let list = &imp.subscription_list;
        
        // Clear existing rows
        while let Some(child) = list.first_child() {
            list.remove(&child.downcast::<gtk::ListBoxRow>().unwrap());
        }
        
        // 1. Unified Inbox - ActionRow directly in ListBox (ActionRow IS a ListBoxRow)
        let inbox = adw::ActionRow::builder()
            .subtitle("Unified Inbox")
            .icon_name("mail-read-symbolic")
            .activatable(true)
            .build();
        unsafe { inbox.set_data("unified-inbox", true); }
        list.append(&inbox);
        
        // Get all servers (ntfy.sh + custom servers)
        let settings = gio::Settings::new(crate::config::APP_ID);
        let mut all_servers = Vec::new();

        if settings.boolean("show-default-server") {
             all_servers.push("https://ntfy.sh".to_string());
        }

        all_servers.extend(
            settings.strv("custom-servers")
                .into_iter()
                .map(|s| s.to_string())
        );
        
        // Group subscriptions by server
        let mut subs_by_server: std::collections::HashMap<String, Vec<Subscription>> = std::collections::HashMap::new();
        if let Some(sort_model) = imp.subscription_sort_model.get() {
            for i in 0..sort_model.n_items() {
                if let Some(sub) = sort_model.item(i).and_downcast::<Subscription>() {
                    subs_by_server
                        .entry(sub.server())
                        .or_insert_with(Vec::new)
                        .push(sub);
                }
            }
        }
        
        // 2. For each server: server ActionRow, then topics or placeholder
        for server in &all_servers {
            // Server ActionRow - directly in ListBox
            let server_row = self.build_server_action_row(server);
            list.append(&server_row);
            
            // Topics or placeholder
            if let Some(subs) = subs_by_server.get(server) {
                for sub in subs {
                    let topic_row = Self::build_topic_action_row(sub);
                    list.append(&topic_row);
                }
            } else {
                let placeholder = self.build_placeholder_action_row();
                list.append(&placeholder);
            }
        }
    }

    fn build_server_action_row(&self, server: &str) -> adw::ActionRow {
        let icon_name = if server == "https://ntfy.sh" {
            "io.github.tobagin.Ntfyr-ntfy-symbolic"
        } else {
            "network-server-symbolic"
        };
        
        // Adw.ActionRow { subtitle, icon-name, selectable: false, styles ["background"] }
        let action_row = adw::ActionRow::builder()
            .subtitle(server)
            .icon_name(icon_name)
            .selectable(false)
            .build();
        action_row.add_css_class("background");
        
        // Gtk.Box { hexpand: true; halign: end; styles ["linked"] }
        let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        button_box.set_hexpand(true);
        button_box.set_halign(gtk::Align::End);
        button_box.add_css_class("linked");
        
        // MenuButton { icon-name: view-more-symbolic, tooltip: Server Actions, flat }
        let menu_btn = gtk::MenuButton::builder()
            .icon_name("view-more-symbolic")
            .tooltip_text("Server Actions")
            .valign(gtk::Align::Center)
            .build();
        menu_btn.add_css_class("flat");

        let popover = gtk::Popover::new();
        let menu_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        menu_box.add_css_class("boxed-list");
        menu_box.set_margin_top(6);
        menu_box.set_margin_bottom(6);
        menu_box.set_margin_start(6);
        menu_box.set_margin_end(6);
        menu_box.set_spacing(0); // Connected list look

        // Helper to create styled menu rows
        let create_menu_row = |label: &str, icon_name: &str| -> gtk::Button {
            let btn = gtk::Button::builder()
                .halign(gtk::Align::Fill)
                .build();
            btn.add_css_class("flat");
            
            let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let icon = gtk::Image::from_icon_name(icon_name);
            let lbl = gtk::Label::builder()
                .label(label)
                .xalign(0.0)
                .hexpand(true)
                .build();
            
            box_.append(&icon);
            box_.append(&lbl);
            btn.set_child(Some(&box_));
            btn
        };

        // Add Topic Item
        let add_topic_btn = create_menu_row("Add Topic", "list-add-symbolic");
        let server_clone = server.to_string();
        let popover_clone = popover.clone();
        add_topic_btn.connect_clicked(move |btn| {
            popover_clone.popdown();
            if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                window.show_add_topic_for_server(&server_clone);
            }
        });
        menu_box.append(&add_topic_btn);

        // Add Account Item
        let add_account_btn = create_menu_row("Add Account", "contact-new-symbolic");
        let server_clone = server.to_string();
        let popover_clone = popover.clone();
            add_account_btn.connect_clicked(move |btn| {
                popover_clone.popdown();
                if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                    window.on_add_account_clicked(&server_clone);
                }
            });
        menu_box.append(&add_account_btn);

        // Remove Server Item (only custom)
        if server != "https://ntfy.sh" {
            let remove_btn = create_menu_row("Remove Server", "user-trash-symbolic");
            remove_btn.add_css_class("destructive-action");

            let server_clone = server.to_string();
            let popover_clone = popover.clone();
            remove_btn.connect_clicked(move |btn| {
                popover_clone.popdown();
                if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                    window.on_remove_server_clicked(&server_clone);
                }
            });
            menu_box.append(&remove_btn);
        } else {
             let hide_btn = create_menu_row("Hide Server", "view-hidden-symbolic");
             hide_btn.add_css_class("destructive-action"); // Optional: style it destructively or normally
 
             let popover_clone = popover.clone();
             hide_btn.connect_clicked(move |btn| {
                 popover_clone.popdown();
                 if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                     window.on_hide_server_clicked();
                 }
             });
             menu_box.append(&hide_btn);
        }

        popover.set_child(Some(&menu_box));
        menu_btn.set_popover(Some(&popover));
        button_box.append(&menu_btn);
        
        action_row.add_suffix(&button_box);
        unsafe { action_row.set_data("server-row", true); }
        action_row
    }

    fn build_placeholder_action_row(&self) -> adw::ActionRow {
        // Adw.ActionRow { subtitle, icon-name, selectable: false }
        let action_row = adw::ActionRow::builder()
            .subtitle("no topics added")
            .icon_name("mail-mark-important-symbolic")
            .selectable(false)
            .build();
        unsafe { action_row.set_data("placeholder", true); }
        action_row
    }

    fn build_empty_server_row(&self, server: &str) -> gtk::ListBoxRow {
        let icon_name = if server == "https://ntfy.sh" {
            "io.github.tobagin.Ntfyr-ntfy-symbolic"
        } else {
            "network-server-symbolic"
        };
        
        let action_row = adw::ActionRow::builder()
            .title(server)
            .subtitle("No topics added, add your first topic")
            .icon_name(icon_name)
            .activatable(false)
            .build();
        
        // Create linked button group
        let button_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(0)
            .valign(gtk::Align::Center)
            .build();
        button_box.add_css_class("linked");
        
        // Add Topic button
        let add_topic_btn = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add Topic")
            .build();
        
        let server_clone = server.to_string();
        add_topic_btn.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                window.show_add_topic_for_server(&server_clone);
            }
        });
        button_box.append(&add_topic_btn);
        
        // Add Account button
        let add_account_btn = gtk::Button::builder()
            .icon_name("contact-new-symbolic")
            .tooltip_text("Add Account")
            .build();
        
        let server_clone = server.to_string();
        add_account_btn.connect_clicked(move |btn| {
            if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                window.on_add_account_clicked(&server_clone);
            }
        });
        button_box.append(&add_account_btn);
        
        // Remove Server button (only for custom servers, not ntfy.sh)
        if server != "https://ntfy.sh" {
            let remove_server_btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Remove Server")
                .build();
            remove_server_btn.add_css_class("destructive-action");
            
            let server_clone = server.to_string();
            remove_server_btn.connect_clicked(move |btn| {
                if let Some(window) = btn.root().and_downcast::<NtfyrWindow>() {
                    window.on_remove_server_clicked(&server_clone);
                }
            });
            button_box.append(&remove_server_btn);
        }
        
        action_row.add_suffix(&button_box);
        
        let row = gtk::ListBoxRow::builder()
            .activatable(false)
            .selectable(false)
            .build();
        row.set_child(Some(&action_row));
        unsafe { row.set_data("empty-server", true); }
        row
    }


    fn attach_sort_trigger(&self, sub: &Subscription) {
        let imp = self.imp();
        let sorter = imp.subscription_sorter.get().unwrap().clone();
        
        let trigger_sort = move |_: &Subscription, _: &glib::ParamSpec| {
            sorter.changed(gtk::SorterChange::Different);
        };
        
        sub.connect_notify_local(Some("server"), trigger_sort.clone());
        sub.connect_notify_local(Some("topic"), trigger_sort);
    }
    fn update_banner(&self, sub: Option<&Subscription>) {
        let imp = self.imp();
        if let Some(sub) = sub {
            match sub.nice_status() {
                Status::Degraded | Status::Down => imp.banner.set_revealed(true),
                Status::Up => imp.banner.set_revealed(false),
            }
        } else {
            imp.banner.set_revealed(false);
        }
    }
    fn selected_subscription_changed(&self, sub: Option<&Subscription>) {
        let imp = self.imp();
        self.update_banner(sub);
        let this = self.clone();
        let set_sensitive = move |b| {
            let imp = this.imp();
            imp.subscription_menu_btn.set_visible(b);
            imp.code_btn.set_sensitive(b);
            imp.send_btn.set_sensitive(b);
            imp.entry.set_sensitive(b);
        };
        if let Some((sub, id)) = imp.banner_binding.take() {
            sub.disconnect(id);
        }
        if let Some(sub) = sub {
            set_sensitive(true);
            imp.navigation_split_view.set_show_content(true);

            let sort_descending = imp.settings.boolean("sort-descending");
            let sorter = gtk::CustomSorter::new(move |a, b| {
                let a = a.downcast_ref::<glib::BoxedAnyObject>().unwrap();
                let a = a.borrow::<models::ReceivedMessage>();
                let b = b.downcast_ref::<glib::BoxedAnyObject>().unwrap();
                let b = b.borrow::<models::ReceivedMessage>();

                let time_a = a.time;
                let time_b = b.time;

                if sort_descending {
                    time_b.cmp(&time_a).into()
                } else {
                    time_a.cmp(&time_b).into()
                }
            });

            let sort_model = gtk::SortListModel::new(Some(sub.imp().messages.clone()), Some(sorter));

            imp.message_list
                .bind_model(Some(&sort_model), move |obj| {
                    let b = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
                    let msg = b.borrow::<models::ReceivedMessage>();

                    MessageRow::new(msg.clone()).upcast()
                });

            let this = self.clone();
            imp.banner_binding.set(Some((
                sub.clone(),
                sub.connect_status_notify(move |sub| {
                    this.update_banner(Some(sub));
                }),
            )));

            let this = self.clone();
            glib::idle_add_local_once(move || {
                this.flag_read();
            });
        } else {
            set_sensitive(false);
            imp.message_list
                .bind_model(gio::ListModel::NONE, |_| adw::Bin::new().into());
        }
    }
    fn flag_read(&self) {
        let vadj = self.imp().message_scroll.vadjustment();
        // There is nothing to scroll, so the user viewed all the messages
        if vadj.page_size() == vadj.upper()
            || ((vadj.page_size() + vadj.value() - vadj.upper()).abs() <= 1.0)
        {
            self.selected_subscription().map(|sub| {
                self.error_boundary()
                    .spawn(async move { sub.flag_all_as_read().await });
            });
        }
    }

    fn build_topic_action_row(sub: &Subscription) -> adw::ActionRow {
        // Adw.ActionRow { title, icon-name, Gtk.Box { icons } }
        let action_row = adw::ActionRow::builder()
            .icon_name("lang-include-symbolic")
            .activatable(true)
            .build();
        
        // Bind title to display-name
        sub.bind_property("display-name", &action_row, "title")
            .sync_create()
            .build();
        
        // Gtk.Box { icons }
        let icon_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        // Gtk.Image { icon-name: "alarm-symbolic" } - schedule
        let schedule = gtk::Image::new();
        schedule.set_icon_name(Some("alarm-symbolic"));
        schedule.set_visible(false);
        sub.bind_property("has-schedule", &schedule, "visible").sync_create().build();
        icon_box.append(&schedule);

        // Gtk.Image { icon-name: "edit-find-replace-symbolic" } - filters
        let filter = gtk::Image::new();
        filter.set_icon_name(Some("edit-find-replace-symbolic"));
        filter.set_visible(false);
        sub.bind_property("has-rules", &filter, "visible").sync_create().build();
        icon_box.append(&filter);

        // Gtk.Image { network-error-symbolic / network-cellular-signal-weak-symbolic } - status
        let status = gtk::Image::new();
        status.set_visible(false);
        let status_clone = status.clone();
        sub.connect_status_notify(move |sub| match sub.nice_status() {
             Status::Down => {
                 status_clone.set_icon_name(Some("network-error-symbolic"));
                 status_clone.set_visible(true);
             }
             Status::Degraded => {
                 status_clone.set_icon_name(Some("network-cellular-signal-weak-symbolic"));
                 status_clone.set_visible(true);
             }
             _ => status_clone.set_visible(false),
        });
        icon_box.append(&status);

        // Gtk.Image { icon-name: "notifications-disabled-symbolic" } - muted
        let muted = gtk::Image::new();
        muted.set_icon_name(Some("notifications-disabled-symbolic"));
        muted.set_visible(false);
        sub.bind_property("muted", &muted, "visible").sync_create().build();
        icon_box.append(&muted);

        // Gtk.Image { icon-name: "channel-secure-symbolic" } - reserved
        let reserved = gtk::Image::new();
        reserved.set_icon_name(Some("channel-secure-symbolic"));
        reserved.set_visible(false);
        sub.bind_property("reserved", &reserved, "visible").sync_create().build();
        icon_box.append(&reserved);

        // Gtk.Label { valign: center; margin-start: 5; label } - unread count
        let badge = gtk::Label::new(None);
        badge.set_valign(gtk::Align::Center);
        badge.set_margin_start(5);
        badge.set_visible(false);
        let badge_clone = badge.clone();
        sub.connect_unread_count_notify(move |sub| {
             let c = sub.unread_count();
             badge_clone.set_label(&c.to_string());
             badge_clone.set_visible(c > 0);
        });
        icon_box.append(&badge);

        action_row.add_suffix(&icon_box);
        
        // Store topic and server for lookup when clicked
        unsafe { 
            action_row.set_data("topic", sub.topic());
            action_row.set_data("server", sub.server());
        }
        
        action_row
    }

    pub fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }
    fn bind_flag_read(&self) {
        let imp = self.imp();

        let this = self.clone();
        imp.message_scroll.connect_edge_reached(move |_, pos_type| {
            if pos_type == gtk::PositionType::Bottom {
                this.flag_read();
            }
        });
        let this = self.clone();
        self.connect_is_active_notify(move |_| {
            if this.is_active() {
                this.flag_read();
            }
        });
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    // Server Management Methods
    pub fn on_add_server_clicked(&self) {
        let dialog = AddServerDialog::new();
        dialog.present(Some(self));
        
        let this = self.clone();
        let dialog_clone = dialog.clone();
        dialog.connect_local("add-request", true, move |_| {
             let url = dialog_clone.server_url();
             
             // Check if server already exists
             let settings = gio::Settings::new(crate::config::APP_ID);
             let mut servers: Vec<String> = settings.strv("custom-servers").into_iter().map(|s| s.to_string()).collect();
             
             if !servers.contains(&url) && url != "https://ntfy.sh" {
                 servers.push(url);
                 let _ = settings.set_strv("custom-servers", servers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice());
                 
                 let toast = adw::Toast::new("Server added successfully");
                 this.imp().toast_overlay.add_toast(toast);
             } else if servers.contains(&url) {
                  let toast = adw::Toast::new("Server already exists");
                  this.imp().toast_overlay.add_toast(toast);
             }
             
             dialog_clone.close();
             None
        });
    }

    async fn validate_ntfy_server(&self, url: &str) -> anyhow::Result<bool> {
        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Ok(false);
        }
        
        // Try to make a simple request to check if server is reachable
        let health_url = format!("{}/v1/health", url.trim_end_matches('/'));
        let url_clone = url.to_string();
        
        let result = tokio::task::spawn_blocking(move || {
            let agent = ureq::Agent::new_with_config(Default::default());
            
            // Try health endpoint first (most ntfy servers have this)
            if let Ok(_) = agent.get(&health_url).call() {
                return true;
            }
            
            // Fallback: try root endpoint - if it responds at all, consider it valid
            if let Ok(_) = agent.get(&url_clone).call() {
                return true;
            }
            
            false
        }).await?;
        
        Ok(result)
    }

    pub fn on_remove_server_clicked(&self, server: &str) {
        let server = server.to_string();
        let this = self.clone();
        let dialog = adw::AlertDialog::builder()
            .heading("Remove Server?")
            .body(format!("Are you sure you want to remove {}?\n\nAll subscriptions for this server will also be removed.", server))
            .build();
        dialog.add_response("cancel", "Cancel");
        dialog.add_response("remove", "Remove");
        dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
        dialog.set_default_response(Some("cancel"));
        dialog.set_close_response("cancel");

        dialog.choose(Some(self), gio::Cancellable::NONE, move |result| {
            if result == "remove" {
                this.remove_server(&server);
            }
        });
    }

    pub fn remove_server(&self, server: &str) {
        let settings = gio::Settings::new(crate::config::APP_ID);
        let mut servers: Vec<String> = settings.strv("custom-servers").into_iter().map(|s| s.to_string()).collect();
        servers.retain(|s| s != server);
        let _ = settings.set_strv("custom-servers", servers.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice());
    }

    pub fn on_hide_server_clicked(&self) {
        let settings = gio::Settings::new(crate::config::APP_ID);
        let _ = settings.set_boolean("show-default-server", false);
        
        // Also show a toast so user knows how to bring it back
        let toast = adw::Toast::new("Default server hidden. You can restore it in Preferences.");
        self.imp().toast_overlay.add_toast(toast);
    }

    pub fn show_add_topic_for_server(&self, server: &str) {
        let dialog = AddSubscriptionDialog::new(server.to_string());
        dialog.present(Some(self));

        let this = self.clone();
        let dc = dialog.clone();
        dialog.connect_local("subscribe-request", true, move |_| {
            let sub = match dc.subscription() {
                Ok(sub) => sub,
                Err(e) => {
                    warn!(errors = ?e, "trying to add invalid subscription");
                    return None;
                }
            };
            this.add_subscription(sub);
            dc.close();
            None
        });
    }

    pub fn on_add_account_clicked(&self, server: &str) {
        let dialog = NtfyrAccountDialog::new(server.to_string());
        dialog.present(Some(self));

        let this = self.clone();
        dialog.connect_closure(
            "save",
            false,
            glib::closure_local!(move |dialog: NtfyrAccountDialog| {
                let (server, username, password) = dialog.account_data();
                let this = this.clone();
                this.error_boundary().spawn(async move {
                    let n = this.notifier();
                    n.add_account(&server, &username, &password).await?;
                    let toast = adw::Toast::new("Account added successfully");
                    this.imp().toast_overlay.add_toast(toast);
                    Ok(())
                });
            }),
        );
    }

    fn setup_app_lock(&self) {
        let imp = self.imp();
        let is_locked = imp.settings.boolean("app-lock-enabled");
        
        if is_locked {
            imp.main_stack.set_visible_child(&*imp.lock_view);
            
            let this = self.clone();
            imp.lock_view.imp().unlock_button.connect_clicked(move |_| {
                this.request_unlock();
            });
            
            let this = self.clone();
            imp.lock_view.imp().password_entry.connect_activate(move |_| {
                this.request_unlock();
            });
        } else {
            imp.main_stack.set_visible_child(&*imp.navigation_split_view);
        }
    }

    fn setup_auto_lock(&self) {
        // Event Controller capturing all events to reset timer
        let controller = gtk::EventControllerLegacy::new();
        let this = self.clone();
        controller.connect_event(move |_, _| {
             this.imp().last_activity.set(std::time::Instant::now());
             glib::Propagation::Proceed
        });
        self.add_controller(controller);

        // Idle check loop
        let this = self.clone();
        glib::MainContext::default().spawn_local(async move {
             loop {
                 glib::timeout_future_seconds(10).await;
                 
                 let imp = this.imp();
                 let settings = &imp.settings;
                 
                 // Check if auto-lock is enabled
                 if !settings.boolean("auto-lock-enabled") {
                     continue;
                 }
                 
                 // Check if already locked
                 if imp.main_stack.visible_child().map(|w| w == *imp.lock_view).unwrap_or(false) {
                     continue;
                 }

                 let timeout_secs = settings.int("lock-timeout") as u64;
                 let elapsed = imp.last_activity.get().elapsed().as_secs();
                 
                 if elapsed >= timeout_secs {
                     info!("Auto-lock timeout reached ({}s), locking app.", elapsed);
                     this.lock_app();
                 }
             }
        });
    }

    pub fn lock_app(&self) {
        let imp = self.imp();
         // If app lock is enabled generally, switch to lock view
         if imp.settings.boolean("app-lock-enabled") {
             imp.main_stack.set_visible_child(&*imp.lock_view);
         }
    }

    fn request_unlock(&self) {
        let imp = self.imp();
        // Access via public method on wrapper, not imp()
        let entry_text = imp.lock_view.password_text();
        
        let this = self.clone();
        self.error_boundary().spawn(async move {
            let stored = crate::secrets::get_password().await.unwrap_or_else(|e| {
                warn!("Failed to get password: {}", e);
                None
            });
            
            let unlocked = if let Some(stored_pass) = stored {
                if entry_text == stored_pass {
                    true
                } else {
                    false
                }
            } else {
                // No password set, allow unlock but warn
                warn!("App lock enabled but no password set.");
                true
            };

            if unlocked {
                info!("Authentication successful");
                glib::MainContext::default().spawn_local(async move {
                    let imp = this.imp();
                    imp.lock_view.clear_password(); // Clear
                    imp.main_stack.set_visible_child(&*imp.navigation_split_view);
                    
                    if entry_text.is_empty() { 
                         let toast = adw::Toast::new("Warning: No password set for App Lock.");
                         this.imp().toast_overlay.add_toast(toast);
                    }
                });
            } else {
                warn!("Authentication failed");
                glib::MainContext::default().spawn_local(async move {
                     let toast = adw::Toast::new("Incorrect password");
                     this.imp().toast_overlay.add_toast(toast);
                     this.imp().lock_view.show_error();
                });
            }
            Ok(())
        });
    }

}

