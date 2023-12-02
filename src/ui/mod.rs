use crate::{config::ClientConfig, core::reqwest};

// Iced UI variant
#[cfg(feature = "iced")]
pub mod iced;
// Windows native UI variant
#[cfg(feature = "native")]
pub mod native;

/// Title used for created windows
pub const WINDOW_TITLE: &str = concat!("Pocket Relay Client v", env!("CARGO_PKG_VERSION"));
/// Window icon bytes
pub const ICON_BYTES: &[u8] = include_bytes!("../resources/icon.ico");

/// Wrapper around the init functions for the different
/// UI variants based on the enabled features
#[inline(always)]
pub fn init(config: Option<ClientConfig>, client: reqwest::Client) {
    #[cfg(feature = "iced")]
    {
        iced::init(config, client)
    }
    #[cfg(feature = "native")]
    {
        native::init(config, client)
    }
}

#[cfg(feature = "native")]
#[inline]
pub fn show_info(title: &str, text: &str) {
    native_windows_gui::simple_message(title, text);
}

#[cfg(feature = "native")]
#[inline]
pub fn show_error(title: &str, text: &str) {
    native_windows_gui::error_message(title, text);
}

#[cfg(feature = "native")]
pub fn show_warning(title: &str, text: &str) {
    let params = native_windows_gui::MessageParams {
        title,
        content: text,
        buttons: native_windows_gui::MessageButtons::Ok,
        icons: native_windows_gui::MessageIcons::Warning,
    };

    native_windows_gui::message(&params);
}

#[cfg(feature = "native")]
pub fn show_confirm(title: &str, text: &str) -> bool {
    let params = native_windows_gui::MessageParams {
        title,
        content: text,
        buttons: native_windows_gui::MessageButtons::YesNo,
        icons: native_windows_gui::MessageIcons::Question,
    };

    native_windows_gui::message(&params) == native_windows_gui::MessageChoice::Yes
}

#[cfg(not(feature = "native"))]
pub fn show_info(title: &str, text: &str) {
    native_dialog::MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Info)
        .show_alert()
        .unwrap()
}

#[cfg(not(feature = "native"))]
pub fn show_error(title: &str, text: &str) {
    native_dialog::MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Error)
        .show_alert()
        .unwrap()
}

#[cfg(not(feature = "native"))]
pub fn show_warning(title: &str, text: &str) {
    native_dialog::MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Warning)
        .show_alert()
        .unwrap()
}

#[cfg(not(feature = "native"))]
pub fn show_confirm(title: &str, text: &str) -> bool {
    native_dialog::MessageDialog::new()
        .set_title(title)
        .set_text(text)
        .set_type(native_dialog::MessageType::Info)
        .show_confirm()
        .unwrap()
}
