use std::net::SocketAddr;

use color_eyre::eyre::Result;
use derive_getters::Getters;
use log::{error, info, trace};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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
        let listen = self.listen_address;
        let connect = self.connect_address;
        info!(
            "Starting TcpProxy {} -> {}",
            self.listen_address, self.connect_address
        );

        let listener = TcpListener::bind(self.listen_address).await?;

        loop {
            let (source, _) = listener.accept().await?;
            let target = TcpStream::connect(self.connect_address).await?;
            let (mut source_read, mut source_write) = source.into_split();
            let (mut target_read, mut target_write) = target.into_split();

            let forward_task = tokio::spawn(async move {
                loop {
                    // read from source
                    let mut br = BufReader::new(&mut source_read);
                    let rx = match br.fill_buf().await {
                        Ok(rx) => rx.to_owned(),
                        Err(e) => {
                            error!("failed to read from socket; err = {:?}", e);
                            break;
                        }
                    };

                    let n_source = rx.len();

                    trace!("{} bytes read from {}", n_source, listen);

                    if n_source == 0 {
                        trace!("closing {} handler because of 0 bytes read", listen);
                        break;
                    }

                    // Write to target
                    if let Err(e) = target_write.write_all(&rx).await {
                        error!("failed to write to socket; err = {:?}", e);
                        break;
                    }

                    br.consume(n_source);

                    trace!("{} bytes written to {}", n_source, connect);
                }
            });

            let backward_task = tokio::spawn(async move {
                loop {
                    // read from target
                    let mut br = BufReader::new(&mut target_read);
                    let rx = match br.fill_buf().await {
                        Ok(rx) => rx.to_owned(),
                        Err(e) => {
                            error!("failed to read from socket; err = {:?}", e);
                            break;
                        }
                    };

                    let n_target = rx.len();

                    trace!("{} bytes read from {}", n_target, connect);

                    if n_target == 0 {
                        trace!("closing {} handler because of 0 bytes read", listen);
                        break;
                    }

                    // Write to source
                    if let Err(e) = source_write.write_all(&rx).await {
                        error!("failed to write to socket; err = {:?}", e);
                        break;
                    }

                    trace!("{} bytes written to {}", n_target, listen);

                    br.consume(n_target);
                }
            });

            tokio::select! {
                _ = forward_task => trace!("{} closed the connection", listen),
                _ = backward_task => trace!("{} closed the connection", connect),
            }
        }
    }
}
