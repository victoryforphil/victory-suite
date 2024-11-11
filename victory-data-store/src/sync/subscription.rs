use std::collections::VecDeque;

use log::debug;

use crate::{datapoints::Datapoint, topics::TopicKey};

use super::{SyncConnectionIDType, SyncSubscriptionIDType};


// Represents an active subscription of a local datastore to a remote channel
#[derive(Debug, Clone)]
pub struct Subscription{
    pub sub_id: SyncSubscriptionIDType,
    pub connection_id: Option<SyncConnectionIDType>,
    pub client_name: String,
    pub topic_query: TopicKey,
    pub queue: VecDeque<Datapoint>,
}

impl Subscription{
    pub fn new(client_name: String, topic: TopicKey) -> Self{
        let sub_id = rand::random();
        debug!("[Sync/Subscription] New subscription: {:?} for client: {:?} (ID: {:?})", topic, client_name, sub_id);
        Self{sub_id, connection_id: None, client_name, topic_query: topic, queue: VecDeque::new()}
    }

    /// Checks to see if a given datapoint is a match / child of this subscription
    pub fn is_match(&self, datapoint: &Datapoint) -> bool{
        datapoint.topic.matches(&self.topic_query)
    }

    /// Add a new datapoint to the subscription queue
    pub fn push(&mut self, datapoint: Datapoint){
        self.queue.push_back(datapoint);
    }

    /// Get the next datapoint from the subscription queue
    pub fn pop(&mut self) -> Option<Datapoint>{
        self.queue.pop_front()
    }

    /// Get the number of datapoints in the subscription queue
    pub fn len(&self) -> usize{
        self.queue.len()
    }
}