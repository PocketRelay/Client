use std::{env::current_exe, path::Path};

use log::debug;
use reqwest::Identity;
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

    debug!("Reading config file");

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

    debug!("Writing config file");

    if let Err(err) = write(file_path, bytes).await {
        show_error("Failed to save client config", &err.to_string());
    }
}

pub async fn load_client_identity() -> Option<Identity> {
    let identity_file = Path::new("pocket-relay-identity.p12");
    if !identity_file.exists() || !identity_file.is_file() {
        return None;
    }

    let identity_bytes = match read(identity_file).await {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to read identity", &err.to_string());
            return None;
        }
    };

    let identity = match Identity::from_pkcs12_der(&identity_bytes, "") {
        Ok(value) => value,
        Err(err) => {
            show_error("Failed to load identity", &err.to_string());
            return None;
        }
    };

    Some(identity)
}
