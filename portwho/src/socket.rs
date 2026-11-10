use crate::cli::SocketFilters;
use crate::utils::{decode_endpoint, NetSocket, NetSockets, Proto};

use std::fs;
use std::io;

fn parse_socket_line(line: &str, proto: Proto) -> Result<NetSocket, String> {
    let columns: Vec<&str> = line.split_whitespace().collect();

    if columns.len() < 10 {
        return Err("invalid socket entry (too few columns)".to_string());
    }

    let ipv6 = matches!(proto, Proto::Tcp6 | Proto::Udp6);

    let local = decode_endpoint(columns[1], ipv6)?;
    let remote = decode_endpoint(columns[2], ipv6)?;

    let state = u8::from_str_radix(columns[3], 16).map_err(|e| e.to_string())?;

    let uid = columns[7].parse::<u32>().map_err(|e| e.to_string())?;

    let inode = columns[9].parse::<u64>().map_err(|e| e.to_string())?;

    Ok(NetSocket {
        proto,
        local,
        remote,
        state,
        uid,
        inode,
        process_pids: Vec::new(),
    })
}

fn parse_socket_table(content: &str, proto: Proto) -> NetSockets {
    let mut result = NetSockets::default();

    for line in content
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
    {
        if let Ok(socket) = parse_socket_line(line, proto) {
            result.insert(socket);
        }
    }

    result
}

pub fn get_proto_socks(proto: Proto) -> io::Result<NetSockets> {
    let path = match proto {
        Proto::Tcp => "/proc/net/tcp",
        Proto::Tcp6 => "/proc/net/tcp6",
        Proto::Udp => "/proc/net/udp",
        Proto::Udp6 => "/proc/net/udp6",
    };

    let content = fs::read_to_string(path)?;

    Ok(parse_socket_table(&content, proto))
}

pub fn collect_sockets(filters: &SocketFilters) -> Result<NetSockets, String> {
    let tcp = filters.tcp || !filters.udp;
    let udp = filters.udp || !filters.tcp;

    let ipv4 = filters.ipv4 || !filters.ipv6;
    let ipv6 = filters.ipv6 || !filters.ipv4;

    let mut sockets = NetSockets::default();

    for (enabled, proto) in [
        (tcp && ipv4, Proto::Tcp),
        (tcp && ipv6, Proto::Tcp6),
        (udp && ipv4, Proto::Udp),
        (udp && ipv6, Proto::Udp6),
    ] {
        if enabled {
            match get_proto_socks(proto) {
                Ok(found) => sockets.extend(found),

                Err(err) if err.kind() == io::ErrorKind::NotFound => {}

                Err(err) => {
                    return Err(format!("unable to read {}: {err}", proto.as_str()));
                }
            }
        }
    }

    Ok(sockets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tcp_row() {
        let line = concat!(
            "0: 0100007F:0016 00000000:0000 0A ",
            "00000000:00000000 00:00000000 ",
            "00000000 0 0 12345 1 0000000000000000"
        );

        let socket = parse_socket_line(line, Proto::Tcp).unwrap();

        assert_eq!(socket.local.to_string(), "127.0.0.1:22");
        assert_eq!(socket.state, 0x0a);
        assert_eq!(socket.inode, 12345);
    }
}
