use victory_time_rs::{Timespan};

#[derive(Debug, Clone)]
pub struct PubSubServerConfig {
    pub collect_rate: Timespan,
}
