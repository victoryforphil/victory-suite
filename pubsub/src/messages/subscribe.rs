use datastore::topics::TopicKey;
#[derive(Debug, Clone)]
pub struct SubscribeMessage {
    pub topic: TopicKey,
}
