use std::sync::Arc;
use log::{info, warn};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, Mutex},
};
use serde::{Deserialize, Serialize};

use super::message::TcpBrokerMessage;

#[derive(Debug)]
pub struct TcpBrokerConnection {
    pub connection_id: u32,
    pub send_tx: mpsc::Sender<TcpBrokerMessage>,
    pub recv_rx: mpsc::Receiver<TcpBrokerMessage>,
}

pub type TcpBrokerConnectionHandle = Arc<Mutex<TcpBrokerConnection>>;

impl TcpBrokerConnection {
    pub async fn new(stream: TcpStream) -> TcpBrokerConnectionHandle {
        let id = rand::random();
        info!("[Broker/TcpConnection] New TcpBrokerConnection: {:?}", id);
        let (send_tx, mut send_rx) = mpsc::channel(512);
        let (recv_tx, recv_rx) = mpsc::channel(512);

        let connection = Arc::new(Mutex::new(Self {
            connection_id: id,
            send_tx: send_tx.clone(),
            recv_rx,
        }));

        let (mut read_half, mut write_half) = stream.into_split();

        // Spawn read task
        {
            let recv_tx_clone = recv_tx.clone();
            tokio::spawn(async move {
                let mut buffer = Vec::new();
                let mut temp_buffer = [0u8; 1024];
                loop {
                    match read_half.read(&mut temp_buffer).await {
                        Ok(0) => {
                            // Connection closed
                            warn!("[Broker/TcpConnection] Connection closed");
                            break;
                        }
                        Ok(n) => {
                            // Append new data to buffer
                            buffer.extend_from_slice(&temp_buffer[..n]);

                            // Try to deserialize complete messages
                            while !buffer.is_empty() {
                                match bincode::deserialize::<TcpBrokerMessage>(&buffer) {
                                    Ok(message) => {
                                        // Get size of deserialized message
                                        let consumed = bincode::serialized_size(&message)
                                            .unwrap_or(0) as usize;

                                        // Remove consumed bytes
                                        buffer.drain(..consumed);

                                        // Send the message
                                        if let Err(e) = recv_tx_clone.send(message).await {
                                            warn!(
                                                "[Broker/TcpConnection] Failed to send message to receiver: {}",
                                                e
                                            );
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
                            if buffer.len() > 1024 * 50 {
                                warn!("[Broker/TcpConnection] Buffer too large, clearing");
                                buffer.clear();
                            }
                        }
                        Err(e) => {
                            warn!("[Broker/TcpConnection] Failed to read from stream: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Spawn write task
        tokio::spawn(async move {
            while let Some(message) = send_rx.recv().await {
                let data = bincode::serialize(&message).unwrap();
                if let Err(e) = write_half.write_all(&data).await {
                    warn!("[Broker/TcpConnection] Failed to write to stream: {}", e);
                    break;
                }
            }
        });

        connection
    }
} 