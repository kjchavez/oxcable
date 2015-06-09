use std::sync::mpsc::channel;
use std::thread;

use types::{SAMPLE_RATE, AudioDevice, DeviceIOType, Sample, Time};

pub struct DeviceChain {
    devices: Vec<AudioNode>,
    time: Time
}

impl DeviceChain {
    pub fn from<D>(device: D) -> DeviceChain where D: 'static+AudioDevice {
        match device.num_inputs() {
            DeviceIOType::Exactly(0) => (),
            _ => panic!("DeviceChain: first device can't take any inputs")
        }
        DeviceChain { devices: vec![AudioNode::new(device)], time: 0 }
    }

    pub fn into<D>(mut self, device: D) -> DeviceChain where D: 'static+AudioDevice {
        match device.num_inputs() {
            DeviceIOType::Exactly(ins) => {
                if self.devices[self.devices.len()-1].outputs.len() != ins {
                    panic!("DeviceChain: number of outputs must match number of inputs");
                }
            },
            _ => ()
        }
        self.devices.push(AudioNode::new(device));
        self
    }

    pub fn tick(&mut self) {
        self.devices[0].tick(self.time, &[0.0;0]);
        for i in 1..self.devices.len() {
            let inputs = self.devices[i-1].outputs.clone();
            self.devices[i].tick(self.time, &inputs);
        }
        self.time += 1;
    }

    pub fn tick_until_enter(&mut self) {
        let (tx, rx) = channel();
        let _ = thread::spawn(move || {
            use std::io::{Read, stdin};
            let mut buf = [0];
            let _ = stdin().read(&mut buf);
            assert!(tx.send(()).is_ok());
        });

        let ticks = SAMPLE_RATE / 10;
        loop {
            // Tick for 100ms, then check for exit command
            for _ in 0..ticks {
                self.tick();
            }
            if rx.try_recv().is_ok() {
                break;
            }
        }
    }
}

struct AudioNode {
    device: Box<AudioDevice>,
    outputs: Vec<Sample>
}

impl AudioNode {
    fn new<D>(device: D) -> AudioNode where D: 'static+AudioDevice {
        let n = match device.num_outputs() {
            DeviceIOType::Any => panic!("DeviceChain does not support Any outputs"),
            DeviceIOType::Exactly(i) => i
        };
        let mut outputs = Vec::with_capacity(n);
        for _ in 0..n {
            outputs.push(0.0);
        }
        AudioNode {
            device: Box::new(device),
            outputs: outputs
        }
    }

    fn tick(&mut self, t: Time, inputs: &[Sample]) {
        self.device.tick(t, inputs, &mut self.outputs);
    }
}
