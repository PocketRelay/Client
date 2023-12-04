use crate::ui::show_error;
use log::debug;
use serde::{Deserialize, Serialize};
use std::{env::current_exe, path::PathBuf};

/// Name of the file that stores saved pocket relay configuration info
pub const CONFIG_FILE_NAME: &str = "pocket-relay-client.json";

/// Structure of the configuration file
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    /// The saved connection URL to use
    pub connection_url: String,
}

/// Provides a [`PathBuf`] to the configuration file
pub fn config_path() -> PathBuf {
    let current_path = current_exe().expect("Failed to find exe path");
    let parent = current_path
        .parent()
        .expect("Missing parent directory to current exe path");
    parent.join(CONFIG_FILE_NAME)
}

/// Reads the [`ClientConfig`] from the config file if one is present
pub fn read_config_file() -> Option<ClientConfig> {
    // Check that the config file exists
    let file_path = config_path();
    if !file_path.exists() {
        return None;
    }

    debug!("Reading config file");

    // Read the config bytes from the file
    let bytes = match std::fs::read(file_path) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to read client config", &err.to_string());
            return None;
        }
    };

    // Parse the config file bytes
    match serde_json::from_slice(&bytes) {
        Ok(value) => Some(value),
        Err(err) => {
            show_error("Failed to parse client config", &err.to_string());
            None
        }
    }
}

/// Writes the provided `config` to the config file, this will create a new
/// file if one is not present
pub fn write_config_file(config: ClientConfig) {
    let file_path = config_path();
    let bytes = match serde_json::to_vec(&config) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to save client config", &err.to_string());
            return;
        }
    };
    debug!("Writing config file");
    if let Err(err) = std::fs::write(file_path, bytes) {
        show_error("Failed to save client config", &err.to_string());
    }
}
