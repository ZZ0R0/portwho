
mod cli;
mod socket;
mod utils;
mod process;
mod output;

use clap::Parser;

use crate::cli::{Cli, Commands, SocketFilters};
use crate::utils::{NetSocket, Proto};
use crate::output::{display_sockets};
use crate::socket::*;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli) {
        eprintln!("portwho: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    // Do not silently ignore unsupported output formats.
    if cli.json {
        return Err("JSON output is not implemented yet".into());
    }

    match cli.command {
        Some(Commands::List { filters }) => {
            let sockets = collect_sockets(&filters)?;
            display_sockets(&sockets);
        }

        Some(Commands::Port { port, filters }) => {
            let sockets = collect_sockets(&filters)?;

            let matches: Vec<NetSocket> = sockets
                .into_iter()
                .filter(|socket| socket.local.port() == port)
                .collect();

            display_sockets(&matches);
        }

        Some(Commands::Pid { pid , filters}) => {
            return Err(format!(
                "PID inspection is not implemented yet (PID: {pid})"
            ));
        }

        None => {
            // Default behavior: list all sockets.
            let filters = SocketFilters::default();
            let sockets = collect_sockets(&filters)?;
            display_sockets(&sockets);
        }
    }

    Ok(())
}


fn collect_sockets(filters: &SocketFilters) -> Result<Vec<NetSocket>, String> {
    let mut sockets = Vec::new();

    let tcp = filters.tcp || !filters.udp;
    let udp = filters.udp || !filters.tcp;

    let ipv4 = filters.ipv4 || !filters.ipv6;
    let ipv6 = filters.ipv6 || !filters.ipv4;

    if tcp && ipv4 {
        sockets.extend(list_proto_socks(Proto::Tcp).map_err(|e| e.to_string())?);
    }

    if tcp && ipv6 {
        sockets.extend(list_proto_socks(Proto::Tcp6).map_err(|e| e.to_string())?);
    }

    if udp && ipv4 {
        sockets.extend(list_proto_socks(Proto::Udp).map_err(|e| e.to_string())?);
    }

    if udp && ipv6 {
        sockets.extend(list_proto_socks(Proto::Udp6).map_err(|e| e.to_string())?);
    }

    Ok(sockets)
}