use rustc_hash::FxHashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

pub type Pid = u32;
pub type Inode = u64;
pub type Port = u16;
pub type Endpoint = SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Proto {
    Tcp,
    Tcp6,
    Udp,
    Udp6,
}

impl Proto {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp4",
            Self::Tcp6 => "tcp6",
            Self::Udp => "udp4",
            Self::Udp6 => "udp6",
        }
    }

    pub fn rank(self) -> u8 {
        match self {
            Self::Tcp => 0,
            Self::Tcp6 => 1,
            Self::Udp => 2,
            Self::Udp6 => 3,
        }
    }

    pub fn is_tcp(self) -> bool {
        matches!(self, Self::Tcp | Self::Tcp6)
    }
}

#[derive(Debug)]
pub struct NetSocket {
    pub proto: Proto,
    pub local: Endpoint,
    pub remote: Endpoint,
    pub state: u8,
    pub uid: u32,
    pub inode: Inode,
    pub process_pids: Vec<Pid>,
}

#[derive(Debug, Default)]
pub struct NetSockets {
    indexed: FxHashMap<Inode, NetSocket>,
    anonymous: Vec<NetSocket>,
}

impl NetSockets {
    pub fn insert(&mut self, socket: NetSocket) {
        if socket.inode == 0 {
            self.anonymous.push(socket);
        } else {
            self.indexed.insert(socket.inode, socket);
        }
    }

    pub fn extend(&mut self, other: NetSockets) {
        self.indexed.extend(other.indexed);
        self.anonymous.extend(other.anonymous);
    }

    pub fn get_mut(&mut self, inode: &Inode) -> Option<&mut NetSocket> {
        self.indexed.get_mut(inode)
    }

    pub fn values(&self) -> impl Iterator<Item = &NetSocket> {
        self.indexed.values().chain(self.anonymous.iter())
    }
}

#[derive(Debug)]
pub struct Process {
    pub pid: Pid,
    pub bin: String,
    pub cmdline: String,
    pub socket_inodes: Vec<Inode>,
}

pub type Processes = FxHashMap<Pid, Process>;

pub fn decode_ipv4(hex: &str) -> Result<Ipv4Addr, String> {
    if hex.len() != 8 {
        return Err(format!("invalid IPv4 field length: {}", hex.len()));
    }

    let raw = u32::from_str_radix(hex, 16).map_err(|e| e.to_string())?;

    Ok(Ipv4Addr::from(raw.to_le_bytes()))
}

pub fn decode_ipv6(hex: &str) -> Result<Ipv6Addr, String> {
    if hex.len() != 32 {
        return Err(format!("invalid IPv6 field length: {}", hex.len()));
    }

    let mut bytes = [0u8; 16];

    for i in 0..4 {
        let raw = u32::from_str_radix(&hex[i * 8..(i + 1) * 8], 16).map_err(|e| e.to_string())?;

        bytes[i * 4..(i + 1) * 4].copy_from_slice(&raw.to_le_bytes());
    }

    Ok(Ipv6Addr::from(bytes))
}

pub fn decode_endpoint(field: &str, ipv6: bool) -> Result<SocketAddr, String> {
    let (ip_hex, port_hex) = field
        .split_once(':')
        .ok_or_else(|| "missing IP:PORT separator".to_string())?;

    let ip = if ipv6 {
        IpAddr::V6(decode_ipv6(ip_hex)?)
    } else {
        IpAddr::V4(decode_ipv4(ip_hex)?)
    };

    let port = u16::from_str_radix(port_hex, 16).map_err(|e| e.to_string())?;

    Ok(SocketAddr::new(ip, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_proc_encoding() {
        assert_eq!(decode_ipv4("0100007F").unwrap().to_string(), "127.0.0.1");

        assert!(decode_ipv4("00000").is_err());
    }

    #[test]
    fn ipv6_proc_encoding() {
        assert_eq!(
            decode_ipv6("00000000000000000000000001000000").unwrap(),
            Ipv6Addr::LOCALHOST
        );

        assert!(decode_ipv6("XYZ").is_err());
    }

    #[test]
    fn preserve_anonymous_sockets() {
        let make_socket = |port| NetSocket {
            proto: Proto::Udp,
            local: SocketAddr::from(([0, 0, 0, 0], port)),
            remote: SocketAddr::from(([0, 0, 0, 0], 0)),
            state: 7,
            uid: 0,
            inode: 0,
            process_pids: vec![],
        };

        let mut sockets = NetSockets::default();

        sockets.insert(make_socket(53));
        sockets.insert(make_socket(5353));

        assert_eq!(sockets.values().count(), 2);
    }
}
