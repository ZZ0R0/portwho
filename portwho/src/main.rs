mod cli;
mod matching;
mod output;
mod process;
mod socket;
mod utils;

use clap::Parser;

use crate::cli::{Cli, Commands, SocketFilters};
use crate::matching::match_sockets_processes;
use crate::output::{display_port, display_process, display_sockets};
use crate::process::{get_process, get_processes};
use crate::socket::collect_sockets;
use crate::utils::Processes;

fn main() {
    let cli = Cli::parse();

    if let Err(error) = run(cli) {
        eprintln!("portwho: {error}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    let json = cli.json;

    match cli.command {
        Some(Commands::List { filters }) => {
            let sockets = collect_sockets(&filters)?;
            display_sockets(sockets.values(), json)?;
        }

        Some(Commands::Port { port, filters }) => {
            let mut sockets = collect_sockets(&filters)?;

            if sockets.values().any(|socket| socket.local.port() == port) {
                let processes = get_processes()?;

                match_sockets_processes(&mut sockets, &processes);

                display_port(
                    port,
                    sockets
                        .values()
                        .filter(|socket| socket.local.port() == port),
                    &processes,
                    json,
                )?;
            } else {
                display_port(port, std::iter::empty(), &Processes::default(), json)?;
            }
        }

        Some(Commands::Pid { pid, filters }) => {
            let process = get_process(pid)?;
            let mut sockets = collect_sockets(&filters)?;

            let mut processes = Processes::default();
            processes.insert(pid, process);

            match_sockets_processes(&mut sockets, &processes);

            let process = processes.get(&pid).expect("process was just inserted");

            display_process(
                process,
                sockets
                    .values()
                    .filter(|socket| socket.process_pids.contains(&pid)),
                json,
            )?;
        }

        None => {
            let sockets = collect_sockets(&SocketFilters::default())?;
            display_sockets(sockets.values(), json)?;
        }
    }

    Ok(())
}
