use crate::{
    adapters::PubSubAdapterHandle, channel::PubSubChannelHandle, client::PubSubClientHandle,
};

pub struct PubSubServer {
    pub channels: Vec<PubSubChannelHandle>,
    pub floating_clinets: Vec<PubSubClientHandle>,
    pub parent_adapters: Vec<PubSubAdapterHandle>,
}
