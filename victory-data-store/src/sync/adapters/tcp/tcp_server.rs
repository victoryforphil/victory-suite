use std::{collections::HashMap, sync::Arc};

use log::{debug, info, warn};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{oneshot, Mutex, RwLock},
};

use crate::sync::{
    adapters::{AdapterError, SyncAdapter},
    packet::SyncMessage,
    SyncConnectionIDType,
};

use super::connection::{TcpSyncConnection, TcpSyncConnectionHandle};

/// TCP Server for Sync Adapter
/// Will consist of two parts:
/// 1. A TCP Server
///  - Takes the form of Tokio tasks and TCP Listener.
///  - Will accept new connections and assign them IDs
///  - Will read and "sign" incoming TCP packets, storing them in a FIFO queue per connection
///  - Will send "signed" packets to other connected clients
/// 2. A set of handlers for the various sync messages (SyncHandler)
///  - Will read packets from the FIFO queue, deserialize them, and pass them to the appropriate handler
///  - Will serialize and sign responses from handlers, sending them to the appropriate client

#[derive(Debug, Default)]
pub struct TcpSyncServer {
    address: String,
    connections: Arc<Mutex<Vec<TcpSyncConnectionHandle>>>,
}

impl TcpSyncServer {
    pub async fn new(address: &str) -> Self {
        info!("[Sync/TcpServer] Starting TCP Server on {}", address);
        let mut server = TcpSyncServer {
            address: address.to_string(),
            connections: Arc::new(Mutex::new(Vec::new())),
        };
        server.start_tcp_server_async().await;
        server
    }

    async fn start_tcp_server_async(&mut self) {
        // Start a looping thread to accept new connections
        let address = self.address.clone();
        let connections = self.connections.clone();
        tokio::spawn(async move {
            let listener = TcpListener::bind(address.clone()).await.unwrap();
            info!("[Sync/TcpServer] Listening on {}", address.clone());
            while let Ok((stream, _)) = listener.accept().await {
                debug!(
                    "[Sync/TcpServer] New connection from: {:?}",
                    stream.peer_addr().unwrap()
                );
                let connection = TcpSyncConnection::new(stream).await;
                connections.lock().await.push(connection);
            }
        });
    }
}

impl SyncAdapter for TcpSyncServer {
    fn get_name(&self) -> String {
        "TcpSyncServer".to_string()
    }

    fn get_anon_connections(&self) -> Vec<SyncConnectionIDType> {
        // Get all connections that have welcome = false, and then send them a welcome message
        let connections = self.connections.try_lock().unwrap();
        connections
            .iter()
            .filter(|c| !c.try_lock().unwrap().welcomed)
            .map(|c| c.try_lock().unwrap().connection_id)
            .collect()
    }

    fn read(
        &mut self,
    ) -> Result<Vec<crate::sync::packet::SyncMessage>, crate::sync::adapters::AdapterError> {
        let mut messages = Vec::new();
        let connections = match self.connections.try_lock() {
            Ok(connections) => connections,
            Err(e) => {
                warn!("[Sync/TcpServer] Failed to lock connections: {:?}", e);
                return Ok(Vec::new());
            }
        };
        for connection in connections.iter() {
            while let Ok(message) = connection.try_lock().unwrap().recv_rx.try_recv() {
                messages.push(message);
            }
        }
        Ok(messages)
    }

    fn write(
        &mut self,
        to_send: Vec<crate::sync::packet::SyncMessage>,
    ) -> Result<(), crate::sync::adapters::AdapterError> {
        for message in to_send {
            debug!(
                "[Sync/TcpServer] Writing message to connections: {:?}",
                message
            );
            let connections = self.connections.try_lock().unwrap();
            if message.connection_id.is_none() {
                // Send to all connections if no specific connection_id
                for connection in connections.iter() {
                    let mut conn = connection.try_lock().unwrap();
                    conn.welcomed = true;
                    conn.send_tx.try_send(message.clone());
                }
            } else {
                // Send to specific connection
                let connection_id = message.connection_id.unwrap();
                let connection = connections
                    .iter()
                    .find(|c| c.try_lock().unwrap().connection_id == connection_id)
                    .unwrap();
                let mut conn = connection.try_lock().unwrap();
                conn.welcomed = true;
                conn.send_tx.try_send(message).unwrap();
            }
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
