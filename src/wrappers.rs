//! Tools for wrapping devices.

use types::{AudioDevice, Sample, Time};


/// Bundles an AudioDevice with allocated input and output buffers.
///
/// To use the device, input samples must first be manually dropped into the
/// `inputs` buffer, then `tick` may be called to generate outputs. The output
/// samples can be found in the `outputs` buffer.
pub struct Buffered<D> where D: AudioDevice {
    /// The AudioDevice being wrapped
    pub device: D,
    /// The input buffer
    pub inputs: Vec<Sample>,
    /// The output buffer
    pub outputs: Vec<Sample>,
}

impl<D> Buffered<D> where D: AudioDevice {
    /// Wrap the provided `AudioDevice`, and allocates input and output buffers
    /// for the device.
    pub fn wrap(device: D) -> Self {
        let inputs = device.num_inputs();
        let outputs = device.num_outputs();
        Buffered {
            device: device,
            inputs: vec![0.0; inputs],
            outputs: vec![0.0; outputs],
        }
    }

    /// Calls the device's tick method using the wrapper's buffers.
    pub fn tick(&mut self, t: Time) {
        self.device.tick(t, &self.inputs, &mut self.outputs);
    }
}