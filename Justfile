set shell := ["bash", "-euo", "pipefail", "-c"]

preflight-debian:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --locked
	cargo audit
	cargo deny check
	dpkg-buildpackage -us -uc -b
