use std::collections::HashMap;

use info::BrokerNodeInfo;
use serde::{Serialize, Deserialize};
use victory_data_store::database::view::DataView;

use crate::{adapters::BrokerAdapterHandle, task::{config::BrokerTaskConfig, BrokerTaskHandle, BrokerTaskID}};

pub mod info;

pub type NodeID = u32;


pub struct BrokerNode{
    pub info: BrokerNodeInfo,
    pub adapter: BrokerAdapterHandle,
    pub view: DataView,
    pub task_handles: HashMap<BrokerTaskID, BrokerTaskHandle>,
    pub task_configs: HashMap<BrokerTaskID, BrokerTaskConfig>,
}

impl BrokerNode{
    pub fn new(info: BrokerNodeInfo, adapter: BrokerAdapterHandle) -> Self{
        Self{info, adapter, view: DataView::new(), task_handles: HashMap::new(), task_configs: HashMap::new()}
    }
}