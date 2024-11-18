use std::collections::HashMap;

use log::{debug, warn};
use victory_data_store::database::{self, view::DataView, Datastore, DatastoreHandle};
use victory_wtf::Timepoint;

use crate::{
    adapters::{AdapterID, BrokerAdapterError, BrokerAdapterHandle},
    commander::BrokerCommander,
    task::{
        config::BrokerTaskConfig, state::{BrokerTaskState, BrokerTaskStatus}, subscription::SubscriptionMode, trigger::BrokerTaskTrigger, BrokerTaskID
    },
};

pub struct Broker<TCommander> {
    pub(crate) commander: TCommander,
    pub(crate) adapters: HashMap<AdapterID, BrokerAdapterHandle>,
    pub(crate) datastore: DatastoreHandle,
    pub(crate) task_configs: HashMap<BrokerTaskID, BrokerTaskConfig>,
    pub(crate) task_states: HashMap<BrokerTaskID, BrokerTaskState>,
    broker_time: Timepoint,
}

#[derive(thiserror::Error, Debug)]
pub enum BrokerError {
    #[error(transparent)]
    Generic(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<TCommander> Broker<TCommander>
where
    TCommander: BrokerCommander,
{
    pub fn new(commander: TCommander) -> Self {
        let database = Datastore::new();
        Self {
            commander,
            adapters: HashMap::new(),
            datastore: database.handle(),
            task_configs: HashMap::new(),
            task_states: HashMap::new(),
            broker_time: Timepoint::zero(),
        }
    }

    pub fn tick(&mut self) -> Result<(), BrokerError> {

        // 1. Read new tasks from adapters
        debug!("Broker // Reading new tasks");
        let _ = match self.read_new_tasks() {
            Ok(tasks) => tasks,
            Err(e) => return Err(BrokerError::Generic(e.into())),
        };

        // 2. Get next tasks to execute
        let next_tasks = match self.commander.get_next_tasks() {
            Ok(tasks) => tasks,
            Err(e) => return Err(BrokerError::Generic(e.into())),
        };
        debug!("Broker // Next tasks: {:?}", next_tasks.iter().map(|t| t.task_id).collect::<Vec<_>>()   );

        // 2.1 Set next_tasks to Queued state.
        for task in next_tasks {
            self.set_task_status(task.task_id, BrokerTaskStatus::Queued);
        }
        

        let queued_tasks = self.get_tasks_with_status(BrokerTaskStatus::Queued);

        // TODO: Run tasks in parallel
        for (task_id, task_config) in queued_tasks {
            // Check the trigger for each task, continue / skip if not triggered
            if !self.check_trigger(&task_config).is_ok() {
                continue;
            }

            // 3. Get inputs for the task
            let inputs = self.get_task_inputs(&task_config).unwrap();

            debug!("Broker // Executing task: {:?}", task_id);
            // Execute the task
            let adapter_id = task_config.adapter_id;
            let adapter = self.adapters.get_mut(&adapter_id).unwrap();
            let mut adapter = adapter.lock().unwrap();
            // Set the last execution time
            let task_state = self.task_states.get_mut(&task_id).unwrap();
            task_state.set_last_execution_time(self.broker_time.clone());
            task_state.set_status(BrokerTaskStatus::Executing);
            // Execute the task
            adapter.send_execute(&task_config, &inputs).unwrap();

            // 4. Check the task response
            debug!("Broker // Waiting for response for {:?}", task_id);
            let mut task_response = adapter.recv_response(&task_config);
            // Set the status to WaitingForTaskResponse
            task_state.set_status(BrokerTaskStatus::Waiting);

            while let Err(BrokerAdapterError::WaitingForTaskResponse) = task_response {
                task_response = adapter.recv_response(&task_config);
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            // Set the status to Completed
            task_state.set_status(BrokerTaskStatus::Completed);
            debug!("Broker // Response for {:?}", task_id);  
            // 5. Apply the task response
            self.datastore.lock().unwrap().apply_view(task_response.unwrap()).unwrap();
        }

        Ok(())
    }

    // Task querying functions
    pub fn get_tasks_with_status(&self, status: BrokerTaskStatus) -> HashMap<BrokerTaskID, BrokerTaskConfig> {
        self.task_states
            .iter()
            .filter(|(_, state)| state.status == status)
            .map(|(task_id, _)| (task_id.clone(), self.task_configs[task_id].clone()))
            .collect()
    }

    pub fn set_task_status(&mut self, task_id: BrokerTaskID, status: BrokerTaskStatus){
        self.task_states.get_mut(&task_id).unwrap().set_status(status);
    }
}

impl<TCommander> Broker<TCommander>
where
    TCommander: BrokerCommander,
{
    /// Read for any new registered tasks from adapters
    fn read_new_tasks(&mut self) -> Result<(), anyhow::Error> {
        for (_adapter_id, adapter_handle) in self.adapters.iter_mut() {
            let mut adapter = adapter_handle.lock().unwrap();
            let new_tasks = adapter.get_new_tasks()?;
        
            for task in new_tasks {
                debug!("Broker // New Task read from adapter {:?}", task.task_id);
                // Create a new task state and insert it into the task_states map
                self.task_states.insert(task.task_id, BrokerTaskState::new(task.task_id));
                // Insert the task config into the task_configs map
                self.task_configs.insert(task.task_id, task.clone() );

                self.commander.add_task(task.clone())?;
            }
        }
        Ok(())
    }
    
    fn check_trigger(&self, task: &BrokerTaskConfig) -> Result<bool, anyhow::Error> {
        match &task.trigger {
            BrokerTaskTrigger::Always => Ok(true),
            BrokerTaskTrigger::Rate(timespan) => {
                let now = &self.broker_time;
                let last_execution = &self.task_states[&task.task_id].last_execution_time;
                // if last_execution is None, then the task has never been executed, so return true
                if last_execution.is_none() {
                    return Ok(true);
                }
                let last_execution = last_execution.as_ref().unwrap();
                //TODO: Don't use secs()
                Ok(now.secs() - last_execution.secs() >= timespan.secs())
            }
        }
    }

    fn get_task_inputs(&self, task: &BrokerTaskConfig) -> Result<DataView, BrokerAdapterError> {
        let subscriptions = &task.subscriptions;
        let mut inputs = DataView::new();
        for subscription in subscriptions {
            match subscription.mode{
                SubscriptionMode::Latest => {
                    inputs = inputs.add_query(&mut self.datastore.lock().unwrap(), &subscription.topic_query).unwrap();
                },
                SubscriptionMode::NewValues => {
                    warn!("NewValues subscription not implemented");
                    inputs = inputs.add_query(&mut self.datastore.lock().unwrap(), &subscription.topic_query).unwrap();
                },
            }
        }
        Ok(inputs)
    }
}

#[cfg(test)]
mod broker_tests {
    use std::sync::{Arc, Mutex};

    use victory_data_store::topics::TopicKey;
    use victory_wtf::Timespan;

    use crate::{adapters::mock::MockBrokerAdapter, commander::mock::MockBrokerCommander, task::subscription::BrokerTaskSubscription};

    use super::*;
    use test_env_log::test;


    /// Test the main tick flow
    /// 1. Create a new broker
    /// 2. Add an adapter to the broker
    /// 3. Add a task to the adapter

    #[test]
    fn test_tick(){
        let mut broker = Broker::new(MockBrokerCommander::new());
        let mut adapter = MockBrokerAdapter::new();
        
        let mut task_a = BrokerTaskConfig::new(0);
        task_a.subscriptions.push(BrokerTaskSubscription::new_latest(&TopicKey::from_str("test/a")));
        let mut task_b = BrokerTaskConfig::new(1);
        task_b.subscriptions.push(BrokerTaskSubscription::new_latest(&TopicKey::from_str("test/b")));

        adapter.new_tasks.push(task_a);
        adapter.new_tasks.push(task_b); 

        broker.adapters.insert(0, Arc::new(Mutex::new(adapter)));
        broker.tick().unwrap();
        assert_eq!(broker.task_states[&0].status, BrokerTaskStatus::Completed);
        assert_eq!(broker.task_states[&1].status, BrokerTaskStatus::Idle);

        broker.tick().unwrap();
        assert_eq!(broker.task_states[&0].status, BrokerTaskStatus::Completed);
        assert_eq!(broker.task_states[&1].status, BrokerTaskStatus::Completed);
    }

    /// Test the get_tasks_with_status method
    /// 1. Create a new broker
    /// 2. Call get_tasks_with_status with a status
    /// 3. Check that the returned HashMap is empty
    /// 4. Add a task to the broker (with status Queued)
    /// 5. Call get_tasks_with_status with Queued
    /// 6. Check that the returned HashMap has one entry
    
    #[test]
    fn test_get_tasks_with_status() {
        let mut broker = Broker::new(MockBrokerCommander::new());
        let tasks = broker.get_tasks_with_status(BrokerTaskStatus::Queued);
        assert_eq!(tasks.len(), 0);

        let task = BrokerTaskConfig::new(0);
        let mut task_state = BrokerTaskState::new(0);
        task_state.set_status(BrokerTaskStatus::Queued);

        broker.task_configs.insert(0, task);
        broker.task_states.insert(0, task_state);
        let tasks = broker.get_tasks_with_status(BrokerTaskStatus::Queued);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[&0].task_id, 0);
    }

    #[test]
    fn test_check_trigger_always(){
        let mut broker = Broker::new(MockBrokerCommander::new());
        let mut task = BrokerTaskConfig::new(0);
        task.trigger = BrokerTaskTrigger::Always;
        let trigger = broker.check_trigger(&task);
        assert!(trigger.is_ok());
        assert!(trigger.unwrap(), "Trigger should always return true");
    }

    #[test]
    fn test_check_trigger_rate(){
        let mut broker = Broker::new(MockBrokerCommander::new());
        let mut task = BrokerTaskConfig::new(0);

        broker.task_configs.insert(0, task.clone());
        let mut task_state = BrokerTaskState::new(0);
        task_state.set_status(BrokerTaskStatus::Queued);
        broker.task_states.insert(0, task_state);

        task.trigger = BrokerTaskTrigger::Rate(Timespan::new_hz(1.0));
        let trigger = broker.check_trigger(&task);
        assert!(trigger.is_ok());
        // Time is 0, so trigger should return true
        assert!(trigger.unwrap(), "Trigger should return true");
        
        let broker_time = broker.broker_time.clone();
        // Set the last execution time to now
        broker.task_states.get_mut(&0).unwrap().set_last_execution_time(broker_time.clone());

        // Set the time to 0.5 seconds in the future
        broker.broker_time = broker_time.clone() + Timespan::new_secs(0.5);
        let trigger = broker.check_trigger(&task);
        assert!(trigger.is_ok());
        // Time is 0.5 seconds in the future, so trigger should return false
        assert!(!trigger.unwrap(), "Trigger should return false");


        // Set the time to 1.5 seconds in the future
        broker.broker_time = broker_time.clone() + Timespan::new_secs(1.5);
        let trigger = broker.check_trigger(&task);
        assert!(trigger.is_ok());
        // Time is 1.5 seconds in the future, so trigger should return true
        assert!(trigger.unwrap(), "Trigger should return true");
    }

    // Test the read_new_tasks method
    /// 1. Create a new broker
    /// 2. Add an adapter to the broker
    /// 3. Call read_new_tasks
    /// 4. Check that the task was added to the task_configs map
    /// 5. Check that the task state was added to the task_states map
    #[test]
    fn test_read_new_tasks(){
        let mut broker = Broker::new(MockBrokerCommander::new());
        let mut adapter = MockBrokerAdapter::new();
        // Add a new task to the adapter
        adapter.new_tasks.push(BrokerTaskConfig::new(0));
        broker.adapters.insert(0, Arc::new(Mutex::new(adapter)));
        broker.read_new_tasks().unwrap();
        // Check that the task was added to the task_configs map
        assert_eq!(broker.task_configs.len(), 1);
        // Check that the task state was added to the task_states map
        assert_eq!(broker.task_states.len(), 1);
    }
}