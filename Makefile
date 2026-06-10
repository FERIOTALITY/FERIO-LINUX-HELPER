.PHONY: help dev check build run clean watch test

# Default target: show help
help:
	@echo "================ ferio-linux-helper Developer Shortcuts ================"
	@echo "  make dev      - Run in developer mode (no root check, dry-run execution)"
	@echo "  make check    - Fast syntax & type check (cargo check)"
	@echo "  make run      - Run with normal privileges (requires root/sudo)"
	@echo "  make build    - Build the project in debug mode"
	@echo "  make test     - Run unit tests"
	@echo "  make watch    - Start hot-reload loop (requires cargo-watch installed)"
	@echo "  make clean    - Clean cargo build artifacts"
	@echo "========================================================================"

# Run in developer mode (no root required, dry-run makes it safe)
dev:
	cargo run -- --no-root --dry-run

# Run normally
run:
	cargo run

# Fast check
check:
	cargo check

# Standard build
build:
	cargo build

# Run tests
test:
	cargo test

# Auto-recompile and run on file save (requires `cargo install cargo-watch`)
watch:
	cargo watch -q -c -x "run -- --no-root --dry-run"

# Clean artifacts
clean:
	cargo clean
