use std::env::current_exe;

use serde::{Deserialize, Serialize};
use tokio::fs::{read, write};

use crate::{constants::CONFIG_FILE_NAME, ui::show_error};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientConfig {
    pub connection_url: String,
}

pub async fn read_config_file() -> Option<ClientConfig> {
    let current_path = current_exe().unwrap();
    let parent = current_path
        .parent()
        .expect("Missing parent directory to current exe path");

    let file_path = parent.join(CONFIG_FILE_NAME);
    if !file_path.exists() {
        return None;
    }

    let bytes = match read(file_path).await {
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

pub async fn write_config_file(config: &ClientConfig) {
    let current_path = current_exe().unwrap();
    let parent = current_path
        .parent()
        .expect("Missing parent directory to current exe path");

    let file_path = parent.join(CONFIG_FILE_NAME);

    let bytes = match serde_json::to_vec(config) {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to save client config", &err.to_string());
            return;
        }
    };

    if let Err(err) = write(file_path, bytes).await {
        show_error("Failed to save client config", &err.to_string());
    }
}
