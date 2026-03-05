set shell := ["bash", "-euo", "pipefail", "-c"]

default: help

help:
	@echo "atspicli development commands"
	@echo "  just bootstrap  # install-check and build once"
	@echo "  just check      # fmt + clippy + tests"
	@echo "  just test       # tests only"
	@echo "  just run list-apps"

bootstrap:
	./scripts/bootstrap-cli.sh

check:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --locked

test:
	cargo test --locked

run *args:
	cargo run --locked -- {{args}}
