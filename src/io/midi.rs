//! Provides MIDI input from OS MIDI devices.

#![experimental]

extern crate portmidi;

use std::vec::Vec;

use core::types::{Device, MidiEvent, MidiMessage, Time};
use core::components::OutputElement;
use core::init;


/// Defines the maximum event buffer size for portmidi
static BUFFER_SIZE: int = 256;


/// Converts a raw portmidi message to an oxcable MIDI event
fn midievent_from_portmidi(event: portmidi::midi::PmEvent) -> MidiEvent {
    let msg = event.message;
    let channel = (msg.status & 0x0F) as u8;
    let payload = match (msg.status as u8) >> 4 {
        0b1000 => {
            let note = msg.data1 as u8;
            let velocity = (msg.data2 as f32) / 127.0;
            MidiMessage::NoteOff(note, velocity)
        },
        0b1001 => {
            let note = msg.data1 as u8;
            let velocity = (msg.data2 as f32) / 127.0;
            MidiMessage::NoteOn(note, velocity)
        }
        0b1110 => {
            let int_value = (msg.data2 as i16 << 7) | (msg.data1 as i16);
            let bend = (int_value - 0x2000) as f32 / 
                (0x2000i16) as f32;
            MidiMessage::PitchBend(bend)
        }
        0b1010 => {
            let note = msg.data1 as u8;
            let pressure = (msg.data2 as f32) / 127.0;
            MidiMessage::KeyPressure(note, pressure)
        }
        0b1011 => MidiMessage::ControlChange(msg.data1 as u8, msg.data2 as u8),
        0b1100 => MidiMessage::ProgramChange(msg.data1 as u8),
        0b1101 => MidiMessage::ChannelPressure(msg.data1 as f32 / 127.0),
        _ => MidiMessage::Other(msg.status as u8, msg.data1 as u8, 
                                msg.data2 as u8)
    };

    MidiEvent { channel: channel, payload: payload }
}


/// Reads audio from the OS's default midi device.
pub struct MidiIn {
    /// Output midi channel
    pub output: OutputElement<Vec<MidiEvent>>,

    pm_stream: portmidi::midi::PmInputPort,
}

impl MidiIn {
    /// Opens a midi input stream.
    pub fn new() -> MidiIn {
        // Check for initialization
        if !init::is_initialized() {
            panic!("Must initialize oxcable first");
        }
        
        // Open a stream. For now, use firs device
        let mut pm_stream = portmidi::midi::PmInputPort::new(1, BUFFER_SIZE);
        assert_eq!(pm_stream.open(), portmidi::midi::PmError::PmNoError);

        MidiIn {
            output: OutputElement::new(),
            pm_stream: pm_stream,
        }
    }

    /// Closes the portmidi stream
    pub fn stop(&mut self) {
        assert_eq!(self.pm_stream.close(), portmidi::midi::PmError::PmNoError);
    }
}

impl Device for MidiIn {
    fn tick(&mut self, _t: Time) {
        let mut events = Vec::new();
        while self.pm_stream.poll() == portmidi::midi::PmError::PmGotData {
            let pm_message = self.pm_stream.read().unwrap();
            let event = midievent_from_portmidi(pm_message);
            events.push(event);
        }
        self.output.push(events);
    }
}