use crate::{broker::time::BrokerTime, task::config::BrokerTaskConfig};
use serde::{Deserialize, Serialize};
use victory_data_store::{database::view::DataView, datapoints::Datapoint};
use victory_wtf::Timepoint;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TcpBrokerMessage {
    NewTask(BrokerTaskConfig),  
    ExecuteTask(BrokerTaskConfig, BrokerTime),
    TaskResponse(BrokerTaskConfig),
    Inputs(Vec<Datapoint>),
    Outputs(Vec<Datapoint>),
}
