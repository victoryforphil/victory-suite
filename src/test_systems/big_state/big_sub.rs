use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use log::debug;
use victory_data_store::database::{DataView, Datastore};
use victory_data_store::topics::TopicKey;
use victory_time_rs::{Timepoint, Timespan};

use crate::system::System;
use crate::test_systems::big_state::BigState;

pub struct BigStateSubscriber {
    pub name: String,
}

impl BigStateSubscriber {
    pub fn new() -> BigStateSubscriber {
        BigStateSubscriber {
            name: "BigStateSubscriber".to_string(),
        }
    }
}

impl System for BigStateSubscriber {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&mut self) {
        // Do nothing
    }

    fn get_subscribed_topics(&self) -> BTreeSet<TopicKey> {
        let mut topics = BTreeSet::new();
        topics.insert(TopicKey::from_str("big_state"));
        topics
    }

    fn execute(&mut self, inputs: &DataView, _dt: Timespan) -> DataView {
        let big_state: BigState = inputs.get_latest(&TopicKey::from_str("big_state")).unwrap();
        debug!("{:?}", big_state.pose.position);
        DataView::new()
    }

    fn cleanup(&mut self) {}
}
