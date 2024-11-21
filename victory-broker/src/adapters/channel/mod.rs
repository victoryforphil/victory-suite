use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::adapters::{BrokerAdapter, BrokerAdapterError};
use crate::task::config::BrokerTaskConfig;
use victory_data_store::database::view::DataView;

#[derive(thiserror::Error, Debug)]
pub enum ChannelBrokerError {
    #[error("Failed to send message through channel")]
    ChannelSendError,
}

pub struct ChannelBrokerAdapter {
    // Sender for sending messages to the connected adapter
    send_tx: Sender<ChannelMessage>,
    // Receiver for receiving messages from the connected adapter
    recv_rx: Receiver<ChannelMessage>,
    // Internal queues for managing tasks and responses
    new_tasks: Vec<BrokerTaskConfig>,
    execute_queue: Vec<(BrokerTaskConfig, DataView)>,
    response_queue: Vec<(BrokerTaskConfig, DataView)>,
}

// Messages exchanged between adapters
enum ChannelMessage {
    NewTask(BrokerTaskConfig),
    ExecuteTask(BrokerTaskConfig, DataView),
    TaskResponse(BrokerTaskConfig, DataView),
}

impl ChannelBrokerAdapter {
    /// Creates a new `ChannelBrokerAdapter` with connected channels.
    pub fn new_pair() -> (Arc<Mutex<Self>>, Arc<Mutex<Self>>) {
        let (a_send_tx, a_recv_rx) = channel::<ChannelMessage>();
        let (b_send_tx, b_recv_rx) = channel::<ChannelMessage>();

        let adapter_a = ChannelBrokerAdapter {
            send_tx: a_send_tx,
            recv_rx: b_recv_rx,
            new_tasks: Vec::new(),
            execute_queue: Vec::new(),
            response_queue: Vec::new(),
        };

        let adapter_b = ChannelBrokerAdapter {
            send_tx: b_send_tx,
            recv_rx: a_recv_rx,
            new_tasks: Vec::new(),
            execute_queue: Vec::new(),
            response_queue: Vec::new(),
        };

        (
            Arc::new(Mutex::new(adapter_a)),
            Arc::new(Mutex::new(adapter_b)),
        )
    }

    /// Internal method to process incoming messages.
    fn process_incoming_messages(&mut self) {
        while let Ok(message) = self.recv_rx.try_recv() {
            match message {
                ChannelMessage::NewTask(task_config) => {
                    self.new_tasks.push(task_config);
                }
                ChannelMessage::ExecuteTask(task_config, inputs) => {
                    self.execute_queue.push((task_config, inputs));
                }
                ChannelMessage::TaskResponse(task_config, outputs) => {
                    self.response_queue.push((task_config, outputs));
                }
            }
        }
    }
}

impl BrokerAdapter for ChannelBrokerAdapter {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        self.process_incoming_messages();
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError> {
        self.send_tx
            .send(ChannelMessage::NewTask(task.clone()))
            .map_err(|_| {
                BrokerAdapterError::Generic(Box::new(ChannelBrokerError::ChannelSendError))
            })
    }

    fn send_execute(
        &mut self,
        task: &BrokerTaskConfig,
        inputs: &DataView,
    ) -> Result<(), BrokerAdapterError> {
        self.send_tx
            .send(ChannelMessage::ExecuteTask(task.clone(), inputs.clone()))
            .map_err(|_| {
                BrokerAdapterError::Generic(Box::new(ChannelBrokerError::ChannelSendError))
            })
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
        self.send_tx
            .send(ChannelMessage::TaskResponse(task.clone(), outputs.clone()))
            .map_err(|_| {
                BrokerAdapterError::Generic(Box::new(ChannelBrokerError::ChannelSendError))
            })
    }
}
