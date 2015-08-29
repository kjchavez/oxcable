//! Test script for monitoring MIDI input.
//!
//! The test operates on two levels:
//!
//! 1. All MIDI events are printed to the console.
//! 2. NoteOn/NoteOff events are used to trigger an ADSR.
//!
//! The ADSR gates an oscillator, so that triggering notes can also be heard
//! through the default audio out, and usage of note events can be tested.

extern crate oxcable;


#[cfg(not(test))]
mod wrapper {
    use oxcable::adsr;
    use oxcable::io::midi::MidiIn;
    use oxcable::types::{AudioDevice, MessageReceiver, MidiDevice, MidiMessage,
            Time, Sample};

    pub struct WrappedAdsr {
        pub midi: MidiIn,
        pub adsr: adsr::Adsr
    }
    impl AudioDevice for WrappedAdsr {
        fn num_inputs(&self) -> usize {
            self.adsr.num_inputs()
        }

        fn num_outputs(&self) -> usize {
            self.adsr.num_outputs()
        }

        fn tick(&mut self, t: Time, inputs: &[Sample], outputs: &mut[Sample]) {
            for event in self.midi.get_events(t) {
                println!("{:?}", event);
                match event.payload {
                    MidiMessage::NoteOn(_,_) =>
                        self.adsr.handle_message(adsr::NoteDown),
                    MidiMessage::NoteOff(_,_) =>
                        self.adsr.handle_message(adsr::NoteUp),
                    _ => ()
                }
            }
            self.adsr.tick(t, inputs, outputs);
        }
    }
}


#[cfg(not(test))]
fn main() {
    use oxcable::adsr::Adsr;
    use oxcable::chain::{DeviceChain, Tick};
    use oxcable::io::audio::AudioEngine;
    use oxcable::io::midi::MidiEngine;
    use oxcable::oscillator::{self, Oscillator};

    println!("Initializing signal chain...");
    let audio_engine = AudioEngine::with_buffer_size(256).unwrap();
    let midi_engine = MidiEngine::open().unwrap();

    let mut chain = DeviceChain::from(
        Oscillator::new(oscillator::Saw(oscillator::PolyBlep)).freq(220.0)
    ).into(
        wrapper::WrappedAdsr {
            midi: midi_engine.choose_input().unwrap(),
            adsr: Adsr::default(1)
        }
    ).into(
        audio_engine.default_output(1).unwrap()
    );

    println!("Playing. Press Enter to quit...");
    chain.tick_until_enter();
    println!("Done!");
}
