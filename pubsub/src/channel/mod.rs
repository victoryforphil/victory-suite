use std::sync::{Arc, Mutex};

use datastore::{buckets::BucketHandle, database::Datastore, topics::TopicKeyHandle};
use log::{debug, info};
use thiserror::Error;

use crate::client::PubSubClientHandle;

#[derive(Clone)]
pub struct PubSubChannel {
    pub topic: TopicKeyHandle,
    pub bucket: BucketHandle,
    pub publishers: Vec<PubSubClientHandle>,
    pub subscribers: Vec<PubSubClientHandle>,
}

pub type PubSubChannelHandle = Arc<Mutex<PubSubChannel>>;

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
        })
    }
    pub fn handle(&self) -> PubSubChannelHandle {
        Arc::new(Mutex::new(self.clone()))
    }
    pub fn add_publisher(&mut self, client: PubSubClientHandle) {
        debug!(
            "Adding publisher to PubSubChannel: {}",
            client.lock().unwrap().id
        );
        self.publishers.push(client);
    }

    pub fn add_subscriber(&mut self, client: PubSubClientHandle) {
        debug!(
            "Adding subscriber to PubSubChannel: {}",
            client.lock().unwrap().id
        );
        self.subscribers.push(client);
    }
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_pubsub_channel() {}
}
