mod config;
mod plugin;
mod proxy;

use color_eyre::eyre::{bail, Result};
use env_logger::Env;
use futures::future::join_all;
use proxy::TcpProxy;
use tokio::net::lookup_host;

use crate::plugin::Plugin;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter_or("PORTPROXY_LOG", "info")).init();
    color_eyre::install()?;

    log::info!("portproxy v{} starting...", VERSION);

    let cfg = config::load()?;

    let mut proxies;

    pyo3::prepare_freethreaded_python();

    match cfg.proxy() {
        None => bail!("No proxies defined in config"),
        Some(proxy_list) => {
            proxies = Vec::with_capacity(proxy_list.len());
            for p in proxy_list {
                let mut plugins: Vec<Plugin> = vec![];
                if let Some(configured_plugins) = p.plugins() {
                    for pl in configured_plugins {
                        plugins.push(Plugin::load(pl.path(), pl.config())?);
                    }
                }
                let resolved_addrs = lookup_host(p.listen()).await;
                match resolved_addrs {
                    Ok(addresses) => {
                        for listen in addresses {
                            log::debug!("Listen address {} resolved to {}", p.listen(), listen);
                            let proxy =
                                TcpProxy::new(listen, p.connect().to_string(), plugins.clone());
                            proxies.push(proxy);
                        }
                    }
                    Err(e) => log::error!("Failed to resolve {}: {}", p.listen(), e),
                }
            }
        }
    }

    join_all(proxies.iter().map(|p| p.run())).await;

    log::info!("Nothing left to do, exiting..");

    Ok(())
}
