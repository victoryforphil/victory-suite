use std::sync::{Arc, Mutex};

use config::BrokerTaskConfig;
use subscription::BrokerTaskSubscription;
use victory_data_store::database::view::DataView;

pub mod example;
pub mod state;
pub mod subscription;
pub mod trigger;

pub mod config;

pub type BrokerTaskID = u32;

pub type BrokerTaskHandle = Arc<Mutex<dyn BrokerTask>>;

pub trait BrokerTask: Send {
    fn get_config(&self) -> BrokerTaskConfig;
    fn on_execute(&mut self, inputs: &DataView) -> Result<DataView, anyhow::Error>;
}
