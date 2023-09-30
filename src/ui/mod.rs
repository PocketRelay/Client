use native_dialog::{FileDialog, MessageDialog};
use reqwest::Client;
use std::path::PathBuf;

use crate::config::ClientConfig;

// Iced UI variant
#[cfg(feature = "iced")]
pub mod iced;
// Windows native UI variant
#[cfg(feature = "native")]
pub mod native;

/// Wrapper around the init functions for the different
/// UI variants based on the enabled features
#[inline(always)]
pub fn init(config: Option<ClientConfig>, client: Client) {
    #[cfg(feature = "iced")]
    {
        iced::init(config, client)
    }
    #[cfg(feature = "native")]
    {
        native::init(config, client)
    }
}

pub fn try_pick_game_path() -> native_dialog::Result<Option<PathBuf>> {
    FileDialog::new()
        .set_filename("MassEffect3.exe")
        .add_filter("Mass Effect 3 Executable", &["exe"])
        .show_open_single_file()
}

pub fn show_info(title: &str, text: &str) {
    MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Info)
        .show_alert()
        .unwrap()
}

pub fn show_error(title: &str, text: &str) {
    MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Error)
        .show_alert()
        .unwrap()
}

pub fn show_confirm(title: &str, text: &str) -> bool {
    MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Info)
        .show_confirm()
        .unwrap()
}
