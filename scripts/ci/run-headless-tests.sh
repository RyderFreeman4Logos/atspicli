#!/usr/bin/env bash
set -euo pipefail

if ! command -v dbus-run-session >/dev/null 2>&1; then
  echo "dbus-run-session is required for headless tests"
  exit 1
fi

if [[ "${ATSPICLI_HEADLESS_INNER:-0}" != "1" ]]; then
  if command -v xvfb-run >/dev/null 2>&1; then
    dbus-run-session -- env ATSPICLI_HEADLESS_INNER=1 xvfb-run -a "$0"
  else
    dbus-run-session -- env ATSPICLI_HEADLESS_INNER=1 "$0"
  fi
  exit $?
fi

cargo test --test readonly_commands --locked
cargo test --test action_phase_a --locked
