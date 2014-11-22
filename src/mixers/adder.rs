//! Device for adding multiple channels into one

#![experimental]

use core::components::{InputArray, OutputArray};
use core::types::{Device, Sample, Time};


/// Sums all its inputs into a single output
pub struct Adder {
    /// The input array, receiving many signals
    pub inputs: InputArray<Sample>,
    /// The output array, with a single channel
    pub output: OutputArray<Sample>,

    num_inputs: uint, 
}

impl Adder {
    /// Returns a new adder with `num_inputs` input channels
    pub fn new(num_inputs: uint) -> Adder {
        Adder {
            inputs: InputArray::new(num_inputs),
            output: OutputArray::new(1),
            num_inputs: num_inputs
        }
    }
}

impl Device for Adder {
    fn tick(&mut self, t: Time) {
        let mut s = 0.0;
        for i in range(0, self.num_inputs) {
            s += self.inputs.get(i, t).unwrap_or(0.0);
        }
        self.output.push(0, s);
    }
}
