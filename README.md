# portproxy
Simple port forwarding tool, built with tokio-rs 🦀

## What does it do?
Pretty much the very same thing as Windows' `netsh interface portproxy` or Linux's `iptables` forward - it maps the incoming connections from `listen` local adress & port to remote `connect` address and port.

Only TCP port forwarding is supported at this time.

This tool does not currently offer anything more over the native OS  tools, maybe besides the unification and ease of defining the mappings.

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
Config file will be read from `$HOME/.config/portproxy.toml`, or from path under `PORTPROXY_CONFIG` env variable, if it is set.

Config should contain one or more `[[proxy]]` elements, that define the port mappings:

```toml
[[proxy]]
listen = "localhost:8080"        # local address to listen on
connect = "some-server.lan:8485" # remote (or local) address to connect to
```

## Run
Running is as simple as it can be - just call the `portproxy` binary. There are no command line arguments, all coniguration is done in the config file.
