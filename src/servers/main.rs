use crate::{
    api::TARGET,
    constants::{HTTP_PORT, MAIN_PORT},
    ui::show_error,
};
use hyper::header::HeaderName;
use log::error;
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use std::{net::Ipv4Addr, process::exit};
use tokio::{
    io::copy_bidirectional,
    net::{TcpListener, TcpStream},
};

/// Starts the main server proxy. This creates a connection to the Pocket Relay
/// which is upgraded and then used as the main connection fro the game.
pub async fn start_server(http_client: Client) {
    // Initializing the underlying TCP listener
    let listener = match TcpListener::bind((Ipv4Addr::UNSPECIFIED, MAIN_PORT)).await {
        Ok(value) => value,
        Err(err) => {
            let text = format!("Failed to start main: {}", err);
            show_error("Failed to start", &text);
            exit(1);
        }
    };

    // Accept incoming connections
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(value) => value,
            Err(_) => break,
        };

        // Spawn off a new handler for the connection
        tokio::spawn(handle_blaze(stream, http_client.clone()));
    }
}

/// Header for the Pocket Relay connection scheme used by the client
const LEGACY_HEADER_SCHEME: &str = "x-pocket-relay-scheme";
/// Header for the Pocket Relay connection port used by the client
const LEGACY_HEADER_PORT: &str = "x-pocket-relay-port";
/// Header for the Pocket Relay connection host used by the client
const LEGACY_HEADER_HOST: &str = "x-pocket-relay-host";
/// Header to tell the server to use local HTTP
const HEADER_LOCAL_HTTP: &str = "x-pocket-relay-local-http";

/// Endpoint for upgrading the server connection
const UPGRADE_ENDPOINT: &str = "api/server/upgrade";

async fn handle_blaze(mut client: TcpStream, http_client: Client) {
    let url = match &*TARGET.read().await {
        // Create the upgrade URL
        Some(target) => target
            .url
            .join(UPGRADE_ENDPOINT)
            .expect("Failed to create update endpoint URL"),

        None => return,
    };

    // Create the required headers
    let headers: HeaderMap<HeaderValue> = [
        // Required headers for HTTP upgrade
        (header::CONNECTION, HeaderValue::from_static("Upgrade")),
        (header::UPGRADE, HeaderValue::from_static("blaze")),
        // Legacy headers to force usage of local HTTP
        (
            HeaderName::from_static(LEGACY_HEADER_SCHEME),
            HeaderValue::from_static("http"),
        ),
        (
            HeaderName::from_static(LEGACY_HEADER_HOST),
            HeaderValue::from_static("127.0.0.1"),
        ),
        (
            HeaderName::from_static(LEGACY_HEADER_PORT),
            HeaderValue::from(HTTP_PORT),
        ),
        // Header informing server to use local http (Legacy servers)
        (
            HeaderName::from_static(HEADER_LOCAL_HTTP),
            HeaderValue::from_static("true"),
        ),
    ]
    .into_iter()
    .collect();

    let request = http_client.get(url).headers(headers);
    let response = request.send().await;

    let response = match response {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to upgrade client (err connect): {}", err);
            return;
        }
    };

    // Handle error responses
    let response = match response.error_for_status() {
        Ok(response) => response,
        Err(err) => {
            error!("Failed to upgrade client (err response): {}", err);
            return;
        }
    };

    // Upgrade the connection to a stream
    let mut server = match response.upgrade().await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to upgrade client (upgrade): {}", err);
            return;
        }
    };

    // Copy the data between the connection
    let _ = copy_bidirectional(&mut client, &mut server).await;
}
