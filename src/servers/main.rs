use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Client,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

use crate::{constants::MAIN_PORT, TARGET};

/// Starts the main server which is responsible for a majority of the
/// game logic such as games, sessions, etc.
pub async fn start_server() {
    // Initializing the underlying TCP listener
    let listener = {
        match TcpListener::bind(("0.0.0.0", MAIN_PORT)).await {
            Ok(value) => {
                println!("Started Main server (Port: {})", MAIN_PORT);
                value
            }
            Err(_) => {
                panic!("Failed to bind  server (Port: {})", MAIN_PORT)
            }
        }
    };

    // Accept incoming connections
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(value) => value,
            Err(err) => {
                panic!("Failed to accept Main connection: {err:?}");
            }
        };

        tokio::spawn(handle_blaze(stream));
    }
}

async fn handle_blaze(mut incoming: TcpStream) {
    let target = &*TARGET.read().await;
    let target = match target {
        Some(value) => value,
        None => return,
    };

    let mut url = String::new();
    url.push_str(&target.scheme);
    url.push_str("://");
    url.push_str(&target.host);
    url.push_str("/api/server/upgrade");

    let mut headers = HeaderMap::new();
    headers.insert(header::CONNECTION, HeaderValue::from_static("Upgrade"));
    headers.insert(header::UPGRADE, HeaderValue::from_static("blaze"));

    let client = Client::builder().build().unwrap();

    let res = client.get(url).headers(headers).send().await.unwrap();

    let mut up = res.upgrade().await.unwrap();

    println!("Upgraded");

    let mut in_buffer = [0u8; 1024];
    let mut out_buffer = [0u8; 1024];

    loop {
        select! {
            result = incoming.read(&mut in_buffer) => {
                let count = result.unwrap();
                up.write(&in_buffer[0..count]).await.unwrap();
                up.flush().await.unwrap();
            },
            result = up.read(&mut out_buffer) => {
                let count = result.unwrap();
                incoming.write(&out_buffer[0..count]).await.unwrap();
                incoming.flush().await.unwrap();
            }
        };
    }
}
