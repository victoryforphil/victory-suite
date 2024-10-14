use blob::VicBlob;
use ::serde::{Deserialize, Serialize};
use victory_time_rs::{Timepoint, Timespan};

use crate::topics::TopicIDType;

pub mod blob;
pub mod integer;
pub mod float;  
pub mod bool; 
pub mod string;
pub mod serde;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Primitives {
    Unset,
    Instant(Timepoint),
    Duration(Timespan),
    Integer(i64),
    Float(f64),
    Text(String),
    Blob(VicBlob),
    Boolean(bool),
    List(Vec<Primitives>),
    Reference(TopicIDType),
    StructType(String),
}
