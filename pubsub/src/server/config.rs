use datastore::time::VicDuration;

#[derive(Debug, Clone)]
pub struct PubSubServerConfig {
    pub collect_rate: VicDuration,
}
