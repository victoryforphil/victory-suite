use crate::task::config::BrokerTaskConfig;

use super::{BrokerCommander, BrokerCommanderError};

pub struct LinearBrokerCommander {
    tasks: Vec<BrokerTaskConfig>,
    current_task: usize,
}

impl LinearBrokerCommander {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current_task: 0,
        }
    }
}

impl BrokerCommander for LinearBrokerCommander {
    fn add_task(&mut self, task: BrokerTaskConfig) -> Result<(), BrokerCommanderError> {
        self.tasks.push(task);
        Ok(())
    }

    fn get_next_tasks(&mut self) -> Result<Vec<BrokerTaskConfig>, BrokerCommanderError> {
        if self.tasks.is_empty() {
            
            return Ok(Vec::new());
        }

        // Get current task
        let task = self.tasks[self.current_task].clone();
        
        // Increment and wrap around
        self.current_task = (self.current_task + 1) % self.tasks.len();

        Ok(vec![task])
    }
}

#[cfg(test)]
mod linear_commander_tests{
    use crate::task::config::BrokerTaskConfig;

    use super::*;

    #[test]
    fn test_add_task(){
        let mut commander = LinearBrokerCommander::new();
        let new_with_id = BrokerTaskConfig::new_with_id(0, "test");
        let task = new_with_id;
        commander.add_task(task).unwrap();
    }

  
}