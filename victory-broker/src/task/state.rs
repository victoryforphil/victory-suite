use serde::{Deserialize, Serialize};
use victory_wtf::Timepoint;

use super::BrokerTaskID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrokerTaskStatus {
    Idle,
    Queued,
    Executing,
    Waiting,
    Completed,
    Failed,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokerTaskState {
    pub task_id: BrokerTaskID,
    pub status: BrokerTaskStatus,
    pub last_execution_time: Option<Timepoint>,
}

impl BrokerTaskState {
    pub fn new(task_id: BrokerTaskID) -> Self {
        Self {
            task_id,
            status: BrokerTaskStatus::Idle,
            last_execution_time: None,
        }
    }

    pub fn set_status(&mut self, status: BrokerTaskStatus) {
        self.status = status;
    }

    pub fn set_last_execution_time(&mut self, time: Timepoint) {
        self.last_execution_time = Some(time);
    }
}
