pub mod mock;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};


use crate::{
    client::PubSubClientIDType,
    messages::PubSubMessage,
};

pub trait PubSubAdapter {
    fn read(&mut self) -> HashMap<PubSubClientIDType, Vec<PubSubMessage>>;
    fn write(&mut self, to_send: HashMap<PubSubClientIDType, Vec<PubSubMessage>>);
}
pub type PubSubAdapterHandle = Arc<Mutex<dyn PubSubAdapter>>;
