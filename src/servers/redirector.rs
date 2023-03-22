use crate::{
    components::{Components, Redirector},
    constants::{MAIN_PORT, REDIRECTOR_PORT},
    models::{InstanceDetails, InstanceNet, NetAddress},
};
use blaze_pk::packet::{Packet, PacketCodec, PacketComponents};
use blaze_ssl_async::{BlazeAccept, BlazeListener};
use futures::{SinkExt, StreamExt};
use std::{io, net::Ipv4Addr, time::Duration};
use tokio::{select, time::sleep};
use tokio_util::codec::Framed;

pub async fn start_server() {
    // Initializing the underlying TCP listener
    let listener = {
        match BlazeListener::bind(("0.0.0.0", REDIRECTOR_PORT)).await {
            Ok(value) => {
                println!("Started Redirector server (Port: {})", REDIRECTOR_PORT);
                value
            }
            Err(_) => {
                panic!("Failed to bind  server (Port: {})", REDIRECTOR_PORT)
            }
        }
    };

    // Accept incoming connections
    loop {
        let accept = match listener.accept().await {
            Ok(value) => value,
            Err(err) => {
                panic!("Failed to accept redirector connection: {err:?}");
            }
        };
        tokio::spawn(async move {
            if let Err(err) = handle_client(accept).await {
                panic!("Unable to handle redirect: {err}");
            };
        });
    }
}

/// The timeout before idle redirector connections are terminated
/// (1 minutes before disconnect timeout)
static DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Handles dealing with a redirector client
///
/// `stream`   The stream to the client
/// `addr`     The client address
/// `instance` The server instance information
async fn handle_client(accept: BlazeAccept) -> io::Result<()> {
    let (stream, addr) = match accept.finish_accept().await {
        Ok(value) => value,
        Err(_) => {
            panic!("Unable to establish SSL connection within redirector");
        }
    };

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

        let component = match Components::from_header(&packet.header) {
            Some(value) => value,
            // Don't know the component type send an empty response
            None => {
                framed.send(packet.respond_empty()).await?;
                continue;
            }
        };

        if let Components::Redirector(Redirector::GetServerInstance) = component {
            println!("Redirecting client (Addr: {addr:?})");

            let instance = InstanceDetails {
                net: InstanceNet {
                    host: crate::models::InstanceHost::Address(NetAddress(Ipv4Addr::LOCALHOST)),
                    port: MAIN_PORT,
                },
                secure: false,
            };

            let response = Packet::response(&packet, instance);
            framed.send(response).await?;
            break;
        } else {
            let response = Packet::response_empty(&packet);
            framed.send(response).await?;
        }
    }

    Ok(())
}
