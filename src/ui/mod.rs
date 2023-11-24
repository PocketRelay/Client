use crate::config::ClientConfig;
use native_dialog::MessageDialog;
use reqwest::Client;

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
    // #[cfg(feature = "iced")]
    // {
    //     iced::init(config, client)
    // }
    #[cfg(feature = "native")]
    {
        native::init(config, client)
    }
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
