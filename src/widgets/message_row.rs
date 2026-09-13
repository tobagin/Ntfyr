

use std::net::{IpAddr, ToSocketAddrs};
use std::rc::Rc;

use adw::prelude::*;
use adw::subclass::prelude::*;

use gettextrs::gettext;
use gtk::{gdk, gio, glib};
use crate::config::APP_ID;
use gdk_pixbuf;
use ntfy_daemon::models;
use pulldown_cmark::{Event, Parser, Tag, TagEnd};
use tracing::error;

use crate::error::*;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MessageRow {}

    #[glib::object_subclass]
    impl ObjectSubclass for MessageRow {
        const NAME: &'static str = "MessageRow";
        type Type = super::MessageRow;
        type ParentType = gtk::Grid;
    }

    impl ObjectImpl for MessageRow {}

    impl WidgetImpl for MessageRow {}
    impl GridImpl for MessageRow {}
}

fn themed_notification_icon(unseen: bool) -> &'static str {
    let Some(display) = gdk::Display::default() else {
        return "dialog-information-symbolic";
    };
    let theme = gtk::IconTheme::for_display(&display);
    let candidates: &[&str] = if unseen {
        &["alarm-symbolic", "preferences-system-notifications-symbolic", "notification-symbolic", "dialog-information-symbolic"]
    } else {
        &["notifications-disabled-symbolic", "preferences-system-notifications-symbolic", "dialog-information-symbolic"]
    };
    candidates.iter().copied().find(|name| theme.has_icon(name)).unwrap_or("dialog-information-symbolic")
}

fn markdown_image_urls(markdown: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut rest = markdown;
    while let Some(start) = rest.find("![") {
        let Some(open) = rest[start..].find("](") else { break };
        let url_start = start + open + 2;
        let Some(end) = rest[url_start..].find(')') else { break };
        let url = &rest[url_start..url_start + end];
        if url.starts_with("http://") || url.starts_with("https://") {
            urls.push(url.to_string());
        }
        rest = &rest[url_start + end + 1..];
    }
    urls
}

fn is_blocked_image_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_multicast()
                || ip.is_broadcast()
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            let is_link_local = segments[0] & 0xffc0 == 0xfe80;
            let is_unique_local = segments[0] & 0xfe00 == 0xfc00;
            let mapped_v4 = ip.to_ipv4_mapped();

            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_multicast()
                || is_link_local
                || is_unique_local
                || mapped_v4.is_some_and(|ip| is_blocked_image_ip(IpAddr::V4(ip)))
        }
    }
}

fn validate_image_url(raw_url: &str) -> anyhow::Result<()> {
    let url = url::Url::parse(raw_url)?;
    if !matches!(url.scheme(), "http" | "https") {
        anyhow::bail!("unsupported image URL scheme");
    }
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("image URL has no host"))?;
    let port = url
        .port_or_known_default()
        .ok_or_else(|| anyhow::anyhow!("image URL has no port"))?;
    let addresses: Vec<_> = (host, port).to_socket_addrs()?.collect();
    if addresses.is_empty() || addresses.iter().any(|address| is_blocked_image_ip(address.ip())) {
        anyhow::bail!("image URL resolves to a blocked address");
    }
    Ok(())
}

fn markdown_to_pango(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut pango = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Paragraph) => {
                if !pango.is_empty() { pango.push('\n'); }
            }
            Event::Start(Tag::Heading { .. }) => pango.push_str("<b>"),
            Event::End(TagEnd::Heading(_)) => pango.push_str("</b>\n"),
            Event::Start(Tag::Strong) => pango.push_str("<b>"),
            Event::End(TagEnd::Strong) => pango.push_str("</b>"),
            Event::Start(Tag::Emphasis) => pango.push_str("<i>"),
            Event::End(TagEnd::Emphasis) => pango.push_str("</i>"),
            Event::Start(Tag::Strikethrough) => pango.push_str("<s>"),
            Event::End(TagEnd::Strikethrough) => pango.push_str("</s>"),
            Event::Start(Tag::Link { dest_url, .. }) => {
                pango.push_str(&format!(
                    "<a href=\"{}\">",
                    glib::markup_escape_text(&dest_url)
                ));
            }
            Event::End(TagEnd::Link) => pango.push_str("</a>"),
            Event::Start(Tag::List(_)) => {}
            Event::End(TagEnd::List(_)) => pango.push('\n'),
            Event::Start(Tag::Item) => pango.push_str("• "),
            Event::End(TagEnd::Item) => pango.push('\n'),
            Event::Start(Tag::BlockQuote(_)) => pango.push_str("<i>│ "),
            Event::End(TagEnd::BlockQuote(_)) => pango.push_str("</i>\n"),
            Event::Start(Tag::CodeBlock(_)) => pango.push_str("<tt>"),
            Event::End(TagEnd::CodeBlock) => pango.push_str("</tt>\n"),
            Event::Code(code) => {
                pango.push_str("<tt>");
                pango.push_str(&glib::markup_escape_text(&code));
                pango.push_str("</tt>");
            }
            Event::Text(text) => pango.push_str(&glib::markup_escape_text(&text)),
            Event::SoftBreak | Event::HardBreak => pango.push('\n'),
            _ => {}
        }
    }

    pango
}

glib::wrapper! {
    pub struct MessageRow(ObjectSubclass<imp::MessageRow>)
        @extends gtk::Widget, gtk::Grid,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MessageRow {
    pub fn new(msg: models::ReceivedMessage, unseen: bool, on_delete: impl Fn() + 'static, on_seen: impl Fn() + 'static) -> Self {
        let this: Self = glib::Object::new();
        this.build_ui(msg, unseen, on_delete, on_seen);
        this
    }
    fn build_ui(&self, msg: models::ReceivedMessage, unseen: bool, on_delete: impl Fn() + 'static, on_seen: impl Fn() + 'static) {
        self.set_margin_top(8);
        self.set_margin_bottom(8);
        self.set_margin_start(12);
        self.set_margin_end(12);
        self.add_css_class("message-card");
        if unseen {
            self.add_css_class("message-unseen");
        }
        let settings = gio::Settings::new(APP_ID);
        if settings.boolean("follow-accent-color") {
            self.add_css_class("follow-accent");
        }
        let row_for_settings = self.clone();
        settings.connect_changed(Some("follow-accent-color"), move |settings, _| {
            if settings.boolean("follow-accent-color") {
                row_for_settings.add_css_class("follow-accent");
            } else {
                row_for_settings.remove_css_class("follow-accent");
            }
        });
        self.set_hexpand(true);
        self.set_can_target(true);
        self.set_column_spacing(8);
        self.set_row_spacing(8);
        let on_seen: Rc<dyn Fn()> = Rc::new(on_seen);
        let seen_row = self.clone();
        let seen_gesture = gtk::GestureClick::new();
        seen_gesture.connect_pressed(move |_, _, _, _| {
            seen_row.remove_css_class("message-unseen");
            (on_seen)();
        });
        self.add_controller(seen_gesture);
        let mut row = 0;

        let datetime_format = settings.string("datetime-format");
        let time = gtk::Label::builder()
            .label(
                &chrono::DateTime::from_timestamp(msg.time as i64, 0)
                    .map(|time| {
                        let time = time.with_timezone(&chrono::Local);
                        time.format(datetime_format.as_str()).to_string()
                    })
                    .unwrap_or_default(),
            )
            .xalign(0.0)
            .build();
        time.add_css_class("caption");

        // ntfy delivers an edited notification as a new message linked by
        // sequence_id; flag it so the replacement isn't mistaken for the original.
        let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        time_box.append(&time);
        if msg.sequence_id.is_some() {
            let edited = gtk::Label::builder().label(gettext("Edited")).build();
            edited.add_css_class("caption");
            edited.add_css_class("chip");
            edited.set_valign(gtk::Align::Center);
            time_box.append(&edited);
        }
        time_box.add_css_class("message-footer-date");
        time_box.set_halign(gtk::Align::Start);
        time_box.set_valign(gtk::Align::End);
        let header_actions = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        header_actions.set_halign(gtk::Align::End);
        header_actions.set_valign(gtk::Align::End);
        header_actions.set_margin_end(8);

        if let Some(p) = msg.priority {
            let level = match p {
                5 => gettext("Max"),
                4 => gettext("High"),
                3 => gettext("Medium"),
                2 => gettext("Low"),
                1 => gettext("Min"),
                _ => gettext("Invalid"),
            };
            let text = gettext("Priority: {}").replacen("{}", &level, 1);
            let priority = gtk::Label::builder().label(&text).build();
            priority.add_css_class("caption");
            priority.add_css_class("priority-badge");
            priority.add_css_class(&format!("priority-{}", p.clamp(1, 5)));
            self.add_css_class(&format!("message-priority-{}", p.clamp(1, 5)));
            time_box.append(&priority);
        }

        let icon_name = themed_notification_icon(unseen);
        let unseen_icon = gtk::Image::from_icon_name(icon_name);
        let unseen_tooltip = if unseen { gettext("Unread notification") } else { gettext("Seen notification") };
        unseen_icon.set_tooltip_text(Some(&unseen_tooltip));
        unseen_icon.add_css_class("unseen-indicator");
        unseen_icon.set_opacity(if unseen { 1.0 } else { 0.35 });
        header_actions.append(&unseen_icon);

        let delete_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .tooltip_text(gettext("Delete notification"))
            .css_classes(vec!["flat", "circular"])
            .build();
        delete_btn.connect_clicked(move |_| on_delete());
        header_actions.append(&delete_btn);
        if let Some(title) = msg.display_title() {
            let label = gtk::Label::builder()
                .label(&title)
                .wrap_mode(gtk::pango::WrapMode::WordChar)
                .xalign(0.0)
                .wrap(true)
                .selectable(true)
                .build();
            label.add_css_class("heading");
            label.add_css_class("message-title");
            self.attach(&label, 0, row, 4, 1);
            row += 1;
        }

        if let Some(message) = msg.display_message() {
            let label = gtk::Label::builder()
                .wrap_mode(gtk::pango::WrapMode::WordChar)
                .xalign(0.0)
                .wrap(true)
                .selectable(true)
                .hexpand(true)
                .build();
            label.add_css_class("message-body");
            if msg.is_markdown() {
                label.set_use_markup(true);
                label.set_markup(&markdown_to_pango(&message));
            } else {
                label.set_label(&message);
            }
            self.attach(&label, 0, row, 4, 1);
            row += 1;

            // Markdown image syntax is rendered below the text as a real GTK picture.
            for url in markdown_image_urls(&message) {
                self.attach(&self.build_image(url), 0, row, 4, 1);
                row += 1;
            }
        }

        if let Some(attachment) = msg.attachment {
            if attachment.is_image() {
                self.attach(&self.build_image(attachment.url.to_string()), 0, row, 4, 1);
                row += 1;
            }
        }

        if msg.actions.len() > 0 {
            let action_btns = gtk::FlowBox::builder()
                .row_spacing(8)
                .column_spacing(8)
                .homogeneous(false)
                .selection_mode(gtk::SelectionMode::None)
                .build();

            for a in msg.actions {
                let btn = self.build_action_btn(a);
                btn.add_css_class("pill");
                action_btns.insert(&btn, -1);
            }

            self.attach(&action_btns, 0, row, 4, 1);
            row += 1;
        }
        if msg.tags.len() > 0 {
            let mut tags_text = gettext("tags: ");
            tags_text.push_str(&msg.tags.join(", "));
            let tags = gtk::Label::builder()
                .label(&tags_text)
                .xalign(0.0)
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::WordChar)
                .build();
            self.attach(&tags, 0, row, 4, 1);
            row += 1;
        }

        // Footer: date on the left, badge and actions on the right.
        self.attach(&time_box, 0, row, 1, 1);
        self.attach(&header_actions, 1, row, 3, 1);
    }

    fn fetch_image_bytes(url: &str) -> anyhow::Result<Vec<u8>> {
        validate_image_url(url)?;
        let path = glib::user_cache_dir().join("io.github.tobagin.Ntfyr").join(&url);
        let bytes = if path.exists() {
            std::fs::read(&path)?
        } else {
            let agent: ureq::Agent = ureq::Agent::config_builder()
                .max_redirects(0)
                .build()
                .into();
            agent
                .get(url)
                .call()?
                .into_body()
                .read_to_vec()?
        };
        Ok(bytes)
    }
    fn build_image(&self, url: String) -> gtk::Picture {
        let (s, r) = async_channel::unbounded();
        gio::spawn_blocking(move || {
            if let Err(e) = Self::fetch_image_bytes(&url).and_then(|bytes| {
                let stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&bytes));
                let pixbuf = gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE)?;
                let t = gdk::Texture::for_pixbuf(&pixbuf);
                s.send_blocking(t)?;
                Ok(())
            }) {
                error!(error = %e)
            }
            glib::ControlFlow::Break
        });
        let picture = gtk::Picture::new();
        picture.set_can_shrink(true);
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_halign(gtk::Align::Fill);
        picture.set_height_request(280);
        picture.add_css_class("message-image");
        let picturec = picture.clone();

        self.error_boundary().spawn(async move {
            let t = r.recv().await?;
            picturec.set_paintable(Some(&t));
            Ok(())
        });

        picture
    }
    fn build_action_btn(&self, action: models::Action) -> gtk::Button {
        let btn = gtk::Button::new();
        match &action {
            models::Action::View { label, url, .. } => {
                btn.set_label(&label);
                btn.set_tooltip_text(Some(
                    &gettext("Go to {}").replacen("{}", url, 1),
                ));
                btn.set_action_name(Some("app.message-action"));
                btn.set_action_target_value(Some(&serde_json::to_string(&action).unwrap().into()));
            }
            models::Action::Http {
                label, method, url, ..
            } => {
                btn.set_label(&label);
                btn.set_tooltip_text(Some(
                    &gettext("Send HTTP {} to {}")
                        .replacen("{}", method, 1)
                        .replacen("{}", url, 1),
                ));
                btn.set_action_name(Some("app.message-action"));
                btn.set_action_target_value(Some(&serde_json::to_string(&action).unwrap().into()));
            }
            models::Action::Broadcast { label, .. } => {
                btn.set_label(&label);
                btn.set_sensitive(false);
                btn.set_tooltip_text(Some(&gettext(
                    "Broadcast action only available on Android",
                )));
            }
        }
        btn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_to_pango() {
        assert_eq!(markdown_to_pango("hello"), "hello");
        assert_eq!(markdown_to_pango("**bold**"), "<b>bold</b>");
        assert_eq!(markdown_to_pango("*italic*"), "<i>italic</i>");
        assert_eq!(
            markdown_to_pango("[link](https://example.com)"),
            "<a href=\"https://example.com\">link</a>"
        );
        assert_eq!(markdown_to_pango("`code`"), "<tt>code</tt>");
        assert_eq!(
            markdown_to_pango("multiple **lines**\n*and* formats"),
            "multiple <b>lines</b>\n<i>and</i> formats"
        );
    }
}
