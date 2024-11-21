use log::{info, warn};
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};

use super::{
    connection::{TcpBrokerConnection, TcpBrokerConnectionHandle},
    message::TcpBrokerMessage,
};
use crate::adapters::{BrokerAdapter, BrokerAdapterError};
use crate::task::config::BrokerTaskConfig;
use victory_data_store::database::view::DataView;

pub struct TcpBrokerClient {
    address: String,
    connection: TcpBrokerConnectionHandle,
    // Internal queues for managing tasks and responses
    new_tasks: Vec<BrokerTaskConfig>,
    execute_queue: Vec<(BrokerTaskConfig, DataView)>,
    response_queue: Vec<(BrokerTaskConfig, DataView)>,
}

impl TcpBrokerClient {
    pub async fn new(address: &str) -> Result<Self, BrokerAdapterError> {
        info!("[Broker/TcpClient] Connecting to {}", address);
        let stream = TcpStream::connect(address)
            .await
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        let connection = TcpBrokerConnection::new(stream).await;

        Ok(Self {
            address: address.to_string(),
            connection,
            new_tasks: Vec::new(),
            execute_queue: Vec::new(),
            response_queue: Vec::new(),
        })
    }

    fn process_incoming_messages(&mut self) {
        let conn = self.connection.clone();
        let mut conn = conn.try_lock().unwrap();
        let mut connection_id = conn.connection_id;
        while let Ok(message) = conn.recv_rx.try_recv() {
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

impl BrokerAdapter for TcpBrokerClient {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::NewTask(task.clone());
        let conn = self.connection.clone();
        let mut conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        inputs: &DataView,
    ) -> Result<(), BrokerAdapterError> {
        let message = TcpBrokerMessage::ExecuteTask(task.clone(), inputs.clone());
        let conn = self.connection.clone();
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
        let conn = self.connection.clone();
        let mut conn = conn.try_lock().unwrap();
        conn.send_tx
            .try_send(message)
            .map_err(|e| BrokerAdapterError::Generic(Box::new(e)))?;
        Ok(())
    }
}
