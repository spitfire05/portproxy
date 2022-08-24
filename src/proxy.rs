use std::net::SocketAddr;

use color_eyre::eyre::Result;
use derive_getters::Getters;
use log::{error, info, trace};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
            let (mut source, _) = listener.accept().await?;
            let mut target = TcpStream::connect(self.connect_address).await?;

            tokio::spawn(async move {
                const BUF_SIZE: usize = 1024;
                let mut buf = [0; BUF_SIZE];

                loop {
                    // read from source
                    let n_source = match source.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            error!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    trace!("{} bytes read from {}", n_source, listen);

                    // Write to target
                    if let Err(e) = target.write_all(&buf[0..n_source]).await {
                        error!("failed to write to socket; err = {:?}", e);
                        return;
                    }

                    trace!("{} bytes written to {}", n_source, connect);

                    if n_source != BUF_SIZE {
                        break;
                    }
                }

                loop {
                    // read target's reply
                    let n_target = match target.read(&mut buf).await {
                        // socket closed
                        Ok(n) if n == 0 => return,
                        Ok(n) => n,
                        Err(e) => {
                            error!("failed to read from socket; err = {:?}", e);
                            return;
                        }
                    };

                    trace!("{} bytes read from {}", n_target, connect);

                    // Write to source
                    if let Err(e) = source.write_all(&buf[0..n_target]).await {
                        error!("failed to write to socket; err = {:?}", e);
                        return;
                    }

                    trace!("{} bytes written to {}", n_target, listen);

                    if n_target != BUF_SIZE {
                        break;
                    }
                }
            });
        }
    }
}
