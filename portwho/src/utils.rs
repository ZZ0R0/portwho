use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use std::fs;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proto {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

pub type Port = u16;
pub type Address = IpAddr;
pub type Endpoint = SocketAddr;

#[derive(Debug)]
pub struct NetSocket {
    pub proto: Proto,
    pub local: Endpoint,
    pub remote: Endpoint,
    pub state: u8,
    pub uid: u32,
    pub inode: u64,
}

pub fn decode_ipv4(hex: &str) -> Result<Ipv4Addr, std::num::ParseIntError> {
    let raw = u32::from_str_radix(hex, 16)?;
    Ok(Ipv4Addr::from(raw.to_le_bytes()))
}

pub fn decode_ipv6(hex: &str) -> Result<Ipv6Addr, std::num::ParseIntError> {
    let mut bytes = [0u8; 16];

    for i in 0..4 {
        let raw = u32::from_str_radix(&hex[i * 8..(i + 1) * 8], 16)?;
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&raw.to_le_bytes());
    }

    Ok(Ipv6Addr::from(bytes))
}

pub fn decode_port(hex: &str) -> Result<u16, std::num::ParseIntError> {
    u16::from_str_radix(hex, 16)
}

pub fn decode_endpoint(
    field: &str,
    ipv6: bool,
) -> Result<SocketAddr, String> {
    let (ip_hex, port_hex) = field
        .split_once(':')
        .ok_or("Missing IP:PORT separator")?;

    let ip = if ipv6 {
        IpAddr::V6(
            decode_ipv6(ip_hex).map_err(|e| e.to_string())?
        )
    } else {
        IpAddr::V4(
            decode_ipv4(ip_hex).map_err(|e| e.to_string())?
        )
    };

    let port = decode_port(port_hex)
        .map_err(|e| e.to_string())?;

    Ok(SocketAddr::new(ip, port))
}