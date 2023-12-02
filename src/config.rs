use crate::{constants::CONFIG_FILE_NAME, ui::show_error};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{env::current_exe, path::PathBuf};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
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

pub fn read_config_file() -> Option<ClientConfig> {
    let file_path = config_path();
    if !file_path.exists() {
        return None;
    }

    debug!("Reading config file");

    let bytes = match std::fs::read(file_path) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to read client config", &err.to_string());
            return None;
        }
    };

    let config: ClientConfig = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to parse client config", &err.to_string());
            return None;
        }
    };

    Some(config)
}

pub async fn write_config_file(config: ClientConfig) {
    let file_path = config_path();
    let bytes = match serde_json::to_vec(&config) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to save client config", &err.to_string());
            return;
        }
    };
    debug!("Writing config file");
    if let Err(err) = tokio::fs::write(file_path, bytes).await {
        show_error("Failed to save client config", &err.to_string());
    }
}
