use blob::VicBlob;
use timestamp::{VicInstant, VicTimecode};

use crate::topics::TopicIDType;

pub mod integer;

pub mod blob;
pub mod timestamp;
#[derive(Debug, Clone, PartialEq)]
pub enum Primitives {
    Timestamp(VicInstant),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(VicBlob),
    Boolean(bool),
    List(Vec<Primitives>),
    Reference(TopicIDType),
}
