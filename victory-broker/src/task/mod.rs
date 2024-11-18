use std::sync::{Arc, Mutex};


pub mod state;
pub mod subscription;
pub mod trigger;

pub mod config;

pub type BrokerTaskID = u32;



pub type BrokerTaskHandle = Arc<Mutex<dyn BrokerTask>>;

pub trait BrokerTask{
   
}