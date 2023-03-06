use std::{
    env, fs,
    path::{Path, PathBuf},
};

use color_eyre::eyre::{eyre, Result, WrapErr};
use derive_getters::Getters;
use serde::Deserialize;

#[derive(Deserialize, Getters)]
pub struct Config {
    proxy: Option<Vec<Proxy>>,
}

#[derive(Deserialize, Getters)]
pub struct Proxy {
    listen: String,
    connect: String,
    plugins: Option<Vec<Plugin>>,
}

#[derive(Deserialize, Getters)]
pub struct Plugin {
    path: PathBuf,
    config: String,
}

pub fn get_config_path() -> Option<String> {
    if let Ok(path) = env::var("PORTPROXY_CONFIG") {
        // Use $PORTPROXY_CONFIG as the config path if available
        log::debug!("PORTPROXY_CONFIG is set: {}", &path);

        Some(path)
    } else {
        // Default to using ~/.config/portproxy.toml
        log::debug!("PORTPROXY_CONFIG is not set");
        let config_path = dirs::home_dir()?.join(".config/portproxy.toml");
        let config_path_str = config_path.to_str()?.to_owned();
        log::debug!("Using default config path: {}", config_path_str);

        Some(config_path_str)
    }
}

pub fn load() -> Result<Config> {
    let path = get_config_path().ok_or(eyre!("Failed to get config path"))?;
    let cfg_bytes = fs::read(&path).wrap_err_with(|| eyre!("Cannot open file \"{}\"", path))?;
    let cfg: Config =
        toml::from_slice(&cfg_bytes).wrap_err_with(|| eyre!("Cannot parse config"))?;

    Ok(cfg)
}
