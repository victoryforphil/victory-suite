use std::collections::HashMap;

use info::BrokerNodeInfo;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use victory_data_store::database::view::DataView;

use crate::{
    adapters::BrokerAdapterHandle,
    task::{config::BrokerTaskConfig, BrokerTaskHandle, BrokerTaskID},
};

pub mod info;

pub type NodeID = u32;

pub struct BrokerNode {
    pub info: BrokerNodeInfo,
    pub adapter: BrokerAdapterHandle,
    pub view: DataView,
    pub task_handles: HashMap<BrokerTaskID, BrokerTaskHandle>,
    pub task_configs: HashMap<BrokerTaskID, BrokerTaskConfig>,
}

impl BrokerNode {
    pub fn new(info: BrokerNodeInfo, adapter: BrokerAdapterHandle) -> Self {
        Self {
            info,
            adapter,
            view: DataView::new(),
            task_handles: HashMap::new(),
            task_configs: HashMap::new()
        }
    }

    pub fn add_task(&mut self, task_handle: BrokerTaskHandle) -> Result<(), anyhow::Error> {
        let task_config = task_handle.lock().unwrap().get_config();
        info!(
            "Node {:?} - Adding task {:?} to node {:?}",
            self.info.node_id, task_config.task_id, self.info.node_id
        );
        self.task_handles.insert(task_config.task_id, task_handle);
        self.task_configs
            .insert(task_config.task_id, task_config.clone());
        
        self.adapter.lock().unwrap().send_new_task(&task_config)?;
        Ok(())
    }

    pub fn tick(&mut self) -> Result<(), anyhow::Error> {
        // 1. Check for any execute requests from the adapter
        let mut adapter = self.adapter.lock().unwrap();
        let execute_requests = adapter.recv_execute()?;

        // 2. Execute the tasks
        // TODO: Parallelize this so tasks can execute in parallel
        for (task_config, inputs) in execute_requests.iter() {
            debug!(
                "Node {:?} // Executing task {:?} with inputs {:?}",
                self.info.node_id, task_config.task_id, inputs
            );

            let task_handle = self
                .task_handles
                .get_mut(&task_config.task_id)
                .expect("Task not found");
            let results = task_handle.lock().unwrap().on_execute(&inputs)?;
            adapter.send_response(&task_config, &results)?;
        }
    
        Ok(())
    }
}
