
use comfy_table::{Table, presets::UTF8_FULL};

use crate::utils::NetSocket;

pub fn display_sockets(sockets: &[NetSocket]) {
    let mut table = Table::new();

    table.load_style(UTF8_FULL);

    table.set_header(vec![
        "PROTO",
        "LOCAL ADDRESS",
        "LOCAL PORT",
        "REMOTE ADDRESS",
        "REMOTE PORT",
        "STATE",
        "UID",
        "INODE",
    ]);

    for socket in sockets {
        table.add_row(vec![
            format!("{:?}", socket.proto),
            socket.local.ip().to_string(),
            socket.local.port().to_string(),
            socket.remote.ip().to_string(),
            socket.remote.port().to_string(),
            format!("{:02X}", socket.state),
            socket.uid.to_string(),
            socket.inode.to_string(),
        ]);
    }

    println!("{table}");
}
