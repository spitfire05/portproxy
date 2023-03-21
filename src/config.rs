use derive_getters::Getters;
use miette::{Diagnostic, Result};
use serde::Deserialize;
use std::{env, fs};
use thiserror::Error;

#[derive(Deserialize, Getters)]
pub struct Config {
    proxy: Option<Vec<Proxy>>,
}

#[derive(Deserialize, Getters)]
pub struct Proxy {
    listen: String,
    connect: String,
}

#[derive(Error, Diagnostic, Debug)]
pub enum Error {
    #[error("Failed to get config path")]
    ConfigPathNotFound,

    #[diagnostic(help(
        "You can set specific config file to use with `--config <PATH>`, \
        or by setting the `PORTPROXY_CONFIG` environment variable"
    ))]
    #[error(r#"Can not open file "{path}""#)]
    Io {
        path: String,
        source: std::io::Error,
    },

    #[error(r#"Parsing config file "{path}" failed"#)]
    Parse {
        path: String,
        source: toml::de::Error,
    },
}

pub fn get_config_path() -> Option<String> {
    if let Ok(path) = env::var("PORTPROXY_CONFIG") {
        // Use $PORTPROXY_CONFIG as the config path if available
        tracing::debug!("PORTPROXY_CONFIG is set: {}", &path);

        Some(path)
    } else {
        // Default to using ~/.config/portproxy.toml
        tracing::debug!("PORTPROXY_CONFIG is not set");
        let config_path = dirs::home_dir()?.join(".config/portproxy.toml");
        let config_path_str = config_path.to_str()?.to_owned();
        tracing::debug!("Using default config path: {}", config_path_str);

        Some(config_path_str)
    }
}

pub fn load(path: Option<String>) -> Result<Config, Error> {
    let path = path.unwrap_or(get_config_path().ok_or(Error::ConfigPathNotFound)?);
    let cfg_str = fs::read_to_string(&path).map_err(|e| Error::Io {
        path: path.clone(),
        source: e,
    })?;
    let cfg: Config = toml::from_str(&cfg_str).map_err(|e| Error::Parse { path, source: e })?;

    Ok(cfg)
}
