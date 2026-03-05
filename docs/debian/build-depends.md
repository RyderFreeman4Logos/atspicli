# Debian Build and Runtime Dependencies

## Build-Depends
The following packages are required for building `atspicli`:

- `debhelper-compat (= 13)`
- `cargo`
- `rustc`
- `pkg-config`
- `libatspi2.0-dev (>= 2.46.0)`
- `libdbus-1-dev (>= 1.14.0)`
- `libglib2.0-dev (>= 2.74.0)`

## Depends (Runtime)
The following packages are required for running `atspicli`:

- `libatspi2-0`
- `libdbus-1-3`
- `libglib2.0-0`
- `dbus-user-session`
- `at-spi2-core`
