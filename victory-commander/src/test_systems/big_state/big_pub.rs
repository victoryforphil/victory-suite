use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use victory_data_store::{database::view::DataView, topics::TopicKey};
use victory_wtf::{Timepoint, Timespan};

use crate::system::{System, SystemHandle};
use crate::test_systems::big_state::BigState;

pub struct BigStatePublisher {
    pub name: String,
    pub big_state: BigState,
}

impl BigStatePublisher {
    pub fn new() -> BigStatePublisher {
        BigStatePublisher {
            name: "BigStatePublisher".to_string(),
            big_state: BigState::new(),
        }
    }
}

impl System for BigStatePublisher {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn init(&mut self) {
        // Do nothing
    }

    fn get_subscribed_topics(&self) -> BTreeSet<TopicKey> {
        BTreeSet::new()
    }

    fn execute(&mut self, _inputs: &DataView, _dt: Timespan) -> DataView {
        let mut data_view = DataView::new();
        data_view.add_latest(&TopicKey::from_str("big_state"), &self.big_state);
        data_view
    }

    fn cleanup(&mut self) {
        // Do nothing
    }
}
