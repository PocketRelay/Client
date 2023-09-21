use crate::{
    constants::{MAIN_PORT, REDIRECTOR_PORT},
    show_error,
};

use blaze_ssl_async::{BlazeAccept, BlazeListener};
use futures::{SinkExt, StreamExt};
use std::{io, net::Ipv4Addr, process::exit, time::Duration};
use tdf::TdfSerialize;
use tokio::{select, time::sleep};
use tokio_util::codec::Framed;

use super::packet::{Packet, PacketCodec};

/// Redirector server. Handles directing clients that connect to the local
/// proxy server that will connect them to the target server.
pub async fn start_server() {
    // Bind a listener for SSLv3 connections over TCP
    let listener = match BlazeListener::bind((Ipv4Addr::UNSPECIFIED, REDIRECTOR_PORT)).await {
        Ok(value) => value,
        Err(err) => {
            // Handle failure to bind the server
            let text = format!("Failed to start redirector: {}", err);
            show_error("Failed to start", &text);
            exit(1);
        }
    };

    // Accept incoming connections
    loop {
        // Accept a new connection
        let accept = match listener.accept().await {
            Ok(value) => value,
            Err(_) => break,
        };

        // Spawn a handler for the listener
        tokio::spawn(async move {
            let _ = handle_client(accept).await;
        });
    }
}

/// The timeout before idle redirector connections are terminated
/// (1 minutes before disconnect timeout)
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

const REDIRECTOR: u16 = 0x5;
const GET_SERVER_INSTANCE: u16 = 0x1;

/// Handles dealing with a redirector client
///
/// `stream`   The stream to the client
/// `addr`     The client address
/// `instance` The server instance information
async fn handle_client(accept: BlazeAccept) -> io::Result<()> {
    // Complete the SSLv3 handshaking process
    let (stream, _) = match accept.finish_accept().await {
        Ok(value) => value,
        Err(_) => return Ok(()),
    };

    // Create a packet reader
    let mut framed = Framed::new(stream, PacketCodec);

    loop {
        let packet = select! {
            // Attempt to read packets from the stream
            result = framed.next() => result,
            // If the timeout completes before the redirect is complete the
            // request is considered over and terminates
            _ = sleep(DEFAULT_TIMEOUT) => { break; }
        };

        let packet = match packet {
            Some(Ok(value)) => value,
            Some(Err(err)) => return Err(err),
            None => break,
        };

        let header = packet.header;

        if header.component != REDIRECTOR || header.command != GET_SERVER_INSTANCE {
            // Empty response for packets that aren't asking to redirect
            framed.send(Packet::response_empty(&packet)).await?;
            continue;
        }

        // Response with the instance details
        let response = Packet::response(&packet, LocalInstance);
        framed.send(response).await?;
        break;
    }

    Ok(())
}

/// Packet contents for providing the redirection details
/// for 127.0.0.1 to allow proxying
pub struct LocalInstance;

impl TdfSerialize for LocalInstance {
    fn serialize<S: tdf::TdfSerializer>(&self, w: &mut S) {
        w.tag_union_start(b"ADDR", 0x0); /* Server address type */

        // Encode the net address portion
        w.group(b"VALU", |w| {
            w.tag_u32(b"IP", u32::from_be_bytes([127, 0, 0, 1]));
            w.tag_u16(b"PORT", MAIN_PORT);
        });

        // Extra deatils
        w.tag_bool(b"SECU", false); /* SSLv3 Enabled */
        w.tag_bool(b"XDNS", false);
    }
}
