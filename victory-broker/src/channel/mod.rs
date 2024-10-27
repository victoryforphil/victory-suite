use std::{collections::HashMap, sync::Arc};

use log::{debug, info};
use thiserror::Error;
use victory_data_store::{
    buckets::BucketHandle, database::Datastore, datapoints::Datapoint, topics::TopicKeyHandle,
};

use crate::{
    client::{PubSubClientHandle, PubSubClientIDType},
    messages::UpdateMessage,
    MutexType,
};

#[derive(Clone)]
pub struct PubSubChannel {
    pub topic: TopicKeyHandle,
    pub bucket: BucketHandle,
    pub publishers: Vec<PubSubClientHandle>,
    pub subscribers: Vec<PubSubClientHandle>,
    pub update_queue: HashMap<PubSubClientIDType, Vec<UpdateMessage>>,
}

pub type PubSubChannelHandle = MutexType<PubSubChannel>;

#[derive(Error, Debug)]
pub enum PubSubChannelError {
    #[error("Generic PubSubChannel Error: {0}")]
    Generic(String),
    #[error("Failed to create new PubSubChannel")]
    CreateFailed,
}
impl PubSubChannel {
    pub fn try_new(
        topic: TopicKeyHandle,
        datastore: &mut Datastore,
    ) -> Result<Self, PubSubChannelError> {
        debug!("Creating new PubSubChannel for topic: {}", topic);
        let bucket = datastore.get_bucket(&topic);

        let bucket = match bucket {
            Ok(bucket) => Ok(bucket),
            Err(_) => {
                debug!("Bucket not found for topic: {}", topic);
                datastore.create_bucket(&topic);
                datastore.get_bucket(&topic)
            }
        };

        if bucket.is_err() {
            return Err(PubSubChannelError::CreateFailed);
        }

        info!("Created new PubSubChannel for topic: {}", topic);
        Ok(PubSubChannel {
            topic,
            bucket: bucket.unwrap(),
            publishers: Vec::new(),
            subscribers: Vec::new(),
            update_queue: HashMap::new(),
        })
    }
    pub fn handle(&self) -> PubSubChannelHandle {
        Arc::new(tokio::sync::Mutex::new(self.clone()))
    }
    pub fn add_publisher(&mut self, client: PubSubClientHandle) {
        debug!(
            "Adding publisher to PubSubChannel: {}",
            client.try_lock().unwrap().id
        );
        self.publishers.push(client);
    }

    pub fn on_publish(&mut self, datapoint: Vec<Datapoint>) {
        debug!("Publishing message to PubSubChannel: {}", self.topic);
        {
            let mut bucket = self.bucket.write().unwrap();
            for dp in datapoint {
                bucket.add_datapoint(dp);
            }
        }
        self.on_update();
    }
    pub fn get_queue_size(&self) -> usize {
        let mut size = 0;
        for (_, updates) in self.update_queue.iter() {
            size += updates.len();
        }
        size
    }

    fn on_update(&mut self) {
        debug!("Updating message to PubSubChannel: {}", self.topic);
        let bucket = self.bucket.read().unwrap();

        for sub in self.subscribers.iter() {
            let value = bucket.get_latest_datapoint().unwrap();
            let update = UpdateMessage::new(value.clone());
            self.update_queue
                .entry(sub.try_lock().unwrap().id)
                .or_insert_with(Vec::new)
                .push(update);
        }
    }

    pub fn get_updates(&mut self, client_id: PubSubClientIDType) -> Vec<UpdateMessage> {
        debug!("Getting updates for PubSubChannel: {}", self.topic);
        let updates = self.update_queue.remove(&client_id);
        match updates {
            Some(updates) => updates,
            None => Vec::new(),
        }
    }

    pub fn add_subscriber(&mut self, client: PubSubClientHandle) {
        debug!(
            "Adding subscriber to PubSubChannel: {}",
            client.try_lock().unwrap().id
        );

        client
            .try_lock()
            .unwrap()
            .subscriptions
            .push(self.topic.clone());

        self.subscribers.push(client);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_pubsub_channel() {}
}
