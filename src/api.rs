use serde::Deserialize;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::config::{write_config_file, ClientConfig};

/// Shared target location
pub static TARGET: RwLock<Option<LookupData>> = RwLock::const_new(None);

/// Details provided by the server. These are the only fields
/// that we need the rest are ignored by this client.
#[derive(Deserialize)]
struct ServerDetails {
    /// The Pocket Relay version of the server
    version: String,
    /// Server identifier
    ident: String,
}

/// Data from completing a lookup contains the resolved address
/// from the connection to the server as well as the server
/// version obtained from the server
#[derive(Debug, Clone)]
pub struct LookupData {
    /// The scheme used to connect to the server (e.g http or https)
    pub scheme: String,
    /// The host address of the server
    pub host: String,
    /// The server version
    pub version: String,
    /// The server port
    pub port: u16,
}

/// Errors that can occur while looking up a server
#[derive(Debug, Error)]
pub enum LookupError {
    /// The server url was missing the host portion
    #[error("Unable to find host portion of provided Connection URL")]
    InvalidHostTarget,
    /// The server connection failed
    #[error("Failed to connect to server: {0}")]
    ConnectionFailed(reqwest::Error),
    /// The server gave an invalid response likely not a PR server
    #[error("Invalid server response: {0}")]
    InvalidResponse(reqwest::Error),
}

pub async fn try_lookup_host(host: &str) -> Result<LookupData, LookupError> {
    let mut url = String::new();

    // Fill in missing host portion
    if !host.starts_with("http://") && !host.starts_with("https://") {
        url.push_str("http://");
        url.push_str(host)
    } else {
        url.push_str(host);
    }

    if !host.ends_with('/') {
        url.push('/')
    }

    url.push_str("api/server");

    let response = reqwest::get(url)
        .await
        .map_err(LookupError::ConnectionFailed)?;

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
        .map_err(LookupError::InvalidResponse)?;

    Ok(LookupData {
        scheme,
        host,
        port,
        version: details.version,
    })
}

/// Attempts to update the host target first looks up the
/// target then will assign the stored global target to the
/// target before returning the result
///
/// `target` The target to use
pub async fn try_update_host(target: String, persist: bool) -> Result<LookupData, LookupError> {
    let result = try_lookup_host(&target).await?;
    let mut write = TARGET.write().await;
    *write = Some(result.clone());

    // Write the config file with the new connection URL
    if persist {
        write_config_file(&ClientConfig {
            connection_url: target,
        })
        .await;
    }

    Ok(result)
}
