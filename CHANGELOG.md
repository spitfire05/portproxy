# Unreleased

## Fixed

- Updated dependencies to fix some potential security issues

# 0.3.1

## Fixed

- File log output no should no longer contain ANSI escape characters

# 0.3.0

## Added

- Command line options `--log-level` & `--config`
- Command line option `--log-dir`

## Removed

- Log level no longer controller by `PORTPROXY_LOG` env variable

## Changed

- Logs are now more structured

## Fixed

- `ctrl-c` shutdown should now be graceful on all platforms

# 0.2.0

## Changed

- Log level is now controlled by `PORPROXY_LOG` env variable, instead of `RUST_LOG`

# 0.1.0

The initial relase.
