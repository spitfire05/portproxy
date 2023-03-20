mod config;
mod proxy;

use env_logger::Env;
use futures::future::join_all;
use miette::{bail, Result};
use proxy::TcpProxy;
use tokio::net::lookup_host;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().filter_or("PORTPROXY_LOG", "info")).init();

    log::info!("portproxy v{} starting...", VERSION);

    let cfg = config::load()?;

    let mut proxies;

    match cfg.proxy() {
        None => bail!("No proxies defined in config"),
        Some(proxy_list) => {
            proxies = Vec::with_capacity(proxy_list.len());
            for p in proxy_list {
                let resolved_addrs = lookup_host(p.listen()).await;
                match resolved_addrs {
                    Ok(addresses) => {
                        for listen in addresses {
                            log::debug!("Listen address {} resolved to {}", p.listen(), listen);
                            let proxy = TcpProxy::new(listen, p.connect().to_string());
                            proxies.push(proxy);
                        }
                    }
                    Err(e) => log::error!("Failed to resolve {}: {}", p.listen(), e),
                }
            }
        }
    }

    join_all(proxies.iter().map(proxy::TcpProxy::run)).await;

    log::info!("Nothing left to do, exiting..");

    Ok(())
}
