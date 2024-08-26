use blob::VicBlob;

use crate::{
    time::{VicDuration, VicInstant},
    topics::TopicIDType,
};

pub mod blob;
pub mod integer;
pub mod string;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Primitives {
    Instant(VicInstant),
    Duration(VicDuration),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(VicBlob),
    Boolean(bool),
    List(Vec<Primitives>),
    Reference(TopicIDType),
}
