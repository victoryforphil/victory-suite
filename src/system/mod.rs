use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, Mutex}};

use enum_dispatch::enum_dispatch;
use serde::{de::DeserializeOwned, Serialize};
use victory_data_store::{database::Datastore, primitives::{serde::serialize::to_map, Primitives}, topics::TopicKey};
use victory_time_rs::{Timepoint, Timespan};



/// Trait that defines a commander system that can be run by the commander.
/// Will use its inputs to generate a data store query to feed said inputs into the system.
pub mod runner;

pub type SubscriberMap = BTreeSet<TopicKey>;
pub type InputMap = BTreeMap<TopicKey, Primitives>;
pub type OutputMap = BTreeMap<TopicKey, Primitives>;

pub struct SubscriberBuilder{}
impl SubscriberBuilder{
    pub fn new<T: Serialize + Default>() -> SubscriberMap{
        let example_input = T::default();
        let mut map = to_map(&example_input).unwrap();
        map.into_iter().map(|(key, _)| key).collect()
    }
}
pub trait System {
   
    fn init(&mut self);
    fn execute(&mut self, inputs: InputMap, dt: Timespan) -> OutputMap;
    fn cleanup(&mut self);
}

pub type SystemHandle = Arc<Mutex<dyn System>>;
