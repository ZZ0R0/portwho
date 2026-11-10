use crate::utils::{NetSocket, Port, Process, Processes};

use comfy_table::{presets::UTF8_FULL, Table};
use serde_json::{json, Value};

fn state_name(socket: &NetSocket) -> &'static str {
    if socket.proto.is_tcp() {
        match socket.state {
            0x01 => "ESTABLISHED",
            0x02 => "SYN_SENT",
            0x03 => "SYN_RECV",
            0x04 => "FIN_WAIT1",
            0x05 => "FIN_WAIT2",
            0x06 => "TIME_WAIT",
            0x07 => "CLOSE",
            0x08 => "CLOSE_WAIT",
            0x09 => "LAST_ACK",
            0x0A => "LISTEN",
            0x0B => "CLOSING",
            0x0C => "NEW_SYN_RECV",
            _ => "UNKNOWN",
        }
    } else {
        match socket.state {
            0x01 => "ESTABLISHED",
            0x07 => "UNCONN",
            _ => "UNKNOWN",
        }
    }
}

fn sorted_sockets<'a>(sockets: impl IntoIterator<Item = &'a NetSocket>) -> Vec<&'a NetSocket> {
    let mut items: Vec<_> = sockets.into_iter().collect();

    items.sort_by(|a, b| {
        (a.proto.rank(), a.local, a.remote, a.inode).cmp(&(
            b.proto.rank(),
            b.local,
            b.remote,
            b.inode,
        ))
    });

    items
}

fn socket_record(socket: &NetSocket, processes: Option<&Processes>) -> Value {
    let mut pids = socket.process_pids.clone();

    pids.sort_unstable();
    pids.dedup();

    let owners: Vec<Value> = pids
        .iter()
        .map(|pid| match processes.and_then(|map| map.get(pid)) {
            Some(process) => json!({
                "pid": pid,
                "bin": process.bin,
                "cmdline": process.cmdline,
            }),
            None => json!({
                "pid": pid
            }),
        })
        .collect();

    json!({
        "proto": socket.proto.as_str(),
        "local": {
            "ip": socket.local.ip().to_string(),
            "port": socket.local.port()
        },
        "remote": {
            "ip": socket.remote.ip().to_string(),
            "port": socket.remote.port()
        },
        "state": state_name(socket),
        "state_code": socket.state,
        "uid": socket.uid,
        "inode": socket.inode,
        "processes": owners,
    })
}

fn print_json(value: Value) -> Result<(), String> {
    let output = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;

    println!("{output}");

    Ok(())
}

fn print_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let mut table = Table::new();

    table.load_style(UTF8_FULL);
    table.set_header(headers.iter().copied());

    for row in rows {
        table.add_row(row);
    }

    println!("{table}");
}

fn socket_cells(socket: &NetSocket) -> Vec<String> {
    vec![
        socket.proto.as_str().to_string(),
        socket.local.ip().to_string(),
        socket.local.port().to_string(),
        socket.remote.ip().to_string(),
        socket.remote.port().to_string(),
        state_name(socket).to_string(),
        socket.uid.to_string(),
        socket.inode.to_string(),
    ]
}

fn names_for_socket(socket: &NetSocket, processes: &Processes) -> (String, String) {
    let mut pids = socket.process_pids.clone();

    pids.sort_unstable();
    pids.dedup();

    if pids.is_empty() {
        return ("-".to_string(), "-".to_string());
    }

    let names = pids
        .iter()
        .map(|pid| {
            processes.get(pid).map_or_else(
                || "?".to_string(),
                |process| {
                    std::path::Path::new(&process.bin)
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .filter(|name| !name.is_empty())
                        .unwrap_or_else(|| "?".to_string())
                },
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    let pid_string = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");

    (pid_string, names)
}

pub fn display_sockets<'a>(
    sockets: impl IntoIterator<Item = &'a NetSocket>,
    json_output: bool,
) -> Result<(), String> {
    let sockets = sorted_sockets(sockets);

    if json_output {
        let records: Vec<_> = sockets
            .iter()
            .map(|socket| socket_record(socket, None))
            .collect();

        return print_json(json!({
            "command": "list",
            "count": records.len(),
            "sockets": records
        }));
    }

    let rows = sockets.iter().map(|socket| socket_cells(socket)).collect();

    print_table(
        &[
            "PROTO",
            "LOCAL ADDRESS",
            "LOCAL PORT",
            "REMOTE ADDRESS",
            "REMOTE PORT",
            "STATE",
            "UID",
            "INODE",
        ],
        rows,
    );

    Ok(())
}

pub fn display_port<'a>(
    port: Port,
    sockets: impl IntoIterator<Item = &'a NetSocket>,
    processes: &Processes,
    json_output: bool,
) -> Result<(), String> {
    let sockets = sorted_sockets(sockets);

    if json_output {
        let records: Vec<_> = sockets
            .iter()
            .map(|socket| socket_record(socket, Some(processes)))
            .collect();

        return print_json(json!({
            "command": "port",
            "port": port,
            "count": records.len(),
            "sockets": records
        }));
    }

    let rows = sockets
        .iter()
        .map(|socket| {
            let (pids, names) = names_for_socket(socket, processes);

            vec![
                socket.proto.as_str().to_string(),
                socket.local.ip().to_string(),
                socket.local.port().to_string(),
                socket.remote.ip().to_string(),
                socket.remote.port().to_string(),
                state_name(socket).to_string(),
                pids,
                names,
            ]
        })
        .collect();

    print_table(
        &[
            "PROTO",
            "LOCAL ADDRESS",
            "LOCAL PORT",
            "REMOTE ADDRESS",
            "REMOTE PORT",
            "STATE",
            "PID",
            "PROCESS",
        ],
        rows,
    );

    Ok(())
}

pub fn display_process<'a>(
    process: &Process,
    sockets: impl IntoIterator<Item = &'a NetSocket>,
    json_output: bool,
) -> Result<(), String> {
    let sockets = sorted_sockets(sockets);

    if json_output {
        let records: Vec<_> = sockets
            .iter()
            .map(|socket| socket_record(socket, None))
            .collect();

        return print_json(json!({
            "command": "pid",
            "pid": process.pid,
            "bin": process.bin,
            "cmdline": process.cmdline,
            "count": records.len(),
            "sockets": records
        }));
    }

    println!("PID:     {}", process.pid);
    println!("BINARY:  {}", process.bin);
    println!("CMDLINE: {}", process.cmdline);
    println!("SOCKETS: {}", sockets.len());

    let rows = sockets.iter().map(|socket| socket_cells(socket)).collect();

    print_table(
        &[
            "PROTO",
            "LOCAL ADDRESS",
            "LOCAL PORT",
            "REMOTE ADDRESS",
            "REMOTE PORT",
            "STATE",
            "UID",
            "INODE",
        ],
        rows,
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Proto;
    use std::net::SocketAddr;

    #[test]
    fn tcp_listen_state() {
        let socket = NetSocket {
            proto: Proto::Tcp,
            local: SocketAddr::from(([0, 0, 0, 0], 22)),
            remote: SocketAddr::from(([0, 0, 0, 0], 0)),
            state: 0x0a,
            uid: 0,
            inode: 1,
            process_pids: vec![],
        };

        assert_eq!(state_name(&socket), "LISTEN");

        assert_eq!(socket_record(&socket, None)["local"]["port"], 22);
    }
}
