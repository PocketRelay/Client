use crate::{
    constants::{MAIN_PORT, REDIRECTOR_PORT},
    show_error,
};
use blaze_pk::{
    codec::Encodable,
    packet::{PacketCodec, PacketComponents},
    writer::TdfWriter,
    PacketComponent, PacketComponents,
};
use blaze_ssl_async::{BlazeAccept, BlazeListener};
use futures_util::{SinkExt, StreamExt};
use std::{io, net::Ipv4Addr, process::exit, time::Duration};
use tokio::{select, time::sleep};
use tokio_util::codec::Framed;

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
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Packet components this server knows how to deal with
#[derive(Debug, Hash, PartialEq, Eq, PacketComponents)]
pub enum Components {
    /// Redirector component
    #[component(target = 0x5)]
    Redirector(Redirector),
}

/// Commands within the Redirector component that we can handle
#[derive(Debug, Hash, PartialEq, Eq, PacketComponent)]
pub enum Redirector {
    /// Command requesting the server instance
    #[command(target = 0x1)]
    GetServerInstance,
}

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

        if Components::from_header(&packet.header).is_none() {
            // Empty response for packets that aren't asking to redirect
            framed.send(packet.respond_empty()).await?;
            continue;
        }

        // Response with the instance details
        let response = packet.respond(LocalInstance {});
        framed.send(response).await?;
        break;
    }

    Ok(())
}

/// Packet contents for providing the redirection details
/// for 127.0.0.1 to allow proxying
pub struct LocalInstance;

impl Encodable for LocalInstance {
    fn encode(&self, writer: &mut TdfWriter) {
        writer.tag_union_start(b"ADDR", 0x0); /* Server address type */

        // Encode the net address portion
        writer.tag_group(b"VALU");
        writer.tag_u32(b"IP", u32::from_be_bytes([127, 0, 0, 1]));
        writer.tag_u16(b"PORT", MAIN_PORT);
        writer.tag_group_end();

        // Extra deatils
        writer.tag_bool(b"SECU", false); /* SSLv3 Enabled */
        writer.tag_bool(b"XDNS", false);
    }
}
