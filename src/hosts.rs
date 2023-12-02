//! Hosts module providing host file modification functionality

use crate::ui::show_warning;
use log::{debug, error, warn};
use std::{
    fs::{read_to_string, write},
    io::{self, ErrorKind},
    path::Path,
    string::FromUtf8Error,
};
use thiserror::Error;

/// The host address to redirect in the hosts file
pub const HOST_KEY: &str = "gosredirector.ea.com";
/// Host address target (Localhost)
pub const HOST_VALUE: &str = "127.0.0.1";
/// The path to the system hosts file on windows devices
#[cfg(target_family = "windows")]
pub const HOSTS_PATH: &str = "C:/Windows/System32/drivers/etc/hosts";
/// The path to the system hosts file on unix devices
#[cfg(target_family = "unix")]
pub const HOSTS_PATH: &str = "/etc/hosts";

/// Errors that could occur while working with the hosts file
#[derive(Debug, Error)]
enum HostsError {
    /// Hosts file doesn't exist
    #[error("Missing system hosts file")]
    FileMissing,
    /// Missing admin permission to access file
    #[error(
        "Missing permission to modify hosts file. Ensure this program is running as admin\n\n\
        You can ignore this warning if you have manually modified your hosts file to include \
        the redirection from gosredirector.ea.com to 127.0.0.1"
    )]
    PermissionsError,
    /// Failed to read the hosts file
    #[error(transparent)]
    IO(io::Error),
    /// File contained non-utf8 characters
    #[error("Hosts file contained non-utf8 characters so could not be parsed.")]
    NonUtf8(#[from] FromUtf8Error),
}

/// Guard structure that applies the host file entry then
/// removes the host entry once the guard is dropped
pub struct HostEntryGuard;

impl HostEntryGuard {
    pub fn apply() -> Option<Self> {
        match Self::apply_entry() {
            Ok(value) => {
                debug!("Applied host modificaiton");
                Some(value)
            }
            Err(err) => {
                show_warning("Failed to apply host modification", &err.to_string());
                warn!("Failed to apply host entry: {}", err);
                None
            }
        }
    }

    fn read_hosts_file() -> Result<String, HostsError> {
        let path = Path::new(HOSTS_PATH);
        if !path.exists() {
            return Err(HostsError::FileMissing);
        }

        // Read the hosts file
        let text = read_to_string(path)?;
        Ok(text)
    }

    fn apply_entry() -> Result<Self, HostsError> {
        let host_line = format!("{} {}", HOST_VALUE, HOST_KEY);

        let output = Self::read_hosts_file()?
            .lines()
            .filter(Self::filter_not_host_line)
            .chain(std::iter::once(host_line.as_str()))
            // Collect the lines into a string with new lines appended
            .fold(String::new(), |mut a, b| {
                a.reserve(b.len() + 1);
                a.push_str(b);
                a.push('\n');
                a
            });

        let path = Path::new(HOSTS_PATH);
        write(path, output)?;
        Ok(Self)
    }

    fn remove_entry() -> Result<(), HostsError> {
        let output = Self::read_hosts_file()?
            .lines()
            .filter(Self::filter_not_host_line)
            // Collect the lines into a string with new lines appended
            .fold(String::new(), |mut a, b| {
                a.reserve(b.len() + 1);
                a.push_str(b);
                a.push('\n');
                a
            });

        let path = Path::new(HOSTS_PATH);
        write(path, output)?;
        Ok(())
    }

    /// Filters lines based on whether they are a host redirect
    /// line entry
    fn filter_not_host_line(value: &&str) -> bool {
        let value = value.trim();
        if value.is_empty() || value.starts_with('#') || !value.contains(HOST_KEY) {
            return true;
        }

        let value = value
            .split_once('#')
            // Take the first half if present
            .map(|(before, _)| before.trim())
            // Take entire line of not containing a comment
            .unwrap_or(value);

        // Check we still have content and contain host
        if value.is_empty() || !value.contains(HOST_KEY) {
            return true;
        }

        // Splits at whitespace and ensures the first part is the host
        let is_host_line = value
            .split_whitespace()
            .nth(1)
            .is_some_and(|value| value.eq(HOST_KEY));
        !is_host_line
    }
}

impl Drop for HostEntryGuard {
    fn drop(&mut self) {
        if let Err(err) = Self::remove_entry() {
            error!("Failed to remove host entry: {}", err);
        } else {
            debug!("Removed host modification")
        }
    }
}

impl From<io::Error> for HostsError {
    fn from(value: io::Error) -> Self {
        match value.kind() {
            ErrorKind::PermissionDenied => HostsError::PermissionsError,
            _ => HostsError::IO(value),
        }
    }
}
