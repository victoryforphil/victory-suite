
pub type AdapterID = u32;
pub type ConnectionID = u32;

use std::sync::{Arc, Mutex};

use victory_data_store::database::view::DataView;

use crate::task::config::BrokerTaskConfig;

pub mod mock;

pub type BrokerAdapterHandle = Arc<Mutex<dyn BrokerAdapter>>;

#[derive(thiserror::Error, Debug)]
pub enum BrokerAdapterError {
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),

    /// Waiting for task response
    #[error("Waiting for task response")]
    WaitingForTaskResponse,
}

pub trait BrokerAdapter{
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError>;
    fn execute_task(&mut self, task: &BrokerTaskConfig, inputs: &DataView) -> Result<(), BrokerAdapterError>;
    fn check_task_response(&mut self, task: &BrokerTaskConfig) -> Result<DataView, BrokerAdapterError>;
}