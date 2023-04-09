#![windows_subsystem = "windows"]
use crate::constants::APP_VERSION;
use constants::*;
use native_dialog::{FileDialog, MessageDialog};
use ngw::{GridLayoutItem, Icon};
use pollster::FutureExt as _;
use serde::Deserialize;
use std::fs::{copy, read, remove_file, write};
use std::rc::Rc;
use std::{
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::sync::RwLock;

mod constants;
mod servers;

extern crate native_windows_gui as ngw;

#[tokio::main]
async fn main() {
    // Add the hosts file entry
    let _ = set_host_entry();

    // Start the servers
    tokio::spawn(servers::start());

    ngw::init().expect("Failed to initialize native UI");
    ngw::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let mut window = Default::default();
    let mut target_url = Default::default();
    let mut set_button = Default::default();
    let mut p_button = Default::default();
    let mut pr_button = Default::default();
    let layout = Default::default();

    let mut top_label = Default::default();
    let mut mid_label = Default::default();
    let mut bot_label = Default::default();
    let mut c_label = Default::default();

    let mut icon = Default::default();

    Icon::builder()
        .source_bin(Some(ICON_BYTES))
        .build(&mut icon)
        .unwrap();

    ngw::Window::builder()
        .size(WINDOW_SIZE)
        .position((5, 5))
        .icon(Some(&icon))
        .title(&format!("Pocket Relay Client v{}", APP_VERSION))
        .build(&mut window)
        .unwrap();

    ngw::Label::builder()
        .text("Please put the server Connection URL below and press 'Set'")
        .parent(&window)
        .build(&mut top_label)
        .unwrap();
    ngw::Label::builder()
        .text("You must keep this program running while playing. Closing this \nprogram will cause you to connect to the official servers instead.")
        .parent(&window)
        .build(&mut mid_label)
        .unwrap();
    ngw::Label::builder()
        .text("You must patch your game in order to make it compatible with\n Pocket Relay")
        .parent(&window)
        .build(&mut bot_label)
        .unwrap();
    ngw::Label::builder()
        .text("Not connected")
        .parent(&window)
        .build(&mut c_label)
        .unwrap();

    ngw::TextInput::builder()
        .text("")
        .focus(true)
        .parent(&window)
        .build(&mut target_url)
        .unwrap();

    ngw::Button::builder()
        .text("Set")
        .parent(&window)
        .build(&mut set_button)
        .unwrap();

    ngw::Button::builder()
        .text("Patch Game")
        .parent(&window)
        .build(&mut p_button)
        .unwrap();
    ngw::Button::builder()
        .text("Remove Patch")
        .parent(&window)
        .build(&mut pr_button)
        .unwrap();

    ngw::GridLayout::builder()
        .parent(&window)
        .child_item(GridLayoutItem::new(&top_label, 0, 0, 2, 1))
        .child_item(GridLayoutItem::new(&target_url, 0, 1, 2, 1))
        .child_item(GridLayoutItem::new(&set_button, 0, 2, 2, 1))
        .child_item(GridLayoutItem::new(&c_label, 0, 3, 2, 1))
        .child_item(GridLayoutItem::new(&mid_label, 0, 4, 2, 1))
        .child_item(GridLayoutItem::new(&p_button, 0, 5, 1, 1))
        .child_item(GridLayoutItem::new(&pr_button, 1, 5, 1, 1))
        .child_item(GridLayoutItem::new(&bot_label, 0, 6, 2, 1))
        .build(&layout)
        .unwrap();

    let window = Rc::new(window);
    let events_window = window.clone();

    let c_label = Rc::new(c_label);

    let handler = ngw::full_bind_event_handler(&window.handle, move |evt, _evt_data, handle| {
        use ngw::Event as E;

        match evt {
            E::OnWindowClose => {
                if &handle == &events_window as &ngw::Window {
                    ngw::stop_thread_dispatch();
                    remove_host_entry();
                }
            }

            E::OnButtonClick => {
                if &handle == &set_button {
                    c_label.set_text("Loading...");

                    let target = target_url.text();

                    let ew = events_window.clone();
                    let c = c_label.clone();
                    async move {
                        let result = try_lookup_host(target).await;
                        match result {
                            Ok(value) => {
                                let write = &mut *TARGET.write().await;
                                *write = Some(value.clone());
                                c.set_text(&format!(
                                    "Connected: {} {} version v{}",
                                    value.scheme, value.host, value.version
                                ));
                            }
                            Err(err) => {
                                c.set_text("Failed to connect");
                                ngw::modal_error_message(
                                    &ew.handle,
                                    "Failed to connect",
                                    &err.to_string(),
                                );
                            }
                        }
                    }
                    .block_on();
                } else if &handle == &p_button {
                    match try_patch_game() {
                        Ok(true) => {
                            ngw::modal_info_message(
                                &events_window.handle,
                                "Game patched",
                                "Sucessfully patched game",
                            );
                        }
                        Ok(false) => {}
                        Err(err) => {
                            ngw::modal_error_message(
                                &events_window.handle,
                                "Failed to patch game",
                                &err.to_string(),
                            );
                        }
                    }
                } else if &handle == &pr_button {
                    match try_remove_patch() {
                        Ok(true) => {
                            ngw::modal_info_message(
                                &events_window.handle,
                                "Patch removed",
                                "Sucessfully removed patch",
                            );
                        }
                        Ok(false) => {}
                        Err(err) => {
                            ngw::modal_error_message(
                                &events_window.handle,
                                "Failed to remove patch",
                                &err.to_string(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    });

    ngw::dispatch_thread_events();
    ngw::unbind_event_handler(&handler);
    println!("Skip end");
}

pub static TARGET: RwLock<Option<LookupData>> = RwLock::const_new(None);

#[derive(Debug, Error)]
enum HostsError {
    #[error("Missing hosts file")]
    FileMissing,
    #[error("Missing permission to modify hosts file")]
    PermissionsError,
    #[error("Failed to read hosts file")]
    ReadFailure,
    #[error("Failed to write hosts file")]
    WriteFailure,
}

/// Attempts to read the hosts file contents to a string
/// returning a HostsError if it was unable to do so
fn read_hosts_file() -> Result<String, HostsError> {
    let path = Path::new(HOSTS_PATH);
    if !path.exists() {
        return Err(HostsError::FileMissing);
    }
    let result = read(path);
    match result {
        Err(err) => Err(if err.kind() == ErrorKind::PermissionDenied {
            HostsError::PermissionsError
        } else {
            HostsError::ReadFailure
        }),
        Ok(bytes) => String::from_utf8(bytes).map_err(|_| HostsError::ReadFailure),
    }
}

/// Attempts to write the hosts file contents from a string
/// returning a HostsError if it was unable to do so
fn write_hosts_file(value: &str) -> Result<(), HostsError> {
    let path = Path::new(HOSTS_PATH);

    if let Err(err) = write(path, value) {
        return Err(if err.kind() == ErrorKind::PermissionDenied {
            HostsError::PermissionsError
        } else {
            HostsError::WriteFailure
        });
    }
    Ok(())
}

/// Filters lines based on whether or not they are a redirect for
/// the host address. Filters out lines that are commented out
/// / are invalid.
///
/// `value` The line to check
fn filter_not_host_line(value: &&str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.starts_with('#') || !value.contains(HOST_KEY) {
        return true;
    }

    // Split to the content before any comments
    let value = match value.split_once('#') {
        Some((before, _)) => before.trim(),
        None => value,
    };

    // Check we still have content and contain host
    if value.is_empty() || !value.contains(HOST_KEY) {
        return true;
    }

    let mut parts = value.split_whitespace();

    match parts.next() {
        Some(_) => {}
        None => return true,
    }

    match parts.next() {
        Some(value) => !value.eq(HOST_KEY),
        None => true,
    }
}

/// Filters all the host redirects removing any for the
/// gosredirector.ea.com host
fn remove_host_entry() -> Result<(), HostsError> {
    let contents = read_hosts_file()?;
    let lines = contents
        .lines()
        .filter(filter_not_host_line)
        .collect::<Vec<&str>>();
    let output = lines.join("\n");
    write_hosts_file(&output)?;
    Ok(())
}

/// Updates the hosts file with the entry loaded from the server
/// url
///
/// `url` The lookup url for Pocket Relay
fn set_host_entry() -> Result<(), HostsError> {
    let contents = read_hosts_file()?;

    let mut lines = contents
        .lines()
        .filter(filter_not_host_line)
        .collect::<Vec<&str>>();

    let line = format!("{} {}", HOST_VALUE, HOST_KEY);

    lines.push(&line);

    let output = lines.join("\n");
    write_hosts_file(&output)?;

    Ok(())
}

/// Details provided by the server. These are the only fields
/// that we need the rest are ignored by this client.
#[derive(Deserialize)]
struct ServerDetails {
    /// The Pocket Relay version of the server
    version: String,
}

/// Data from completing a lookup contains the resolved address
/// from the connection to the server as well as the server
/// version obtained from the server
#[derive(Debug, Clone)]
pub struct LookupData {
    scheme: String,
    host: String,
    version: String,
    port: u16,
}

#[derive(Debug, Error)]
enum LookupError {
    #[error("Unable to find host portion of provided Connection URL")]
    InvalidHostTarget,
    #[error("Failed to connect to server")]
    ConnectionFailed,
    #[error("Invalid server response")]
    InvalidResponse,
}

/// Attempts to connect to the Pocket Relay HTTP server at the provided
/// host. Will make a connection to the /api/server endpoint and if the
/// response is a valid ServerDetails message then the server is
/// considered valid.
///
/// `host` The host to try and lookup
async fn try_lookup_host(host: String) -> Result<LookupData, LookupError> {
    let mut url = String::new();

    if !host.starts_with("http://") && !host.starts_with("https://") {
        url.push_str("http://");
        url.push_str(&host)
    } else {
        url.push_str(&host);
    }

    if !host.ends_with('/') {
        url.push('/')
    }

    url.push_str("api/server");

    let response = reqwest::get(url)
        .await
        .map_err(|_| LookupError::ConnectionFailed)?;

    let url = response.url();
    let scheme = url.scheme().to_string();

    let port = url.port_or_known_default().unwrap_or(80);
    let host = match url.host() {
        Some(value) => value.to_string(),
        None => return Err(LookupError::InvalidHostTarget),
    };

    let details = response
        .json::<ServerDetails>()
        .await
        .map_err(|_| LookupError::InvalidResponse)?;

    Ok(LookupData {
        scheme,
        host,
        port,
        version: details.version,
    })
}

#[derive(Debug, Error)]
enum PatchError {
    #[error("Failed to get picked file. Make sure this program is running as administrator")]
    PickFileFailed,
    #[error("The path given doesn't contains the MassEffect.exe executable")]
    MissingGame,
    #[error("Failed to delete binkw32.dll you will have to manually unpatch your game: {0}")]
    FailedDelete(io::Error),
    #[error("Failed to replace binkw32.dll with origin binkw23.ddl: {0}")]
    FailedReplaceOriginal(io::Error),
    #[error("Failed to write patch file dlls (binkw32.dll and binkw32.dll): {0}")]
    FailedWritingPatchFiles(io::Error),
}

/// Attempt to use the system file picker to pick the path to the
/// Mass Effect 3 executable
fn try_pick_game_path() -> Result<Option<PathBuf>, PatchError> {
    FileDialog::new()
        .set_filename("MassEffect3.exe")
        .add_filter("Mass Effect 3 Executable", &["exe"])
        .show_open_single_file()
        .map_err(|_| PatchError::PickFileFailed)
}

pub fn show_info(title: &str, text: &str) {
    MessageDialog::new()
        .set_title(&title)
        .set_text(&text)
        .set_type(native_dialog::MessageType::Info)
        .show_alert()
        .unwrap()
}

pub fn show_error(title: &str, text: &str) {
    MessageDialog::new()
        .set_title(&title)
        .set_text(&text)
        .set_type(native_dialog::MessageType::Error)
        .show_alert()
        .unwrap()
}

/// Attempts to remove the patch from the provided Mass Effect
/// installation by swapping the binkw32 ddl with binkw23 and
/// deleting the old DLL
fn try_remove_patch() -> Result<bool, PatchError> {
    let path = match try_pick_game_path()? {
        Some(value) => value,
        None => return Ok(false),
    };
    if !path.exists() {
        return Err(PatchError::MissingGame);
    }

    let parent = path.parent().ok_or(PatchError::MissingGame)?;

    let binkw23 = parent.join("binkw23.dll");
    let binkw32 = parent.join("binkw32.dll");

    if binkw32.exists() {
        remove_file(&binkw32).map_err(PatchError::FailedDelete)?;
    }

    if binkw23.exists() {
        copy(&binkw23, &binkw32).map_err(PatchError::FailedReplaceOriginal)?;
        let _ = remove_file(&binkw23);
    } else {
        write(&binkw32, BINKW23_DLL_BYTES).map_err(PatchError::FailedReplaceOriginal)?;
    }

    Ok(true)
}

/// Attempts to patch the Mass Effect installation at the provided game
/// path. Writes the two embedded DLLs to the game directory.
fn try_patch_game() -> Result<bool, PatchError> {
    let path = match try_pick_game_path()? {
        Some(value) => value,
        None => return Ok(false),
    };
    if !path.exists() {
        return Err(PatchError::MissingGame);
    }
    let parent = path.parent().ok_or(PatchError::MissingGame)?;

    let binkw23 = parent.join("binkw23.dll");
    let binkw32 = parent.join("binkw32.dll");

    write(&binkw23, BINKW23_DLL_BYTES).map_err(PatchError::FailedWritingPatchFiles)?;
    write(&binkw32, BINKW32_DLL_BYTES).map_err(PatchError::FailedWritingPatchFiles)?;
    Ok(true)
}
