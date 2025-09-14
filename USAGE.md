# Detailed Usage Guide

## Install

```bash
# Install directly from Cargo
cargo install qs

# Or build from source
cargo build --release
sudo cp target/release/qs /usr/local/bin/

# Or use make tool
make install
```

## Quick Start

```bash
# 1. Check dependencies (optional)
qs check

# 2. Initialize SSH keys
qs init

# 3. Add your machines (enter password once when prompted)
qs add myserver --host 192.168.1.100 --user username

# 4. Use it!
qs connect              # Connect to default profile
qs send file.txt /tmp/  # Send file
qs get /etc/hosts ./    # Get file
qs exec "docker ps"     # Run command
```

## Commands

### Setup Commands

```bash
qs check                                                  # Check if ssh, rsync are installed
qs init                                                   # Create SSH keys if needed
qs add <alias> --host <host> --user <username>            # Add profile with alias & copy SSH key
qs add webserver --host 10.0.0.5 --user bob --skip-key    # Add without key setup
qs add database --host 10.0.0.6 --user alice --is-default # Add and make default
qs add webserver --host 10.0.0.8 --user admin --overwrite # Replace existing alias
qs add staging --host 10.0.0.10 --user dev --port 2222    # Add with custom SSH port
qs remove webserver                                       # Remove alias 'webserver' (asks for confirmation)
qs remove webserver -y                                    # Remove alias without confirmation
qs set-default database                                   # Set 'database' as default profile
```

### Daily Use

```bash
# Connect
qs connect              # Connect to default profile
qs connect webserver    # Connect to specific profile

# Transfer files (uses rsync with progress)
qs send file.txt /remote/path/
qs send folder/ webserver:/backup/
qs get /var/log/app.log ./
qs get database:/data/dump.sql ./backups/

# Execute commands
qs exec "ls -la"
qs exec webserver "nvidia-smi"
qs exec "cd /app && docker-compose up -d"

# Manage hosts
qs list                 # Show all configured aliases
qs status               # Check default connection
qs status webserver     # Check specific alias connection
```

## How It Works

1. **SSH Multiplexing**: First connection creates a master socket in `~/.ssh/sockets/`. All subsequent operations reuse it (instant, no auth).

2. **Auto Key Setup**: `qs add` automatically copies your SSH key to the remote host. No more password typing. You can use the `--skip-key` flag to avoid this step.

3. **Smart Defaults**: First alias becomes default. Most commands work without specifying an alias.

4. **Config Location**: `~/.config/qs/config.toml`

## Config Example

```toml
default = "webserver"

[profiles.webserver]
host = "192.168.1.100"
user = "bob"

[profiles.database]
host = "10.0.0.50"
user = "admin"
```

## Tips

- Connection stays alive for 10 minutes after last use
- Use `alias:path` syntax to specify different hosts in file operations
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
