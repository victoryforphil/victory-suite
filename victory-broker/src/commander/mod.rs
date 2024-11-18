use crate::task::config::BrokerTaskConfig;

pub mod mock;

#[derive(thiserror::Error, Debug)]
pub enum BrokerCommanderError {
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),
    
    #[error("Task already exists")]
    TaskAlreadyExists,
}

pub trait BrokerCommander{
    fn add_task(&mut self, task: BrokerTaskConfig) -> Result<(), BrokerCommanderError>;
    fn get_next_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerCommanderError>;
}