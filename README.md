# qs - Quick SSH

Dead simple, zero-friction SSH wrapper that makes working with remote machines effortless. Automatic multiplexing, easy file transfers, no password hassles.

## Features

- **One-time setup**: Add a host once, never type passwords again
- **SSH multiplexing**: First connection opens, rest are instant
- **Simple file transfer**: `qs send file.txt /remote/` - that's it
- **Smart defaults**: Works without specifying hosts
- **Cross-platform**: macOS and Linux

## Prerequisites

- **Rust/Cargo** - [Install Rust](https://rustup.rs/)
- **SSH** - OpenSSH client (`ssh`, `ssh-keygen`)
- **rsync** - For file transfers

## Installation

```bash
cargo install qs
```

Or build from source:
```bash
cargo build --release
sudo cp target/release/qs /usr/local/bin/
```

## Usage

```bash
# Setup (one time)
qs init                                  # Create SSH keys
qs add HOST_NAME 192.168.1.100 --user bob # Add host (auto-copies SSH key)

# Daily use (no passwords!)
qs connect                               # SSH to default host
qs send project.tar.gz /tmp/            # Upload file
qs get /var/log/app.log ./              # Download file
qs exec "docker ps"                      # Run remote command

# Host management
qs list                                  # Show all hosts
qs remove HOST_NAME                      # Remove a host
qs set-default HOST_NAME                 # Change default host
```

For detailed usage and examples, see [USAGE.md](USAGE.md).

## Why?

Because `ssh user@192.168.1.100` and `scp -r ./folder user@192.168.1.100:/path/to/dest/` gets old fast.

With qs, it's just `qs connect` and `qs send folder /path/to/dest/`.
