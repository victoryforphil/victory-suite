use serde::{Deserialize, Serialize};
use victory_data_store::topics::TopicKeyHandle;

use crate::adapters::{AdapterID, ConnectionID};

use super::{state::BrokerTaskStatus, subscription::BrokerTaskSubscription, trigger::BrokerTaskTrigger, BrokerTaskID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTaskConfig{
    pub task_id: BrokerTaskID,
    pub name: String,
    pub adapter_id: AdapterID,
    pub connection_id: ConnectionID,
    pub subscriptions: Vec<BrokerTaskSubscription>,
    pub trigger: BrokerTaskTrigger,
}

impl BrokerTaskConfig{
    pub fn new(task_id: BrokerTaskID, name: &str) -> Self{
        Self{task_id, name: name.to_string(), adapter_id: 0, connection_id: 0, subscriptions: vec![], trigger: BrokerTaskTrigger::Always}
    }
}