mod config;
mod proxy;

use clap::{builder::TypedValueParser as _, Parser};
use futures::future::join_all;
use miette::{bail, IntoDiagnostic, Result};
use proxy::Tcp;
use tokio::net::lookup_host;
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, prelude::__tracing_subscriber_field_MakeExt,
};

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

    /// Directory to write the log files to. Logging to file will be disabled if this is not set
    #[arg(short('d'), long)]
    log_dir: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let std_layer = tracing_subscriber::fmt::Layer::default()
        .with_writer(std::io::stderr.with_max_level(args.log_level));

    let (subscriber, _guard): (
        Box<dyn tracing::Subscriber + Send + Sync>,
        Option<tracing_appender::non_blocking::WorkerGuard>,
    ) = match args.log_dir {
        Some(ld) => {
            let file_appender = tracing_appender::rolling::daily(ld, "portproxy.log");
            let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
            (
                Box::new(
                    tracing_subscriber::Registry::default()
                        .with(std_layer)
                        .with(
                            tracing_subscriber::fmt::Layer::default()
                                .with_ansi(false)
                                // need custom fields formatter here, as the default one doe snot respect `with_ansi` :(
                                .fmt_fields(
                                    tracing_subscriber::fmt::format::debug_fn(
                                        |writer, field, value| write!(writer, "{field}: {value:?}"),
                                    )
                                    .delimited(", "),
                                )
                                .with_writer(file_writer.with_max_level(args.log_level)),
                        ),
                ),
                Some(guard),
            )
        }
        None => (
            Box::new(tracing_subscriber::Registry::default().with(std_layer)),
            None,
        ),
    };

    tracing::subscriber::set_global_default(subscriber).into_diagnostic()?;

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

    tokio::spawn(async move {
        join_all(proxies.iter().map(proxy::Tcp::run)).await;
    });

    tokio::signal::ctrl_c().await.into_diagnostic()?;

    tracing::info!("Nothing left to do, exiting..");

    Ok(())
}
