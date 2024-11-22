use victory_data_store::{database::view::DataView, topics::TopicKey};

use crate::task::{config::BrokerTaskConfig, trigger::BrokerTaskTrigger, BrokerTask};

pub struct TaskTicker {
    pub publish_topic: TopicKey,
    tick_value: u64,
}

impl TaskTicker {
    pub fn new(publish_topic: TopicKey) -> Self {
        Self {
            publish_topic,
            tick_value: 0,
        }
    }
}

impl BrokerTask for TaskTicker {
    fn get_config(&self) -> crate::task::config::BrokerTaskConfig {
        BrokerTaskConfig::new("TaskTicker").with_trigger(BrokerTaskTrigger::Always)
    }

    fn on_execute(
        &mut self,
        _inputs: &victory_data_store::database::view::DataView,
    ) -> Result<victory_data_store::database::view::DataView, anyhow::Error> {
        let mut outputs = DataView::new();
        self.tick_value += 1;
        outputs.add_latest(&self.publish_topic, self.tick_value);
        Ok(outputs)
    }
    
    fn init(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ticker_counts() {
        let topic = TopicKey::from_str("test_topic");
        let mut ticker = TaskTicker::new(topic.clone());

        // First execution
        let inputs = DataView::new();
        let outputs = ticker.on_execute(&inputs).unwrap();
        assert_eq!(outputs.get_latest::<_, u64>(&topic).unwrap(), 1);

        // Second execution - should output same value
        let outputs = ticker.on_execute(&inputs).unwrap();
        assert_eq!(outputs.get_latest::<_, u64>(&topic).unwrap(), 2);
    }
}
