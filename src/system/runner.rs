use std::{any::Any, marker::PhantomData, sync::{Arc, Mutex}};


use serde::{de::DeserializeOwned, Deserialize, Serialize};
use strum_macros::EnumIter;
use victory_data_store::database::Datastore;
use victory_time_rs::{Timepoint, Timespan};

use super::{System, SystemHandle};


pub struct BasherSysRunner<EnumT: System>
{
    pub systems: Vec<EnumT>,
    pub data_store: Datastore,
    pub end_time: Timepoint,
    pub current_time: Timepoint,
    pub dt: Timespan,
}   

impl<EnumT: System + Iterator> BasherSysRunner<EnumT>
{
    pub fn new() -> BasherSysRunner<EnumT>{
        
        BasherSysRunner {
            systems: Vec::new(),
            data_store: Datastore::new(),
            end_time: Timepoint::zero(),
            current_time: Timepoint::zero(),
            dt: Timespan::new_hz(100.0),
        }
    }

    pub fn run(&mut self, end_time: Timepoint){
        self.end_time = end_time;
        self.current_time = Timepoint::zero();
        
    }
}

