use native_dialog::{FileDialog, MessageDialog};
use std::path::PathBuf;

#[cfg(feature = "iced")]
pub mod iced;
#[cfg(feature = "native")]
pub mod native;

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
