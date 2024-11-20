
pub type AdapterID = u32;
pub type ConnectionID = u32;

use std::sync::{Arc, Mutex};

use victory_data_store::database::view::DataView;

use crate::task::config::BrokerTaskConfig;

pub mod mock;
pub mod tcp;
pub mod channel;
pub type BrokerAdapterHandle = Arc<Mutex<dyn BrokerAdapter>>;

#[derive(thiserror::Error, Debug)]
pub enum BrokerAdapterError {
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Waiting for task response
    #[error("Waiting for task response")]
    WaitingForTaskResponse,
}

pub trait BrokerAdapter: Send {
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError>;
    fn send_new_task(&mut self, task: &BrokerTaskConfig) -> Result<(), BrokerAdapterError>;
    fn send_execute(&mut self, task: &BrokerTaskConfig, inputs: &DataView) -> Result<(), BrokerAdapterError>;
    fn recv_response(&mut self, task: &BrokerTaskConfig) -> Result<DataView, BrokerAdapterError>;

    fn recv_execute(&mut self) -> Result<Vec<(BrokerTaskConfig, DataView)>, BrokerAdapterError>;
    fn send_response(&mut self, task: &BrokerTaskConfig, outputs: &DataView) -> Result<(), BrokerAdapterError>;
}