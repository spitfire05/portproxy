mod config;
mod proxy;

use clap::{builder::TypedValueParser as _, Parser};
use futures::future::join_all;
use miette::{bail, Result};
use proxy::Tcp;
use tokio::net::lookup_host;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Path to read the config from.
    /// If not set, will fall back to value of $PORTPROXY_CONFIG,
    /// and "~/.config/portproxy.toml", in that order
    #[arg(short, long)]
    config_path: Option<String>,

    #[arg(short, long,
        default_value = "info",
        value_parser = clap::builder::PossibleValuesParser::new(["error", "warn", "info", "debug", "trace"]).map(|s| s.parse::<tracing::Level>().unwrap()))]
    log_level: tracing::Level,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .init();

    tracing::info!("portproxy v{} starting...", VERSION);

    let cfg = config::load(args.config_path)?;

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
                            tracing::debug!("Listen address {} resolved to {}", p.listen(), listen);
                            let proxy = Tcp::new(listen, p.connect());
                            proxies.push(proxy);
                        }
                    }
                    Err(e) => tracing::error!("Failed to resolve {}: {}", p.listen(), e),
                }
            }
        }
    }

    join_all(proxies.iter().map(proxy::Tcp::run)).await;

    tracing::info!("Nothing left to do, exiting..");

    Ok(())
}
