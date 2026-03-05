#!/usr/bin/env bash
set -euo pipefail

./scripts/check-system-deps.sh
cargo fetch --locked
cargo build --locked
