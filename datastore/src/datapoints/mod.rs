use crate::{
    primitives::{timestamp::VicInstantHandle, Primitives},
    topics::TopicKeyHandle,
};
#[derive(Debug, Clone)]
pub struct Datapoint {
    pub topic: TopicKeyHandle,
    pub time: VicInstantHandle,
    pub value: Primitives,
}
