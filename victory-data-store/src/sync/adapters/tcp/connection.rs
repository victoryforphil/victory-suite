use std::sync::Arc;

use log::{debug, info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, Mutex},
};

use crate::sync::{packet::SyncMessage, SyncConnectionIDType};
#[derive(Debug)]
pub struct TcpSyncConnection {
    pub connection_id: SyncConnectionIDType,
    pub send_tx: mpsc::Sender<SyncMessage>,
    pub recv_rx: mpsc::Receiver<SyncMessage>,
    pub welcomed: bool,
}

pub type TcpSyncConnectionHandle = Arc<Mutex<TcpSyncConnection>>;

impl TcpSyncConnection {
    pub async fn new(mut stream: TcpStream) -> TcpSyncConnectionHandle {
        let id = rand::random();
        info!("[Sync/TcpConnection] New TcpSyncConnection: {:?}", id);
        let (send_tx, mut send_rx) = mpsc::channel(512);
        let (recv_tx, recv_rx) = mpsc::channel(512);

        let connection = Arc::new(Mutex::new(Self {
            connection_id: id,
            send_tx: send_tx.clone(),
            recv_rx,
            welcomed: false,
        }));

        let (mut read_half, mut write_half) = stream.into_split();
        // Spawn read task
        tokio::spawn(async move {
            let mut buffer = Vec::with_capacity(1024 * 5); // Increased to 50kb
            let mut temp_buffer = [0u8; 1024];

            loop {
                match read_half.read(&mut temp_buffer).await {
                    Ok(0) => {
                        // Connection closed
                        warn!("[Sync/TcpConnection] Connection closed");
                        break;
                    }
                    Ok(n) => {
                        // Append new data to buffer
                        buffer.extend_from_slice(&temp_buffer[..n]);

                        // Try to deserialize complete messages
                        while !buffer.is_empty() {
                            match rmp_serde::from_slice::<SyncMessage>(&buffer) {
                                Ok(message) => {
                                    // Get size of deserialized message
                                    let consumed = rmp_serde::encode::to_vec_named(&message)
                                        .map(|v| v.len())
                                        .unwrap_or(0);

                                    // Remove consumed bytes
                                    buffer.drain(..consumed);

                                    // Send the message
                                    if let Err(e) = recv_tx.send(message).await {
                                        warn!("[Sync/TcpConnection] Failed to send message to receiver: {}", e);
                                        return;
                                    }
                                }
                                Err(_) => {
                                    // Message incomplete or invalid, wait for more data
                                    break;
                                }
                            }
                        }

                        // Prevent buffer from growing too large
                        if buffer.len() > 1024 * 5 {
                            // Increased to 50kb
                            warn!("[Sync/TcpConnection] Buffer too large, clearing");
                            buffer.clear();
                        }
                    }
                    Err(e) => {
                        warn!("[Sync/TcpConnection] Failed to read from stream: {}", e);
                        break;
                    }
                }
            }
        });

        tokio::spawn(async move {
            while let Some(message) = send_rx.recv().await {
                let data = rmp_serde::to_vec_named(&message).unwrap();

                if let Err(e) = write_half.write(&data).await {
                    warn!("[Sync/TcpConnection] Failed to write to stream: {}", e);
                    break;
                }
            }
        });
        connection
    }
}
