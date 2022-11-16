#![windows_subsystem = "windows"]
use std::{
    fmt::Display,
    fs::{read, write},
    io::ErrorKind,
    path::Path,
};

use iced::{
    widget::{button, container, svg, Button, Column, Row, Svg, Text, TextInput},
    window::{self, Icon},
    Color, Length, Sandbox, Settings,
};
use serde::Deserialize;

/// Constant storing the application version
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The host address to redirect in the hosts file
const HOST: &str = "gosredirector.ea.com";
/// Key in the hosts file comment to contain the origin url value
const HOST_ADDR_KEY: &str = "PR_ADDR:";
/// The path to the system hosts file
const HOSTS_PATH: &str = "C:/Windows/System32/drivers/etc/hosts";
/// Logo svg bytes
const LOGO_SVG: &[u8] = include_bytes!("logo.svg");
/// Window icon bytes
const ICON_BYTES: &[u8] = include_bytes!("icon.ico");
/// The window size
const WINDOW_SIZE: (u32, u32) = (380, 325);

#[derive(Debug, Clone)]
pub enum Message {
    /// Sets the host value
    HostChanged(String),
    /// Update the host entry
    UpdateHostEntry,
    /// Removes any host file entries added by the program
    RemoveHostEntry,
    /// Resets the application state
    ResetState,
}

fn main() -> iced::Result {
    PocketRelay::run(Settings {
        window: window::Settings {
            icon: Icon::from_file_data(ICON_BYTES, None).ok(),
            size: WINDOW_SIZE,
            resizable: false,
            ..window::Settings::default()
        },

        ..Settings::default()
    })
}

/// Application states / screens
enum AppState {
    /// Initial home screen state
    Initial,
    /// Hosts file updated screen state
    Updated(LookupData),
    /// Hosts file entry removed screen state
    Removed,
    /// Error occurred screen state
    Error(HostsError),
}

struct PocketRelay {
    /// Whether the hosts file already contains a entry
    has_entry: bool,
    /// The new host to insert into the hosts file
    host: String,
    /// The current state of the app
    state: AppState,
}

impl Sandbox for PocketRelay {
    type Message = Message;

    fn new() -> Self {
        let (has_entry, host) = match get_host_entry() {
            Some(value) => (true, value.original.unwrap_or(value.address)),
            None => (false, String::new()),
        };

        Self {
            has_entry,
            host,
            state: AppState::Initial,
        }
    }

    fn title(&self) -> String {
        "Pocket Relay Client".to_string()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::HostChanged(value) => {
                self.host = value;
            }
            Message::UpdateHostEntry => {
                self.state = match set_host_entry(&self.host) {
                    Err(err) => AppState::Error(err),
                    Ok(value) => {
                        self.has_entry = true;
                        AppState::Updated(value)
                    }
                };
            }
            Message::RemoveHostEntry => {
                self.state = if let Err(err) = remove_host_entry() {
                    AppState::Error(err)
                } else {
                    self.has_entry = false;
                    AppState::Removed
                };
            }
            Message::ResetState => self.state = AppState::Initial,
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let logo = Svg::new(svg::Handle::from_memory(LOGO_SVG))
            .width(Length::Fill)
            .height(Length::Units(90));
        let version_text = Text::new(format!("Version: {}", APP_VERSION))
            .size(10)
            .style(Color::from_rgb(0.4, 0.4, 0.4));
        let mut content = Column::new().spacing(15).push(logo).push(version_text);

        match &self.state {
            AppState::Initial => {
                let host_input = TextInput::new("example.com", &self.host, Message::HostChanged)
                    .padding(10)
                    .size(20);

                let text = Text::new(
                    "Enter the connection url for the Pocket Relay server. \
                        If you are using a domain with a non static IP you will need to press \
                        update when the address changes",
                )
                .width(Length::Fill)
                .size(15)
                .style(Color::from_rgb(0.75, 0.75, 0.75));

                content = content.push(host_input).push(text);

                if self.has_entry {
                    let update_button = Button::new("Update")
                        .on_press(Message::UpdateHostEntry)
                        .width(Length::Fill)
                        .padding(10);

                    let remove_button = Button::new("Remove")
                        .on_press(Message::RemoveHostEntry)
                        .width(Length::Fill)
                        .padding(10);

                    let button_row = Row::new()
                        .spacing(15)
                        .push(update_button)
                        .push(remove_button);

                    content = content.push(button_row);
                } else {
                    let set_button = Button::new("Set")
                        .on_press(Message::UpdateHostEntry)
                        .width(Length::Fill)
                        .padding(10);

                    content = content.push(set_button);
                }
            }
            AppState::Updated(value) => {
                content = content.spacing(10);
                let title = Text::new("Updated Hosts Entry").size(20);
                let text = Text::new(
                    "Successfully connected to Pocket Relay server and updated hosts file",
                )
                .size(15);

                let server_details = Text::new(format!(
                    "Server Version: {}\nServer Address: {}",
                    value.version, value.address
                ))
                .size(15)
                .style(Color::from_rgb(0.5, 0.5, 0.5));

                let ok_button = button("Ok")
                    .width(Length::Fill)
                    .padding(10)
                    .on_press(Message::ResetState);

                content = content
                    .push(title)
                    .push(text)
                    .push(server_details)
                    .push(ok_button)
            }

            AppState::Removed => {
                let title = Text::new("Removed Hosts Entry").size(20);
                let text =
                    Text::new("Sucessfully removed entries that were added to the hosts file")
                        .size(15);
                let ok_button = button("Ok")
                    .width(Length::Fill)
                    .padding(10)
                    .on_press(Message::ResetState);
                content = content.push(title).push(text).push(ok_button)
            }

            AppState::Error(err) => {
                let title = Text::new("Error Occurred").size(20);
                let text = Text::new(format!("{}", err)).size(15);
                let ok_button = button("Ok")
                    .width(Length::Fill)
                    .padding(10)
                    .on_press(Message::ResetState);
                content = content.push(title).push(text).push(ok_button)
            }
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .into()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }
}

#[derive(Debug)]
enum HostsError {
    /// The user provided a empty or invalid host url
    InvalidHost,
    /// The hosts file was missing
    FileMissing,
    /// Failed to read the hosts file
    ReadFailure,
    /// Failed to write the hosts file
    WriteFailure,
    /// Didn't have the permissions required to read / write the hosts file
    PermissionsError,
    /// Failed to connect to the server when looking up the addresss
    ConnectionFailed,
    /// Got an invalid response from the server its possible the server is
    /// not a Pocket Relay server
    InvalidResponse,
}

impl Display for HostsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HostsError::InvalidHost => "Provided host url is invalid try another",
            HostsError::FileMissing => "System hosts file was missing",
            HostsError::ReadFailure => "Unable to read contents of system hosts file",
            HostsError::WriteFailure => "Unable to write contents of system hosts file",
            HostsError::PermissionsError => "Missing permissions to access hosts file. Make sure this program is running as administrator",
            HostsError::ConnectionFailed => "Failed to connect to Pocket Relay server. Check the address you provided is correct.",
            HostsError::InvalidResponse => "Server responded with an unexpected response. Check the address and port you provided is correct."
        })
    }
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
    write(path, value).map_err(|err| {
        if err.kind() == ErrorKind::PermissionDenied {
            HostsError::PermissionsError
        } else {
            HostsError::WriteFailure
        }
    })
}

/// Details provided by the server. These are the only fields
/// that we need the rest are ignored by this client.
#[derive(Deserialize)]
struct ServerDetails {
    /// The external address that the server uses
    address: String,
    /// The Pocket Relay version of the server
    version: String,
}

/// Data from completing a lookup contains the resolved address
/// from the connection to the server as well as the server
/// version obtained from the server
struct LookupData {
    address: String,
    version: String,
}

/// Attempts to connect to the Pocket Relay HTTP server at the provided
/// host. Will make a connection to the /api/server endpoint and if the
/// response is a valid ServerDetails message then the server is
/// considered valid.
///
/// `host` The host to try and lookup
fn try_lookup_host(host: &str) -> Result<LookupData, HostsError> {
    let mut url = String::new();

    if !host.starts_with("http://") && !host.starts_with("https://") {
        url.push_str("http://");
        url.push_str(host)
    } else {
        url.push_str(host);
    }

    if !host.ends_with("/") {
        url.push('/')
    }

    url.push_str("api/server");

    let response = reqwest::blocking::get(url).map_err(|_| HostsError::ConnectionFailed)?;
    let address = response.remote_addr().ok_or(HostsError::ConnectionFailed)?;
    let address = format!("{}", address.ip());
    let details = response
        .json::<ServerDetails>()
        .map_err(|_| HostsError::InvalidResponse)?;

    println!(
        "Server details (Address: {}, Version: {})",
        details.address, details.version
    );

    Ok(LookupData {
        address,
        version: details.version,
    })
}

/// Updates the hosts file with the entry loaded from the server
/// url
///
/// `url` The lookup url for Pocket Relay
fn set_host_entry(url: &str) -> Result<LookupData, HostsError> {
    if url.is_empty() {
        return Err(HostsError::InvalidHost);
    }

    let lookup_data = try_lookup_host(url)?;

    let contents = read_hosts_file()?;

    let mut lines = contents
        .lines()
        .filter(filter_not_host_line)
        .collect::<Vec<&str>>();

    let line = format!(
        "{} {} # {} {}",
        &lookup_data.address, HOST, HOST_ADDR_KEY, url
    );

    lines.push(&line);

    let output = lines.join("\n");
    write_hosts_file(&output)?;

    Ok(lookup_data)
}

/// Structure representing a host address that optionally has
/// an original url string that the address was derived from
struct HostAddr {
    /// The address string
    address: String,
    /// The original url used to obtain the address string
    original: Option<String>,
}

/// Filters and maps lines to host address values. Will return
/// none if the line doesn't contain a host address. If it does
/// then the line will be parsed
///
/// `value` The value to filter and map
fn filter_map_host_line(value: &str) -> Option<HostAddr> {
    let value = value.trim();
    if value.is_empty() || value.starts_with("#") || !value.contains(HOST) {
        return None;
    }

    // Split to the content before any comments
    let (value, after) = match value.split_once("#") {
        Some((before, after)) => (before.trim(), Some(after.trim())),
        None => (value, None),
    };

    // Check we still have content and contain host
    if value.is_empty() || !value.contains(HOST) {
        return None;
    }

    let mut parts = value.split_whitespace();
    let address = parts.next()?;
    let host = parts.next()?;

    if !host.eq(HOST) {
        return None;
    }

    let address = address.to_owned();

    let mut original = None;

    if let Some(after) = after {
        if let Some((_, after)) = after.split_once(HOST_ADDR_KEY) {
            let value = after.trim();
            original = Some(value.to_owned())
        }
    }

    Some(HostAddr { address, original })
}

/// Filters lines based on whether or not they are a redirect for
/// the host address. Filters out lines that are commented out
/// / are invalid.
///
/// `value` The line to check
fn filter_not_host_line(value: &&str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.starts_with("#") || !value.contains(HOST) {
        return true;
    }

    // Split to the content before any comments
    let value = match value.split_once("#") {
        Some((before, _)) => before.trim(),
        None => value,
    };

    // Check we still have content and contain host
    if value.is_empty() || !value.contains(HOST) {
        return true;
    }

    let mut parts = value.split_whitespace();

    match parts.next() {
        Some(_) => {}
        None => return true,
    }

    match parts.next() {
        Some(value) => !value.eq(HOST),
        None => true,
    }
}

/// Attempts to find the current host entry value
/// returning None if one wasnt found
fn get_host_entry() -> Option<HostAddr> {
    let contents = read_hosts_file().ok()?;
    contents.lines().find_map(filter_map_host_line)
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
