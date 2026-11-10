# portwho

A lightweight Linux CLI tool written in Rust to inspect network sockets and their associated processes.

## Features

- Inspect TCP and UDP sockets
- IPv4 and IPv6 support
- Filter sockets by local port or PID
- Display socket addresses, states, UIDs and inodes

## Installation

```bash
git clone <repository-url>
cd portwho
cargo build --release
```

## Usage

```bash
portwho list                  # List all sockets
portwho list --tcp            # TCP only
portwho list --udp -4         # UDP IPv4 only
portwho port 8080             # Inspect local port
portwho port 8080 --tcp -6    # TCP IPv6 on port 8080
portwho pid 1234              # Inspect process sockets
portwho --help                # Show help
```

## Requirements

- Linux
- Rust (Cargo)

## License

MIT
