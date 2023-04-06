# portproxy
Simple port forwarding tool, built with tokio-rs 🦀

[![github](https://img.shields.io/badge/github-spitfire05/portproxy-lightgrey?style=for-the-badge&logo=github)](https://github.com/spitfire05/portproxy)
[![Crates.io](https://img.shields.io/crates/v/portproxy?style=for-the-badge&logo=rust)](https://crates.io/crates/portproxy)

## What does it do?
Pretty much the very same thing as Windows' `netsh interface portproxy` or Linux's `iptables` forward - it maps the incoming connections from `listen` local adress & port to remote `connect` address and port.

Only TCP port forwarding is supported at this time.

This tool does not currently offer anything more over the native OS tools, maybe besides the unification and ease of defining the mappings. Other notable use case is those rare instances were native port forwarding tool cannot be used (for example, windows' `netsh portproxy` requires IPv6 to be enabled on the system).

## Install

### Cargo

```sh
cargo install portproxy
```

### From source

Clone the repo and compile it:
```sh
git clone https://github.com/spitfire05/portproxy.git
cd portproxy
cargo install
```

### Pre-compiled binaries

Windows binaries are avialable in the releases section of this repo.

## Configuration

Configuration is specified in the [TOML](https://toml.io/) language.

`portproxy` will try to read the config for paths in following order:

1) `--config` CLI arg, if set
2) Value of `PORTPROXY_CONFIG` env variable, if set
3) `~/.config/portproxy.toml`

Config should contain one or more `[[proxy]]` elements, that define the port mappings:

```toml
[[proxy]]
listen = "localhost:8080"        # local address to listen on
connect = "some-server.lan:8485" # remote (or local) address to connect to
```

## Run
Running is as simple as it can be - just call the `portproxy` binary. There are optional flags/parameters:

```
Usage: portproxy [OPTIONS]

Options:
  -c, --config-path <CONFIG_PATH>  Path to read the config from. If not set, will fall back to value of $PORTPROXY_CONFIG, and "~/.config/portproxy.toml", in that order
  -l, --log-level <LOG_LEVEL>      [default: info] [possible values: error, warn, info, debug, trace]
  -d, --log-dir <LOG_DIR>          Directory to write the log files to. Logging to file will be disabled if this is not set
  -h, --help                       Print help
  -V, --version                    Print version
```

### As a service/daemon

#### Windows

Use [Shawl](https://github.com/mtkennerly/shawl) to create a windows service of `portproxy`.

```
shawl add --no-restart --no-log --name portproxy -- C:\full\path\to\portproxy.exe --log-level debug --log-dir C:\full\path\to\logs\directory <optional args>
```

****
