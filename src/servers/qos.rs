use crate::{constants::QOS_PORT, ui::show_error};
use log::debug;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    process::exit,
    time::{Duration, SystemTime},
};
use tokio::{net::UdpSocket, sync::RwLock};

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
    let mut buffer = [0u8; 64];
    // Buffer for the output message
    let mut output = [0u8; 128];

    loop {
        let (count, addr) = match socket.recv_from(&mut buffer).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        let address = match public_address().await {
            Some(value) => value,
            None => {
                if let SocketAddr::V4(addr) = addr {
                    *addr.ip()
                } else {
                    Ipv4Addr::LOCALHOST
                }
            }
        };
        debug!("QOS IP: {}", address);
        debug!("QOS PORT: {}", addr.port());

        // Get the new buffer content
        let recv = &buffer[..count];

        // Get the address and port bytes
        let address = address.octets();
        let port = addr.port().to_be_bytes();

        // Compute the content lengths
        let addr_end = count + 4;
        let port_end = addr_end + 2;
        let total_length = port_end + 4;

        // Copy the output
        output[..count].copy_from_slice(recv);
        output[count..addr_end].copy_from_slice(&address);
        output[addr_end..port_end].copy_from_slice(&port);
        output[port_end..total_length].copy_from_slice(&[0, 0, 0, 0]);

        // Send output response
        let _ = socket.send_to(&output, addr).await;
    }
}

/// Caching structure for the public address value
enum PublicAddrCache {
    /// The value hasn't yet been computed
    Unset,
    /// The value has been computed
    Set {
        /// The public address value
        value: Ipv4Addr,
        /// The system time the cache expires at
        expires: SystemTime,
    },
}

/// Cache value for storing the public address
static PUBLIC_ADDR_CACHE: RwLock<PublicAddrCache> = RwLock::const_new(PublicAddrCache::Unset);

/// Cache public address for 30 minutes
const ADDR_CACHE_TIME: Duration = Duration::from_secs(60 * 30);

/// Retrieves the public address of the server either using the cached
/// value if its not expired or fetching the new value from the one of
/// two possible APIs
async fn public_address() -> Option<Ipv4Addr> {
    {
        let cached = &*PUBLIC_ADDR_CACHE.read().await;
        if let PublicAddrCache::Set { value, expires } = cached {
            let time = SystemTime::now();
            if time.lt(expires) {
                return Some(*value);
            }
        }
    }

    // Hold the write lock to prevent others from attempting to update aswell
    let cached = &mut *PUBLIC_ADDR_CACHE.write().await;

    // API addresses for IP lookup
    let addresses = ["https://api.ipify.org/", "https://ipv4.icanhazip.com/"];
    let mut value: Option<Ipv4Addr> = None;

    // Try all addresses using the first valid value
    for address in addresses {
        let response = match reqwest::get(address).await {
            Ok(value) => value,
            Err(_) => continue,
        };

        let ip = match response.text().await {
            Ok(value) => value.trim().replace('\n', ""),
            Err(_) => continue,
        };

        if let Ok(parsed) = ip.parse() {
            value = Some(parsed);
            break;
        }
    }

    // If we couldn't connect to any IP services its likely
    // we don't have internet lets try using our local address
    if value.is_none() {
        if let Ok(IpAddr::V4(addr)) = local_ip_address::local_ip() {
            value = Some(addr)
        }
    }

    let value = value?;

    // Update cached value with the new address

    *cached = PublicAddrCache::Set {
        value,
        expires: SystemTime::now() + ADDR_CACHE_TIME,
    };

    Some(value)
}
