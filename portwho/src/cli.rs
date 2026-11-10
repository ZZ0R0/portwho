use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "portwho",
    version,
    about = "Inspect Linux network sockets and processes"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Print machine-readable JSON
    #[arg(short, long, global = true)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List network sockets without scanning processes
    List {
        #[command(flatten)]
        filters: SocketFilters,
    },

    /// Inspect sockets using a local port
    Port {
        #[arg(value_parser = clap::value_parser!(u16).range(1..))]
        port: u16,

        #[command(flatten)]
        filters: SocketFilters,
    },

    /// Inspect sockets owned by a process
    Pid {
        pid: u32,

        #[command(flatten)]
        filters: SocketFilters,
    },
}

#[derive(Debug, Args, Default)]
pub struct SocketFilters {
    /// Include TCP sockets
    #[arg(long)]
    pub tcp: bool,

    /// Include UDP sockets
    #[arg(long)]
    pub udp: bool,

    /// Include IPv4 sockets
    #[arg(short = '4', long)]
    pub ipv4: bool,

    /// Include IPv6 sockets
    #[arg(short = '6', long)]
    pub ipv6: bool,
}
