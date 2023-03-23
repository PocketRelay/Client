use crate::{constants::QOS_PORT, show_error};
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
