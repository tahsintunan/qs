# Development Guide

## Prerequisites

- **Rust** 1.70+ ([rustup.rs](https://rustup.rs/))
- **OpenSSH** client (`ssh`, `ssh-keygen`)
- **rsync** for testing file transfers

## Platform

- macOS
- Linux

## Build

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Install locally
make install
```

## Test

```bash
# Run tests
cargo test --test '*'

# Format code
cargo fmt

# Check for issues
cargo clippy

# Manual testing
cargo run -- check
cargo run -- init
cargo run -- add test --host localhost --user $USER
```

## Contributing

1. Fork and create a feature branch
2. Make changes and test (`cargo test`)
3. Add tests in the `tests/` folder only
4. Format code (`cargo fmt`)
5. Submit a PR with clear description
