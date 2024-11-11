use crate::system::System;
use serde::{Deserialize, Serialize};
use victory_data_store::{database::view::DataView, topics::TopicKey};
use victory_wtf::Timespan;

#[derive(Serialize, Deserialize)]
pub struct TimeScaleData {
    pub test_value: f32,
}

pub struct TimeScaleSystem {
    pub topic_to_scale: TopicKey,
}

impl TimeScaleSystem {
    pub fn new(sub_topic: &str) -> TimeScaleSystem {
        TimeScaleSystem {
            topic_to_scale: sub_topic.into(),
        }
    }
}

impl System for TimeScaleSystem {
    fn name(&self) -> String {
        "TimeScaleSystem".to_string()
    }
    fn init(&mut self) {}
    fn cleanup(&mut self) {}

    fn execute<'a>(&mut self, inputs_data: &'a DataView, dt: Timespan) -> DataView {
        let inputs: TimeScaleData = inputs_data.get_latest(&self.topic_to_scale).unwrap();

        let mut outputs = DataView::new();
        let out_data = TimeScaleData {
            test_value: inputs.test_value + dt.secs() as f32,
        };

        outputs.add_latest(&self.topic_to_scale.clone(), out_data);
        outputs
    }

    fn get_subscribed_topics(
        &self,
    ) -> std::collections::BTreeSet<victory_data_store::topics::TopicKey> {
        let mut topics = std::collections::BTreeSet::new();
        topics.insert(self.topic_to_scale.clone());
        topics
    }
}
