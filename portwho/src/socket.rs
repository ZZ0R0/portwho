use crate::utils::{Proto, NetSocket, decode_endpoint};

use std::fs;
use std::io;

fn parse_socket_line(
    line: &str,
    proto: Proto,
) -> Result<NetSocket, String> {
    let columns: Vec<&str> = line.split_whitespace().collect();

    if columns.len() < 10 {
        return Err("Invalid socket entry".into());
    }

    let ipv6 = matches!(proto, Proto::Tcp6 | Proto::Udp6);

    let local = decode_endpoint(columns[1], ipv6)?;
    let remote = decode_endpoint(columns[2], ipv6)?;

    let state = u8::from_str_radix(columns[3], 16)
        .map_err(|e| e.to_string())?;

    let uid = columns[7]
        .parse::<u32>()
        .map_err(|e| e.to_string())?;

    let inode = columns[9]
        .parse::<u64>()
        .map_err(|e| e.to_string())?;

    Ok(NetSocket {
        proto,
        local,
        remote,
        state,
        uid,
        inode,
    })
}

pub fn list_proto_socks(
    protocol: Proto,
) -> io::Result<Vec<NetSocket>> {
    let filename = match protocol {
        Proto::Tcp => "/proc/net/tcp",
        Proto::Tcp6 => "/proc/net/tcp6",
        Proto::Udp => "/proc/net/udp",
        Proto::Udp6 => "/proc/net/udp6",
    };

    let content = fs::read_to_string(filename)?;
    let mut result = Vec::new();

    for (index, line) in content.lines().skip(1).enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        match parse_socket_line(line, protocol) {
            Ok(socket) => result.push(socket),

            Err(error) => {
                eprintln!(
                    "Failed to parse {} line {}: {}",
                    filename,
                    index + 2,
                    error
                );
            }
        }
    }

    Ok(result)
}