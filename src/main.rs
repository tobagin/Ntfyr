mod application;
#[rustfmt::skip]
mod config;
mod async_utils;
pub mod error;
mod i18n;
mod subscription;
pub mod widgets;
mod tray;
pub mod secrets;

use gettextrs::gettext;
use gtk::{gio, glib};

use self::application::NtfyrApplication;
use self::config::RESOURCES_FILE;

fn main() -> glib::ExitCode {
    // Initialize logger
    tracing_subscriber::fmt::init();

    // Prepare i18n (locale from GSettings, then gettext catalogs)
    i18n::init();

    glib::set_application_name(&gettext("Ntfyr"));

    let res = gio::Resource::load(RESOURCES_FILE).expect("Could not load gresource file");
    gio::resources_register(&res);

    let app = NtfyrApplication::default();
    app.run()
}
