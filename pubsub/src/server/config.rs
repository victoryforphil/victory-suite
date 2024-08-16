use datastore::primitives::timestamp::VicDuration;

#[derive(Debug, Clone)]
pub struct PubSubServerConfig {
    pub collect_rate: VicDuration,
}
