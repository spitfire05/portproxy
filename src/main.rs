mod config;
mod proxy;

use color_eyre::eyre::{bail, Context, Result};
use env_logger::Env;
use futures::future::join_all;
use proxy::TcpProxy;
use tokio::net::lookup_host;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    color_eyre::install()?;

    log::info!("portproxy v{} starting...", VERSION);

    let cfg = config::load()?;

    let mut proxies;

    match cfg.proxy() {
        None => bail!("No proxies defined in config"),
        Some(proxy_list) => {
            proxies = Vec::with_capacity(proxy_list.len());
            for p in proxy_list {
                for listen in lookup_host(p.listen())
                    .await
                    .wrap_err_with(|| format!("Failed to resolve {}", p.listen()))?
                {
                    log::debug!("Listen address {} resolved to {}", p.listen(), listen);
                    let proxy = TcpProxy::new(listen, p.connect().to_string());
                    proxies.push(proxy);
                }
            }
        }
    }

    join_all(proxies.iter().map(|p| p.run())).await;

    Ok(())
}
