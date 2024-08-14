use blob::VicBlob;
use timestamp::{VicInstant, VicTimecode};

use crate::topics::TopicIDType;

pub mod timestamp;
pub mod blob;
#[derive(Debug, Clone, PartialEq)]
pub enum Primitives{
    Timestamp(VicInstant),
    Integer(u64),
    Float(f64),
    Text(String),
    Blob(VicBlob),
    Boolean(bool),
    List(Vec<Primitives>),
    Reference(TopicIDType),
}

