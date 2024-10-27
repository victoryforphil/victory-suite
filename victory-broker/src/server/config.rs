use victory_wtf::Timespan;

#[derive(Debug, Clone)]
pub struct PubSubServerConfig {
    pub collect_rate: Timespan,
}
