//! Defines AudioDevices for getting audio into and out of oxcable.

#![experimental]

extern crate portaudio;

pub static PORTAUDIO_T: portaudio::types::PaSampleFormat = 
    portaudio::types::PaFloat32;
pub static BUFFER_SIZE: uint = 256;

pub mod microphone;
pub mod speaker;
pub mod wav;
