
use serde::{Serialize, Deserialize};
use victory_time_rs::Timespan;
use crate::system::System;
#[derive(Serialize, Deserialize)]
pub struct MockInputs{
    pub test_value_in: u32,
}


#[derive(Serialize, Deserialize)]
pub struct MockOutputs{
    pub test_value_out: u32,
}


pub struct MockSystem{
    state_test_value: u32,
}

impl MockSystem{
    pub fn new() -> MockSystem{
        MockSystem{state_test_value: 0}
    }
}

impl System for MockSystem{
   
    type Inputs = MockInputs;
    type Outputs = MockOutputs;
    
    fn init(&mut self) {
        
    }
    
    fn execute(&mut self, inputs: Self::Inputs, dt: Timespan) -> Self::Outputs {
        MockOutputs{test_value_out: inputs.test_value_in}
    }
    
    fn cleanup(&mut self) {
            
    }

    
}