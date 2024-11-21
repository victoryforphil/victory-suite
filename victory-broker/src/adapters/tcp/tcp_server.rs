use log::{debug, info, warn};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use super::{
    connection::{TcpBrokerConnection, TcpBrokerConnectionHandle},
    message::TcpBrokerMessage,
};
use crate::adapters::{BrokerAdapter, BrokerAdapterError};
use crate::task::config::BrokerTaskConfig;
use victory_data_store::database::view::DataView;

pub struct TcpBrokerServer {
    address: String,
    connections: Arc<Mutex<Vec<TcpBrokerConnectionHandle>>>,
    // Internal queues for managing tasks and responses
    new_tasks: Vec<BrokerTaskConfig>,
    execute_queue: Vec<(BrokerTaskConfig, DataView)>,
    response_queue: Vec<(BrokerTaskConfig, DataView)>,
}

impl TcpBrokerServer {
    pub async fn new(address: &str) -> Result<Self, BrokerAdapterError> {
        info!("[Broker/TcpServer] Starting TCP Server on {}", address);

        let server = TcpBrokerServer {
            address: address.to_string(),
            connections: Arc::new(Mutex::new(Vec::new())),
            new_tasks: Vec::new(),
            execute_queue: Vec::new(),
            response_queue: Vec::new(),
        };

        server.start_listener().await;

        Ok(server)
    }

    async fn start_listener(&self) {
        let address = self.address.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let listener = TcpListener::bind(address.clone()).await.unwrap();
            info!("[Broker/TcpServer] Listening on {}", address);
            while let Ok((stream, _)) = listener.accept().await {
                debug!(
                    "[Broker/TcpServer] New connection from: {:?}",
                    stream.peer_addr().unwrap()
                );
                let connection = TcpBrokerConnection::new(stream).await;
                connections.lock().await.push(connection);
            }
        });
    }

    fn process_incoming_messages(&mut self) {
        let connections = self.connections.clone();
        // Use a timeout to avoid blocking indefinitely
        let mut connections = match connections.try_lock() {
            Ok(v) => v,
            Err(e) => {
                warn!("[Broker/TcpServer] Timeout acquiring lock: {:?}", e);
                return;
            }
        };

        for connection in connections.iter_mut() {
            let connection_id = connection.try_lock().unwrap().connection_id;
            while let Ok(message) = connection.try_lock().unwrap().recv_rx.try_recv() {
                match message {
                    TcpBrokerMessage::NewTask(mut task_config) => {
                        task_config.connection_id = connection_id;
                        self.new_tasks.push(task_config);
                    }
                    TcpBrokerMessage::ExecuteTask(mut task_config, inputs) => {
                        task_config.connection_id = connection_id;
                        self.execute_queue.push((task_config, inputs));
                    }
                    TcpBrokerMessage::TaskResponse(mut task_config, outputs) => {
                        task_config.connection_id = connection_id;
                        self.response_queue.push((task_config, outputs));
                    }
                }
            }
        }
    }
}

impl BrokerAdapter for TcpBrokerServer {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::NewTask(task.clone());
        let connections = self.connections.clone();

        let connections = match connections.try_lock() {
            Ok(v) => v,
            Err(e) => {
                warn!("[Broker/TcpServer] Timeout acquiring lock: {:?}", e);
                return Err(BrokerAdapterError::Generic(Box::new(e)));
            }
        };

        // Send to all remaining connections
        for connection in connections.iter() {
            let conn = connection.clone();
            let mut conn = conn.try_lock().unwrap();
            if let Err(e) = conn.send_tx.try_send(message.clone()) {
                warn!("[Broker/TcpServer] Failed to send new task: {:?}", e);
                continue;
            }
        }
        Ok(())
    }

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        inputs: &DataView,
    ) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::ExecuteTask(task.clone(), inputs.clone());
        let connections = self.connections.clone();
        let connections = futures::executor::block_on(connections.lock());

        // Find connection with matching ID
        let connection = connections
            .iter()
            .find(|conn| {
                let conn = conn.clone();
                let conn = conn.try_lock().unwrap();
                conn.connection_id == task.connection_id
            })
            .ok_or(BrokerAdapterError::Generic(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Connection not found",
            ))))?;

        let conn = connection.clone();
        let mut conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn recv_response(&mut self, _task: &BrokerTaskConfig) -> Result<DataView, BrokerAdapterError> {
        self.process_incoming_messages();

        if let Some((_task_config, outputs)) = self.response_queue.pop() {
            Ok(outputs)
        } else {
            Err(BrokerAdapterError::WaitingForTaskResponse)
        }
    }

    fn recv_execute(&mut self) -> Result<Vec<(BrokerTaskConfig, DataView)>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.execute_queue.drain(..).collect())
    }

    fn send_response(
        &mut self,
        task: &BrokerTaskConfig,
        outputs: &DataView,
    ) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::TaskResponse(task.clone(), outputs.clone());
        let connections = self.connections.clone();
        let connections = futures::executor::block_on(connections.lock());

        // Find connection with matching ID
        let connection = connections
            .iter()
            .find(|conn| {
                let conn = conn.clone();
                let conn = futures::executor::block_on(conn.lock());
                conn.connection_id == task.connection_id
            })
            .ok_or(BrokerAdapterError::Generic(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Connection not found",
            ))))?;

        let conn = connection.clone();
        let mut conn = futures::executor::block_on(conn.lock());
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }
}
