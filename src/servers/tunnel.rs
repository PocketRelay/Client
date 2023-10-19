use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream},
    os::windows::prelude::RawSocket,
};

use log::debug;
use pnet_packet::{ip::IpNextHeaderProtocols, ipv4, tcp::TcpPacket, udp::UdpPacket};

pub fn start_tunnel() {
    // TODO: RESOLVE THIS AT RUNTIME USING TARGET
    let mut target = TcpStream::connect(("127.0.0.1", 9887)).unwrap();
    let mut virtual_addr_octets = [0u8; 4];
    target.read_exact(&mut virtual_addr_octets).unwrap();
    let virtual_address = Ipv4Addr::from(virtual_addr_octets);

    debug!("Server assigned address: {}", virtual_address);

    let mut config = tun::Configuration::default();
    config
        .name("PR Tunnel")
        .address(virtual_address)
        .netmask((255, 255, 0, 0))
        .up();

    let mut dev = tun::create(&config).unwrap();
    let mut buf = [0; 65507];

    // TODO: Read from target stream to get packets that are supposed to be sent

    loop {
        let count = dev.read(&mut buf).unwrap();
        if count < 4 {
            continue;
        }
        let buf = &buf[..count];

        // Ignore non IPv4 packets
        if buf[0] >> 4 != 4 {
            continue;
        }

        // Parse as Ipv4 packet
        let ip_packet = ipv4::Ipv4Packet::new(buf).expect("Invalid Ipv4 packet");

        // Only handle udp packets
        if ip_packet.get_next_level_protocol() != IpNextHeaderProtocols::Udp {
            continue;
        }

        let udp_packet = UdpPacket::new(&buf[20..]).expect("Invalid udp packet");

        let dest = ip_packet.get_destination();

        debug!(
            "Sending packet to {}:{} ({}b)",
            &dest,
            &udp_packet.get_destination(),
            buf.len() - 28
        );

        // let mut out = Vec::new();
        // out.extend_from_slice(&virtual_address.octets());
        // out.extend_from_slice(&dest.octets());
        // out.extend_from_slice(&udp_packet.get_source().to_be_bytes());
        // out.extend_from_slice(&udp_packet.get_destination().to_be_bytes());
        // out.extend_from_slice(&buf[28..]);

        // target.write_all(&out).unwrap();
    }
}
