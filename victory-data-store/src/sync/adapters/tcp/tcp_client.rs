use std::collections::HashMap;

use log::{info, warn};
use tokio::net::TcpStream;

use crate::sync::{
    adapters::{AdapterError, SyncAdapter},
    packet::SyncMessage,
    SyncConnectionIDType,
};

use super::connection::{TcpSyncConnection, TcpSyncConnectionHandle};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TcpClientError {
    #[error("{0}")]
    Generic(String),
    #[error("Failed to connect to TCP server")]
    ConnectFailed(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, TcpClientError>;

#[derive(Debug, Clone)]
pub struct TCPClient {
    address: String,
    connection: TcpSyncConnectionHandle,
}

impl TCPClient {
    pub async fn new(address: String) -> Result<Self> {
        info!("[Sync/TcpClient] Connecting to {}", address);
        let mut stream = TcpStream::connect(address.clone())
            .await
            .map_err(|e| TcpClientError::ConnectFailed(e))?;
        let connection = TcpSyncConnection::new(stream).await;
        Ok(Self {
            address: address.clone(),
            connection,
        })
    }
}

impl SyncAdapter for TCPClient {
    fn get_name(&self) -> String {
        "TCPClient".to_string()
    }

    fn get_anon_connections(&self) -> Vec<SyncConnectionIDType> {
        // Get all connections that have welcome = false, and then send them a welcome message
        let connection = self.connection.try_lock().unwrap();
        if !connection.welcomed {
            return vec![connection.connection_id];
        }
        vec![]
    }

    fn read(&mut self) -> std::result::Result<Vec<SyncMessage>, AdapterError> {
        let mut messages = Vec::new();
        let mut connection = match self.connection.try_lock() {
            Ok(c) => c,
            Err(e) => {
                return Ok(Vec::new());
            }
        };
        let message = connection.recv_rx.try_recv();
        match message {
            Ok(message) => messages.push(message),
            Err(e) => {}
        }
        Ok(messages)
    }

    fn write(&mut self, to_send: Vec<SyncMessage>) -> std::result::Result<(), AdapterError> {
        for message in to_send {
            let mut connection = self.connection.try_lock().unwrap();
            connection.welcomed = true;
            connection.send_tx.try_send(message).unwrap();
        }
        Ok(())
    }

    fn get_description(&self) -> String {
        std::format!("Adapter: {}", self.get_name())
    }

    fn get_stats(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn get_live(&self) -> bool {
        true
    }
}
