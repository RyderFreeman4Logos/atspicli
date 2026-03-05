# atspicli

`atspicli` is a Debian-targeted Rust CLI for AT-SPI automation, aligned with the `axcli.rs` command model.

## Security and least privilege

- Run as a non-root user by default.
- Ensure the user has access to the session bus (`dbus-user-session`) and AT-SPI runtime (`at-spi2-core`).
- Do not run with elevated privileges unless absolutely required by the host policy.
- Sensitive nodes (for example password-like fields) are blocked from read and screenshot commands.

## Commands

- `snapshot`
- `click`
- `dblclick`
- `input`
- `fill`
- `press`
- `hover`
- `focus`
- `scroll-to`
- `scroll`
- `screenshot`
- `wait`
- `get`
- `list-apps`

## Development

Run local checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --locked
```
