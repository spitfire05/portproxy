use crate::plugin::Plugin;
use derive_getters::Getters;
use std::{fmt::Display, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

#[derive(Debug, Clone, Getters)]
pub struct TcpProxy {
    listen_address: SocketAddr,
    connect_address: String,
    plugins: Arc<Vec<Plugin>>,
}

impl TcpProxy {
    pub fn new(listen_address: SocketAddr, connect_address: String, plugins: Vec<Plugin>) -> Self {
        Self {
            listen_address,
            connect_address,
            plugins: Arc::new(plugins),
        }
    }

    pub async fn run(&self) {
        log::info!(
            "Starting TcpProxy {} -> {}",
            self.listen_address,
            self.connect_address
        );

        let listener = match TcpListener::bind(self.listen_address).await {
            Ok(l) => l,
            Err(e) => {
                log::error!(
                    "Failed to start TcpListener at \"{}\" : {}",
                    self.listen_address(),
                    e
                );

                return;
            }
        };

        let listen = Arc::new(self.listen_address().to_owned());
        let connect = Arc::new(self.connect_address().clone());

        loop {
            let (source, downstream_addr) = match listener.accept().await {
                Ok(x) => x,
                Err(e) => {
                    log::error!(
                        "{} -> {}: Could not accept connection: {}",
                        self.listen_address(),
                        self.connect_address(),
                        e
                    );
                    continue;
                }
            };

            log::debug!(
                "Downstream {} connected to {}",
                downstream_addr,
                self.listen_address()
            );

            let listen = listen.clone();
            let connect = connect.clone();
            let plugins = self.plugins.clone();

            tokio::spawn(async move {
                let target = match TcpStream::connect(connect.as_str()).await {
                    Ok(target) => target,
                    Err(e) => {
                        log::error!(
                            "{} -> {}: Could not connect to upstream: {}",
                            listen,
                            connect,
                            e
                        );
                        return;
                    }
                };

                log::debug!("Proxy {} connected to upstream {}", listen, connect);

                let (source_read, source_write) = source.into_split();
                let (target_read, target_write) = target.into_split();

                let forward_task = tokio::spawn(handle_task(
                    source_read,
                    target_write,
                    downstream_addr,
                    listen.clone(),
                    connect.clone(),
                    plugins.clone(),
                ));
                let backward_task = tokio::spawn(handle_task(
                    target_read,
                    source_write,
                    downstream_addr,
                    listen.clone(),
                    connect.clone(),
                    plugins.clone(),
                ));

                tokio::select! {
                    _ = forward_task => log::debug!("Downstream {} closed the connection to {}", downstream_addr, listen),
                    _ = backward_task => log::debug!("Upstream {} closed the connection", connect),
                }

                log::debug!(
                    "{} -> {} -> {} connections closed",
                    downstream_addr,
                    listen,
                    connect,
                );
            });
        }
    }
}

async fn handle_task<
    T: AsyncRead + Unpin,
    U: AsyncWrite + Unpin,
    D: Display,
    S: Display,
    X: Display,
>(
    source: T,
    mut target: U,
    downstream_addr: D,
    source_addr: S,
    target_addr: X,
    plugins: Arc<Vec<Plugin>>,
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
        let rx = Arc::new(rx);

        let n_target = rx.len();

        log::trace!(
            "{} bytes read from {} at {}",
            n_target,
            downstream_addr,
            source_addr
        );

        let move_rx = rx.clone();
        let plugins = plugins.clone();
        tokio::spawn(async move {
            for p in plugins.iter() {
                p.exec(&move_rx).unwrap();
            }
        });

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
