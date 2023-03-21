use derive_getters::Getters;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};
use tracing::{info_span, Instrument};

#[derive(Debug, Clone, Getters)]
pub struct Tcp {
    listen_address: SocketAddr,
    connect_address: String,
    span: tracing::span::Span,
}

impl Tcp {
    pub fn new(listen_address: SocketAddr, connect_address: &str) -> Self {
        Self {
            listen_address,
            connect_address: connect_address.to_string(),
            span: info_span!(
                "TCP",
                listen = listen_address.to_string(),
                connect = connect_address
            ),
        }
    }

    pub async fn run(&self) {
        self.run_internal().instrument(self.span.clone()).await;
    }

    async fn run_internal(&self) {
        tracing::info!("Starting");

        let listener = match TcpListener::bind(self.listen_address).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("Failed to start TcpListener: {}", e);

                return;
            }
        };

        let listen = Arc::new(*self.listen_address());
        let connect = Arc::new(self.connect_address().clone());

        loop {
            let (source, downstream_addr) = match listener.accept().await {
                Ok(x) => x,
                Err(e) => {
                    tracing::error!(
                        "{} -> {}: Could not accept connection: {}",
                        self.listen_address(),
                        self.connect_address(),
                        e
                    );
                    continue;
                }
            };

            tracing::debug!(
                "Downstream {} connected to {}",
                downstream_addr,
                self.listen_address()
            );

            let listen = listen.clone();
            let connect = connect.clone();

            tokio::spawn(async move {
                let target = match TcpStream::connect(connect.as_str()).await {
                    Ok(target) => target,
                    Err(e) => {
                        tracing::error!(
                            "Could not connect to upstream: {}",
                            e
                        );
                        return;
                    }
                };

                tracing::debug!("Proxy {} connected to upstream {}", listen, connect);

                let (source_read, source_write) = source.into_split();
                let (target_read, target_write) = target.into_split();

                let forward_task = tokio::spawn(handle_task(
                    source_read,
                    target_write,
                ).instrument(info_span!("fwd", downstream_address = downstream_addr.to_string())));
                let backward_task = tokio::spawn(handle_task(
                    target_read,
                    source_write,
                ).instrument(info_span!("bwd", downstream_address = downstream_addr.to_string())));

                tokio::select! {
                    _ = forward_task => tracing::debug!("Downstream {} closed the connection to {}", downstream_addr, listen),
                    _ = backward_task => tracing::debug!("Upstream {} closed the connection", connect),
                }

                tracing::debug!(
                    "{} -> {} -> {} connections closed",
                    downstream_addr,
                    listen,
                    connect,
                );
            }.instrument(info_span!(parent: &self.span, "handler")));
        }
    }
}

async fn handle_task<T: AsyncRead + Unpin, U: AsyncWrite + Unpin>(source: T, mut target: U) {
    let mut br = BufReader::new(source);
    loop {
        // read from source
        let rx = match br.fill_buf().await {
            Ok(rx) => rx.to_owned(),
            Err(e) => {
                tracing::error!("Failed to read from socket: {}", e);
                break;
            }
        };

        let n_target = rx.len();

        tracing::trace!("{} bytes read", n_target,);

        if n_target == 0 {
            tracing::trace!("Closing handler because of 0 bytes read");
            break;
        }

        // Write to target
        if let Err(e) = target.write_all(&rx).await {
            tracing::error!("Failed to write to socket: {}", e);
            break;
        }

        tracing::trace!("{} bytes written", n_target);

        br.consume(n_target);
    }
}
