//! A antialiasing oscillator.
//!
//! ## Waveforms
//!
//! The oscillator supports several classical waveforms. The square, triangle,
//! and saw waves support both aliased and antialiased variants.
//!
//! The aliased variants produce pure signals; for example, a square wave only
//! emits two values: `-1.0` and `+1.0`. This is useful when a control signal is
//! wanted, but produces aliasing in the frequency domain.
//!
//! The antialiased variants use PolyBLEP (polynomial bandlimited step) to
//! mitigate the aliasing in the pure signals. This results in a much cleaner
//! audible signal, and is more desirable for most musical purposes.
//!
//! ## Pitch Bend
//!
//! The oscillator supports pitch bending as an additional modifier of the base
//! frequency. This makes it easier to bend a single note when working with
//! MIDI, rather than manually tracking the base frequency and computing a new
//! frequency yourself.
//!
//! ## Low Frequency Oscillator
//!
//! The oscillator supports a low frequency oscillator (LFO) as optional input.
//! If provided, then the LFO is used to modulate the frequency of the
//! oscillator, producing a vibrato.
//!
//! ## Example
//!
//! The oscillator uses a builder pattern for initialization. The following will
//! set up an antialiased saw wave at 440 Hz, with a 0.1 step vibrato:
//!
//! ```
//! use oxcable::oscillator::*;
//! let osc = Oscillator::new(Saw(PolyBlep)).freq(440.0).lfo_intensity(0.1);
//! ```

use std::f32::consts::PI;
use num::traits::Float;
use rand::random;

use types::{SAMPLE_RATE, AudioDevice, MessageReceiver, Sample, Time};


/// Defines the messages that the Oscillator supports.
#[derive(Clone, Copy, Debug)]
pub enum Message {
    /// Sets the frequency in Hz.
    SetFreq(f32),
    /// Sets the waveform type.
    SetWaveform(Waveform),
    /// Sets the LFO vibrato depth, in steps.
    SetLFOIntensity(f32),
    /// Sets the pitch transposition, in steps.
    SetTranspose(f32),
    /// Sets the pitch bend, in steps.
    SetBend(f32),
}
pub use self::Message::*;


/// Antialiasing method for certain waveforms.
#[derive(Clone, Copy, Debug)]
pub enum AntialiasType {
    /// Naive, aliasing waveforms.
    Aliased,
    /// Antialiasing using PolyBLEP.
    PolyBlep
}
pub use self::AntialiasType::*;


/// Oscillator waveforms.
#[derive(Clone, Copy, Debug)]
pub enum Waveform {
    /// A sine wave.
    Sine,
    /// A saw wave.
    Saw(AntialiasType),
    /// A square wave.
    Square(AntialiasType),
    /// A triangle wave.
    Tri(AntialiasType),
    /// Pure white noise.
    WhiteNoise,
    /// A series of impulses.
    PulseTrain
}
pub use self::Waveform::*;


/// An oscillator that generates a periodic waveform.
pub struct Oscillator {
    waveform: Waveform,
    lfo_intensity: f32,
    transpose: f32,
    bend: f32,
    phase: f32,
    phase_delta: f32,
    last_sample: Sample,
}

impl Oscillator {
    /// Returns an oscillator with the specified waveform.
    pub fn new(waveform: Waveform) -> Self {
        Oscillator {
            waveform: waveform,
            lfo_intensity: 0.0,
            transpose: 1.0,
            bend: 1.0,
            phase: 0.0,
            phase_delta: 0.0,
            last_sample: 0.0
        }
    }

    /// Sets the frequency of the waveform, and return the same oscillator.
    pub fn freq(mut self, freq: f32) -> Self {
        self.handle_message(SetFreq(freq));
        self
    }

    /// Sets the frequency transposition (in steps), and return the same
    /// oscillator.
    pub fn transpose(mut self, steps: f32) -> Self {
        self.handle_message(SetTranspose(steps));
        self
    }

    /// Sets the intensity of the LFO vibrato, and return the same oscillator.
    ///
    /// The intensity is provided in half steps (1/2ths of an octave).
    pub fn lfo_intensity(mut self, lfo_intensity: f32) -> Self {
        self.handle_message(SetLFOIntensity(lfo_intensity));
        self
    }
}

impl MessageReceiver for Oscillator {
    type Msg = Message;
    fn handle_message(&mut self, msg: Message) {
        match msg {
            SetFreq(freq) => {
                self.phase_delta = freq*2.0*PI/(SAMPLE_RATE as f32);
            },
            SetWaveform(waveform) => {
                self.waveform = waveform;
            },
            SetLFOIntensity(steps) => {
                self.lfo_intensity = steps/12.0;
            },
            SetTranspose(steps) => {
                self.transpose = 2.0.powf(steps/12.0);
            },
            SetBend(steps) => {
                self.bend = 2.0.powf(steps/12.0);
            },
        }
    }
}

impl AudioDevice for Oscillator {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn tick(&mut self, _: Time, inputs: &[Sample], outputs: &mut[Sample]) {
        // Tick the phase
        let phase_delta = if inputs.len() > 0 {
            self.phase_delta*2.0.powf(inputs[0]*self.lfo_intensity)
        } else {
            self.phase_delta
        } * self.bend * self.transpose;
        self.phase += phase_delta;
        if self.phase >= 2.0*PI {
            self.phase -= 2.0*PI;
        }

        // Compute the next sample
        self.last_sample = match self.waveform {
            Sine => self.phase.sin(),
            Saw(_) => {
                self.phase/PI -1.0 +
                    poly_blep(self.waveform, self.phase, phase_delta)
            },
            Square(_) => {
                (if self.phase < PI { 1.0 } else { -1.0 }) +
                    poly_blep(self.waveform, self.phase, phase_delta)
            },
            Tri(_) => {
                // Compute a square wave signal
                let out = (if self.phase < PI { 1.0 } else { -1.0 }) +
                    poly_blep(self.waveform, self.phase, phase_delta);

                // Perform leaky integration
                phase_delta*out + (1.0-phase_delta)*self.last_sample
            },
            WhiteNoise => 2.0*random::<f32>() - 1.0,
            PulseTrain => {
                // If we wrapped around...
                if self.phase < self.phase_delta { 1.0 } else { 0.0 }
            }
        };
        outputs[0] = self.last_sample;
    }
}


/// Computes the PolyBLEP step for a given waveform type. This should be added
/// to the naive waveform.
///
/// `waveform` should be the waveform we are antialiasing.
/// `phase` should be the current phase, from 0 to 2pi.
/// `phase_delta` should be the change in phase for one tick.
fn poly_blep(waveform: Waveform, phase: f32, phase_delta: f32) -> Sample {
    match waveform {
        Saw(PolyBlep) => {
            -1.0*poly_blep_offset(phase/(2.0*PI), phase_delta/(2.0*PI))
        },
        Square(PolyBlep) | Tri(PolyBlep) => {
            let t = phase/(2.0*PI);
            let dt = phase_delta/(2.0*PI);
            poly_blep_offset(t, dt) - poly_blep_offset((t+0.5) % 1.0, dt)
        },
        _ => 0.0
    }
}

/// Computes a single offset for PolyBLEP antialiasing.
///
/// `t` should be the current waveform phase, normalized.
/// `dt` should be the change in phase for one sample time, normalized.
fn poly_blep_offset(t: f32, dt: f32) -> f32 {
    if t < dt { // t ~= 0
        let t = t / dt;
        -t*t + 2.0*t - 1.0
    } else if t > 1.0-dt { // t ~= 1
        let t = (t-1.0) / dt;
        t*t + 2.0*t + 1.0
    } else {
        0.0
    }
}


#[cfg(test)]
mod test {
    use testing::flt_eq;
    use super::{Aliased, Oscillator, Waveform};
    use types::{AudioDevice, Sample, Time};

    const FREQ: f32 = 4410.0;
    fn get_one_cycle(osc: &mut Oscillator) -> [Sample; 10] {
        let mut output = [0.0; 10];
        for t in 0..10 {
            osc.tick(t as Time, &[], &mut output[t..t+1]);
        }
        output
    }

    fn check(actual: &[Sample], expected: &[Sample]) {
        println!("actual:\n    {:?}", actual);
        println!("expected:\n    {:?}", expected);
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!(flt_eq(*a, *e));
        }
    }

    #[test]
    fn test_naive_square() {
        let mut osc = Oscillator::new(Waveform::Square(Aliased)).freq(FREQ);
        check(&get_one_cycle(&mut osc),
              &[1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0, -1.0, 1.0]);
    }

    #[test]
    fn test_naive_saw() {
        let mut osc = Oscillator::new(Waveform::Saw(Aliased)).freq(FREQ);
        check(&get_one_cycle(&mut osc),
              &[-0.8, -0.6, -0.4, -0.2, 0.0, 0.2, 0.4, 0.6, 0.8, -1.0]);
    }

    #[test]
    fn test_pulse() {
        let mut osc = Oscillator::new(Waveform::PulseTrain).freq(FREQ);
        check(&get_one_cycle(&mut osc),
              &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
    }
}
