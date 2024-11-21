use crate::task::config::BrokerTaskConfig;
use serde::{Deserialize, Serialize};
use victory_data_store::database::view::DataView;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TcpBrokerMessage {
    NewTask(BrokerTaskConfig),
    ExecuteTask(BrokerTaskConfig, DataView),
    TaskResponse(BrokerTaskConfig, DataView),
}
