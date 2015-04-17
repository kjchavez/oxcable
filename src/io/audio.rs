//! Provides audio IO from OS sound devices.

extern crate portaudio;

use std::rc::Rc;

use types::{SAMPLE_RATE, Device, Sample, Time};
use components::{InputArray, OutputArray};


/// Defines the audio format for Portaudio.
static PORTAUDIO_T: portaudio::pa::SampleFormat =
    portaudio::pa::SampleFormat::Float32;

/// Defines the buffer size for Portaudio
static BUFFER_SIZE: usize = 256;


/// Used to handle portaudio resources.
pub struct AudioEngine;

impl AudioEngine {
    pub fn open() -> Result<AudioEngine, &'static str> {
        if portaudio::pa::initialize().is_err() {
            return Result::Err("failed to initialize portaudio");
        }
        Result::Ok(AudioEngine)
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self)
    {
        assert!(portaudio::pa::terminate().is_ok());
    }
}


/// Reads audio from the OS's default input device.
pub struct AudioIn {
    /// Output audio channels
    pub outputs: OutputArray<Sample>,

    #[allow(dead_code)] // the engine is used as an RAII marker
    engine: Rc<AudioEngine>,
    pa_stream: portaudio::pa::Stream<Sample, Sample>,
    num_channels: usize,
    buffer: Vec<Sample>,
    samples_read: usize,
}

impl AudioIn {
    /// Opens an audio input stream reading `num_channels` inputs.
    pub fn new(engine: Rc<AudioEngine>, num_channels: usize) -> AudioIn {
        // Open a stream
        let mut pa_stream = portaudio::pa::Stream::new();
        assert!(pa_stream.open_default(SAMPLE_RATE as f64, BUFFER_SIZE as u32,
                                       num_channels as i32, 0i32,
                                       PORTAUDIO_T).is_ok());
        assert!(pa_stream.start().is_ok());

        AudioIn {
            outputs: OutputArray::new(num_channels),
            engine: engine,
            pa_stream: pa_stream,
            num_channels: num_channels,
            buffer: Vec::with_capacity(num_channels*BUFFER_SIZE),
            samples_read: BUFFER_SIZE,
        }
    }
}

impl Drop for AudioIn {
    fn drop(&mut self) {
        assert!(self.pa_stream.stop().is_ok());
        assert!(self.pa_stream.close().is_ok());
    }
}

impl Device for AudioIn {
    fn tick(&mut self, _t: Time) {
        if self.samples_read == BUFFER_SIZE {
            let result = self.pa_stream.read(BUFFER_SIZE as u32);
            match result {
                Ok(v) => self.buffer = v.clone(),
                Err(e) => panic!(e)
            }
            self.samples_read = 0;
        }

        for i in (0 .. self.num_channels) {
            let s = self.buffer[self.samples_read*self.num_channels + i];
            self.outputs.push(i, s);
        }
        self.samples_read += 1;
    }
}


/// Writes audio to the OS's default output device.
pub struct AudioOut {
    /// Input audio channels
    pub inputs: InputArray<Sample>,

    #[allow(dead_code)] // the engine is used as an RAII marker
    engine: Rc<AudioEngine>,
    pa_stream: portaudio::pa::Stream<Sample, Sample>,
    num_channels: usize,
    buffer: Vec<Sample>,
    samples_written: usize,
}

impl AudioOut {
    /// Opens an output stream writing `num_channels` outputs.
    pub fn new(engine: Rc<AudioEngine>, num_channels: usize) -> AudioOut {
        // Open a stream
        let mut pa_stream = portaudio::pa::Stream::new();
        assert!(pa_stream.open_default(SAMPLE_RATE as f64, BUFFER_SIZE as u32,
                                       0i32, num_channels as i32,
                                       PORTAUDIO_T).is_ok());
        assert!(pa_stream.start().is_ok());

        AudioOut {
            inputs: InputArray::new(num_channels),
            engine: engine,
            pa_stream: pa_stream,
            num_channels: num_channels,
            buffer: Vec::with_capacity(num_channels*BUFFER_SIZE),
            samples_written: 0,
        }
    }
}

impl Drop for AudioOut {
    fn drop(&mut self) {
        assert!(self.pa_stream.stop().is_ok());
        assert!(self.pa_stream.close().is_ok());
    }
}

impl Device for AudioOut {
    fn tick(&mut self, t: Time) {
        for i in (0 .. self.num_channels) {
            let mut s = self.inputs.get(i, t).unwrap_or(0.0);
            if s > 1.0 { s = 1.0; }
            if s < -1.0 { s = -1.0; }
            self.buffer.push(s)
        }
        self.samples_written += 1;

        if self.samples_written == BUFFER_SIZE {
            assert!(self.pa_stream.write(self.buffer.clone(),
                                         BUFFER_SIZE as u32).is_ok());
            self.samples_written = 0;
            self.buffer.clear()
        }
    }
}
