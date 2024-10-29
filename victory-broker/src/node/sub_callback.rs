use std::sync::{Arc, Mutex};

use victory_data_store::datapoints::DatapointMap;

use crate::MutexType;

pub trait SubCallback {
    fn on_update(&mut self, datapoints: &DatapointMap);
}

pub type SubCallbackHandle = MutexType<dyn SubCallback>;
