.PHONY: build install uninstall clean check

# Installation directory
PREFIX ?= /usr/local
BINDIR = $(PREFIX)/bin
BINARY = qs

build:
	cargo build --release

install: build
	@echo "Installing qs to $(BINDIR)..."
	@sudo mkdir -p $(BINDIR)
	@sudo cp target/release/$(BINARY) $(BINDIR)/
	@sudo chmod 755 $(BINDIR)/$(BINARY)
	@echo "✓ Installed successfully"
	@echo "Run 'qs init' to get started"

uninstall:
	@echo "Removing qs..."
	@sudo rm -f $(BINDIR)/$(BINARY)
	@rm -rf ~/.config/qs
	@echo "✓ Uninstalled"

clean:
	cargo clean

check:
	@target/release/$(BINARY) check || true

# Development
dev:
	cargo run -- $(ARGS)

test:
	cargo test

fmt:
	cargo fmt

# Quick reinstall for development
reinstall: uninstall install