use victory_data_store::database::view::DataView;

use crate::task::{config::BrokerTaskConfig, state::BrokerTaskStatus};

use super::{BrokerAdapter, BrokerAdapterError};


#[derive(Default)]

pub struct MockBrokerAdapter{
    pub new_tasks: Vec<BrokerTaskConfig>,
    pub executed_tasks: Vec<(BrokerTaskConfig, DataView)>,

}

impl MockBrokerAdapter{
    pub fn new() -> Self{
        Self{new_tasks: vec![], executed_tasks: vec![]}
    }
}

impl BrokerAdapter for MockBrokerAdapter{
    fn get_new_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerAdapterError> {
        Ok(self.new_tasks.drain(..).collect())
    }

    fn send_execute(&mut self, task: &BrokerTaskConfig, inputs: &DataView) -> Result<(), BrokerAdapterError> {
        self.executed_tasks.push((task.clone(), inputs.clone()));
        Ok(())
    }

    fn recv_response(&mut self, task: &BrokerTaskConfig) -> Result<DataView, BrokerAdapterError> {
        Ok(DataView::new())
    }
}


#[cfg(test)]
mod broker_adapter_tests{
    use super::*;

    /// Test the get_new_tasks method
    /// 1. Add a new task to the adapter
    /// 2. Call get_new_tasks
    /// 3. Check that the task was returned
    #[test]
    fn test_mock_adapter_get_new_tasks(){
        let mut adapter = MockBrokerAdapter::new();
        let tasks = vec![BrokerTaskConfig::new(0, "test_task")];
        adapter.new_tasks = tasks.clone();
        let new_tasks = adapter.get_new_tasks().unwrap();
        assert_eq!(new_tasks.len(), 1);
        assert_eq!(new_tasks[0].task_id, tasks[0].task_id);
    }

    /// Test the execute_task method
    /// 1. Call the execute_task method with a task and inputs
    /// 2. Check that the task and inputs were added to the executed_tasks vector
    #[test]
    fn test_mock_adapter_execute_task(){
        let mut adapter = MockBrokerAdapter::new();
        let task = BrokerTaskConfig::new(0, "test_task");
        let inputs = DataView::new();
        adapter.send_execute(&task, &inputs).unwrap();
        assert_eq!(adapter.executed_tasks.len(), 1);
        assert_eq!(adapter.executed_tasks[0].0.task_id, task.task_id);
    }

    /// Test the check_task_response method
    /// 1. Call the check_task_response method with a task
    /// 2. Check for an Ok result
    #[test]
    fn test_mock_adapter_check_task_response(){
        let mut adapter = MockBrokerAdapter::new();
        let task = BrokerTaskConfig::new(0, "test_task");
        let response = adapter.recv_response(&task);
        assert!(response.is_ok());
    }
}