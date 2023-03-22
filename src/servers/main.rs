use crate::{constants::MAIN_PORT, show_error, TARGET};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client, Upgraded,
};
use std::{io, net::Ipv4Addr, process::exit};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

/// Starts the main server proxy. This creates a connection to the Pocket Relay
/// which is upgraded and then used as the main connection fro the game.
pub async fn start_server() {
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
        tokio::spawn(handle_blaze(stream));
    }
}

/// Header for the Pocket Relay connection scheme used by the client
const HEADER_SCHEME: &str = "X-Pocket-Relay-Scheme";
/// Header for the Pocket Relay connection host used by the client
const HEADER_HOST: &str = "X-Pocket-Relay-Host";
/// Endpoint for upgrading the server connection
const UPGRADE_ENDPOINT: &str = "/api/server/upgrade";
/// The size of the buffers used for proxying data
const BUFFER_SIZE: usize = 4096;

async fn handle_blaze(client: TcpStream) {
    let target = match &*TARGET.read().await {
        Some(value) => value.clone(),
        None => return,
    };

    // Create the upgrade URL
    let mut url = String::new();
    url.push_str(&target.scheme);
    url.push_str("://");
    url.push_str(&target.host);
    url.push_str(UPGRADE_ENDPOINT);

    // Create the HTTP Upgrade headers
    let mut headers = HeaderMap::new();
    headers.insert(header::CONNECTION, HeaderValue::from_static("Upgrade"));
    headers.insert(header::UPGRADE, HeaderValue::from_static("blaze"));

    // Append the schema header
    if let Ok(scheme_value) = HeaderValue::from_str(&target.scheme) {
        headers.insert(HEADER_SCHEME, scheme_value);
    }

    // Append the host header
    if let Ok(host_value) = HeaderValue::from_str(&target.host) {
        headers.insert(HEADER_HOST, host_value);
    }

    // Create the request
    let request = Client::new().get(url).headers(headers).send();

    // Await the server response to the request
    let response = match request.await {
        Ok(value) => value,
        Err(_) => return,
    };

    // Server connection gained through upgrading the client
    let server = match response.upgrade().await {
        Ok(value) => value,
        Err(_) => return,
    };

    // Pipe all the content between the client and server
    let _ = pipe(client, server).await;
}

/// Reads all the bytes from the client and the server sending the bytes to
/// the opposite side (i.e. client -> server, and server -> client)
///
/// `client` The client stream to pipe
/// `server` The server stream to pipe
async fn pipe(mut client: TcpStream, mut server: Upgraded) -> io::Result<()> {
    // Buffer for data recieved from the client
    let mut client_buffer = [0u8; BUFFER_SIZE];
    // Buffer for data recieved from the server
    let mut server_buffer = [0u8; BUFFER_SIZE];

    loop {
        select! {
            result = client.read(&mut client_buffer) => {
                let count = result?;
                server.write(&client_buffer[0..count]).await?;
                server.flush().await?;
            },
            result = server.read(&mut server_buffer) => {
                let count = result?;
                client.write(&server_buffer[0..count]).await?;
                client.flush().await?;
            }
        };
    }
}
