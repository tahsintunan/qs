# qs - Quick SSH

## Install

```bash
# Clone and build
cargo build --release
sudo cp target/release/qs /usr/local/bin/

# Or install directly
cargo install --path .
```

## Quick Start

```bash
# 1. Check dependencies (optional)
qs check

# 2. Initialize SSH keys
qs init

# 3. Add your machines
qs add HOST_NAME 192.168.1.100 --user chief
# Enter password once when prompted

# 4. Use it!
qs connect              # Connect to default host
qs send file.txt /tmp/  # Send file
qs get /etc/hosts ./    # Get file
qs exec "docker ps"     # Run command
```

## Commands

### Setup Commands

```bash
qs check                          # Check if ssh, rsync are installed
qs init                           # Create SSH keys if needed
qs add <name> <host> [--user u]     # Add host & copy SSH key
qs add HOST_NAME 10.0.0.5 --skip-key  # Add without key setup
qs add HOST_NAME 10.0.0.6 --is-default # Add and make default
qs remove HOST_NAME                   # Remove a host
qs set-default HOST_NAME               # Change default host
```

### Daily Use

```bash
# Connect
qs connect              # Default host
qs connect HOST_NAME    # Specific host

# Transfer files (uses rsync with progress)
qs send file.txt /remote/path/
qs send folder/ HOST_NAME:/backup/
qs get /var/log/app.log ./
qs get HOST_NAME:/data/dump.sql ./backups/

# Execute commands
qs exec "ls -la"
qs exec HOST_NAME "nvidia-smi"
qs exec "cd /app && docker-compose up -d"

# Manage hosts
qs list                 # Show all hosts
qs status               # Default host connection
qs status HOST_NAME     # Specific host
```

## How It Works

1. **SSH Multiplexing**: First connection creates a master socket in `~/.ssh/sockets/`. All subsequent operations reuse it (instant, no auth).

2. **Auto Key Setup**: `qs add` automatically copies your SSH key to the remote host. No more password typing.

3. **Smart Defaults**: First host becomes default. Most commands work without specifying host.

4. **Config Location**: `~/.config/qs/config.toml`

## Config Example

```toml
default = "HOST_NAME"

[hosts.HOST_NAME]
host = "192.168.1.100"
user = "chief"

[hosts.HOST_NAME_2]
host = "10.0.0.50"
user = "admin"
```

## Tips

- Connection stays alive for 10 minutes after last use
- Use `host:path` syntax to specify different hosts in file operations
- All rsync flags: `-avz --progress` (archive, verbose, compress, progress bar)
- Works on macOS and Linux (checks for dependencies)

## Troubleshooting

```bash
# If connection seems stuck
qs status               # Check if active
ssh -O exit user@host   # Kill master connection manually

# If key copy fails
cat ~/.ssh/id_ed25519.pub | ssh user@host 'cat >> ~/.ssh/authorized_keys'

# Missing tools?
qs check               # Shows what to install
```
