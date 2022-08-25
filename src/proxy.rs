use std::net::SocketAddr;
use std::sync::Arc;

use color_eyre::eyre::Result;
use derive_getters::Getters;
use log::{error, info, trace};
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
        info!(
            "Starting TcpProxy {} -> {}",
            self.listen_address, self.connect_address
        );

        let listener = TcpListener::bind(self.listen_address).await?;

        loop {
            let (source, _) = listener.accept().await?;
            let target = TcpStream::connect(self.connect_address).await?;
            let (source_read, source_write) = source.into_split();
            let (target_read, target_write) = target.into_split();

            let forward_task = tokio::spawn(handle_task(
                source_read,
                target_write,
                listen.clone(),
                connect.clone(),
            ));
            let backward_task = tokio::spawn(handle_task(
                target_read,
                source_write,
                listen.clone(),
                connect.clone(),
            ));

            tokio::select! {
                _ = forward_task => trace!("{} closed the connection", listen),
                _ = backward_task => trace!("{} closed the connection", connect),
            }
        }
    }
}

async fn handle_task<T: AsyncRead + Unpin, U: AsyncWrite + Unpin>(
    source: T,
    mut target: U,
    source_addr: Arc<SocketAddr>,
    target_addr: Arc<SocketAddr>,
) {
    let mut br = BufReader::new(source);
    loop {
        // read from source
        let rx = match br.fill_buf().await {
            Ok(rx) => rx.to_owned(),
            Err(e) => {
                error!("failed to read from socket; err = {:?}", e);
                break;
            }
        };

        let n_target = rx.len();

        trace!("{} bytes read from {}", n_target, source_addr);

        if n_target == 0 {
            trace!("closing {} handler because of 0 bytes read", source_addr);
            break;
        }

        // Write to target
        if let Err(e) = target.write_all(&rx).await {
            error!("failed to write to socket; err = {:?}", e);
            break;
        }

        trace!("{} bytes written to {}", n_target, target_addr);

        br.consume(n_target);
    }
}
