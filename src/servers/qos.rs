use crate::{constants::QOS_PORT, net::public_address, show_error};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
};
use tokio::net::UdpSocket;

/// Starts the Quality Of Service server which handles providing the public
/// address values to the clients that connect.
pub async fn start_server() {
    let socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, QOS_PORT)).await {
        Ok(value) => value,
        Err(err) => {
            let text = format!("Failed to start qos: {}", err);
            show_error("Failed to start", &text);
            exit(1);
        }
    };

    // Buffer for the heading portion of the incoming message
    let mut buffer = [0u8; 20];
    // Buffer for the output message
    let mut output = [0u8; 30];

    loop {
        let (_, addr) = match socket.recv_from(&mut buffer).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        let address = match get_address(&addr).await {
            Some(value) => value,
            None => continue,
        };

        let port = addr.port().to_be_bytes();
        let address = address.octets();

        // Copy the heading from the read buffer
        output[..20].copy_from_slice(&buffer);

        // Copy the address bytes
        output[20..24].copy_from_slice(&address);

        // Copy the port bytes
        output[24..26].copy_from_slice(&port);

        // Fill remaining contents
        output[26..].copy_from_slice(&[0, 0, 0, 0]);

        // Send output response
        let _ = socket.send_to(&output, addr).await;
    }
}

/// Gets the public address for the provided socket address. Non Ipv4 addresses
/// fail returning None. Loopback and private addresses use the resolved public
/// address of the server and the rest are used directly
///
/// `addr` The address to get the public addr for
async fn get_address(addr: &SocketAddr) -> Option<Ipv4Addr> {
    let ip = addr.ip();
    if let IpAddr::V4(value) = ip {
        // Attempt to lookup machine public address to use
        if value.is_loopback() || value.is_private() {
            if let Some(public_addr) = public_address().await {
                return Some(public_addr);
            }
        }
        return Some(value);
    }
    None
}
