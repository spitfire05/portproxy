mod proxy;

use std::fs;
use std::net::SocketAddr;

use color_eyre::eyre::{bail, eyre, Context, Result};
use env_logger::Env;
use futures::future::join_all;
use proxy::TcpProxy;
use serde::Deserialize;
use tokio::net::lookup_host;

#[derive(Deserialize)]
struct Config {
    proxy: Option<Vec<Proxy>>,
}

#[derive(Deserialize)]
struct Proxy {
    listen: String,
    connect: String,
}

async fn str_to_sock_addr(input: &str) -> Result<SocketAddr> {
    let results = lookup_host(input)
        .await
        .wrap_err_with(|| format!("Cannot resolve '{}' to socket address", input))?
        .take(1)
        .collect::<Vec<_>>();
    if results.is_empty() {
        return Err(eyre!("{} did not resolve as in IP address", input));
    }

    Ok(results[0])
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    color_eyre::install()?;

    let cfg_bytes =
        fs::read("config.toml").wrap_err_with(|| eyre!("Cannot open file \"config.toml\""))?;
    let cfg: Config =
        toml::from_slice(&cfg_bytes).wrap_err_with(|| eyre!("Cannot parse config"))?;

    let mut proxies = vec![];

    match cfg.proxy {
        None => bail!("No proxies defined in config"),
        Some(proxy_list) => {
            for p in &proxy_list {
                let listen = str_to_sock_addr(&p.listen).await?;
                let connect = str_to_sock_addr(&p.connect).await?;

                let proxy = TcpProxy::new(listen, connect);
                proxies.push(proxy);
            }

            let futures = proxies.iter().map(|p| p.listen());
            join_all(futures).await;
        }
    }

    Ok(())
}
