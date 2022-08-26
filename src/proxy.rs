use std::net::SocketAddr;
use std::sync::Arc;

use color_eyre::eyre::Result;
use derive_getters::Getters;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Copy, Clone, Getters)]
pub struct TcpProxy {
    listen_address: SocketAddr,
    connect_address: SocketAddr,
}

impl TcpProxy {
    pub fn new(listen_address: SocketAddr, connect_address: SocketAddr) -> Self {
        Self {
            listen_address,
            connect_address,
        }
    }

    pub async fn listen(&self) -> Result<()> {
        let listen = Arc::new(self.listen_address);
        let connect = Arc::new(self.connect_address);
        log::info!(
            "Starting TcpProxy {} -> {}",
            self.listen_address,
            self.connect_address
        );

        let listener = TcpListener::bind(self.listen_address).await?;

        loop {
            let (source, downstream_addr) = match listener.accept().await {
                Ok(x) => x,
                Err(e) => {
                    log::error!(
                        "{} -> {}: Could not accept connection: {}",
                        listen,
                        connect,
                        e
                    );
                    continue;
                }
            };

            let downstream_addr = Arc::new(downstream_addr);
            log::debug!("Downstream {} connected to {}", downstream_addr, listen);

            let target = match TcpStream::connect(self.connect_address).await {
                Ok(target) => target,
                Err(e) => {
                    log::error!(
                        "{} -> {}: Could not connect to upstream: {}",
                        listen,
                        connect,
                        e
                    );
                    continue;
                }
            };

            log::debug!("Proxy {} connected to upstream {}", listen, connect);

            let (source_read, source_write) = source.into_split();
            let (target_read, target_write) = target.into_split();

            let forward_task = tokio::spawn(handle_task(
                source_read,
                target_write,
                downstream_addr.clone(),
                listen.clone(),
                connect.clone(),
            ));
            let backward_task = tokio::spawn(handle_task(
                target_read,
                source_write,
                downstream_addr.clone(),
                listen.clone(),
                connect.clone(),
            ));

            tokio::select! {
                _ = forward_task => log::debug!("Downstream {} closed the connection to {}", downstream_addr, listen),
                _ = backward_task => log::debug!("Upstream {} closed the connection", connect),
            }

            log::debug!(
                "{} -> {} -> {} connections closed",
                downstream_addr,
                listen,
                connect
            );
        }
    }
}

async fn handle_task<T: AsyncRead + Unpin, U: AsyncWrite + Unpin>(
    source: T,
    mut target: U,
    downstream_addr: Arc<SocketAddr>,
    source_addr: Arc<SocketAddr>,
    target_addr: Arc<SocketAddr>,
) {
    let mut br = BufReader::new(source);
    loop {
        // read from source
        let rx = match br.fill_buf().await {
            Ok(rx) => rx.to_owned(),
            Err(e) => {
                log::error!("Failed to read from socket: {}", e);
                break;
            }
        };

        let n_target = rx.len();

        log::trace!(
            "{} bytes read from {} at {}",
            n_target,
            downstream_addr,
            source_addr
        );

        if n_target == 0 {
            log::trace!("Closing {} handler because of 0 bytes read", source_addr);
            break;
        }

        // Write to target
        if let Err(e) = target.write_all(&rx).await {
            log::error!("Failed to write to socket: {}", e);
            break;
        }

        log::trace!("{} bytes written to {}", n_target, target_addr);

        br.consume(n_target);
    }
}
