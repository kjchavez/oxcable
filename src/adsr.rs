//! Provides an ADSR filter

use types::{SAMPLE_RATE, AudioDevice, Sample, Time};


/// Defines the messages that the ADSR supports
#[derive(Clone, Copy)]
pub enum AdsrMessage {
    NoteDown,
    NoteUp,
    SetAttack(f32),
    SetDecay(f32),
    SetSustain(f32),
    SetRelease(f32),
}


/// Defines the current mode the ADSR is operating in
enum AdsrState { Silent, Attack, Decay, Sustain, Release }

impl AdsrState {
    /// Given the current state, gets our next state
    fn next(&self) -> AdsrState {
        match self {
            &AdsrState::Attack  => AdsrState::Decay,
            &AdsrState::Decay   => AdsrState::Sustain,
            &AdsrState::Sustain => AdsrState::Release,
            &AdsrState::Release => AdsrState::Silent,
            &AdsrState::Silent  => AdsrState::Silent
        }
    }
}


/// A multichannel ADSR filter
pub struct Adsr {
    // Remember parameter values
    num_channels: usize,
    attack_time: Time,
    decay_time: Time,
    release_time: Time,
    sustain_level: f32,

    // Track state
    current_state: AdsrState,
    next_state_change: Time,
    multiplier: f32,
    multiplier_delta: f32,
    last_time: Time
}

impl Adsr {
    /// Returns a new ADSR filter with the provided envelope settings.
    ///
    /// * `attack_time` specifies the length of the attack in seconds.
    /// * `decay_time` specifies the length of the decay in seconds.
    /// * `sustain_level` specifies the amplitude of the sustain from 0 to 1.
    /// * `release_time` specifies the length of the release in seconds.
    /// * `num_channels` defines how many channels of audio to filter.
    pub fn new(attack_time: f32, decay_time: f32, sustain_level: f32,
               release_time: f32, num_channels: usize) -> Adsr {
        // Convert times to samples
        let attack_samples = (attack_time*SAMPLE_RATE as f32) as Time;
        let decay_samples = (decay_time*SAMPLE_RATE as f32) as Time;
        let release_samples = (release_time*SAMPLE_RATE as f32) as Time;

        Adsr {
            num_channels: num_channels,
            attack_time: attack_samples,
            decay_time: decay_samples,
            release_time: release_samples,
            sustain_level: sustain_level,
            current_state: AdsrState::Silent,
            next_state_change: 0,
            multiplier: 0.0,
            multiplier_delta: 0.0,
            last_time: 0,
        }
    }

    /// Returns an ADSR with reasonable default values for the envelope.
    pub fn default(num_channels: usize) -> Adsr {
        Adsr::new(0.05, 0.1, 0.5, 0.1, num_channels)
    }

    /// Applies the message to our Adsr
    pub fn handle_message(&mut self, msg: AdsrMessage) {
        let t = self.last_time;
        match msg {
            AdsrMessage::NoteDown => {
                self.handle_state_change(AdsrState::Attack, t);
            },
            AdsrMessage::NoteUp => {
                self.handle_state_change(AdsrState::Release, t);
            },
            AdsrMessage::SetAttack(attack) => {
                self.attack_time = (attack*SAMPLE_RATE as f32) as Time;
            },
            AdsrMessage::SetDecay(decay) => {
                self.decay_time = (decay*SAMPLE_RATE as f32) as Time;
            },
            AdsrMessage::SetSustain(sustain) => {
                self.sustain_level = sustain;
            },
            AdsrMessage::SetRelease(release) => {
                self.release_time = (release*SAMPLE_RATE as f32) as Time;
            },
        }
    }

    /// Triggers a state change and updates the corresponding state
    fn handle_state_change(&mut self, to: AdsrState, t: Time) {
        match to {
            AdsrState::Attack => {
                self.current_state = AdsrState::Attack;
                self.next_state_change = t + self.attack_time;
                self.multiplier_delta = (1.0 - self.multiplier) /
                    self.attack_time as f32;
            },
            AdsrState::Decay => {
                self.current_state = AdsrState::Decay;
                self.next_state_change = t + self.decay_time;
                self.multiplier_delta = (self.sustain_level - self.multiplier) /
                    self.decay_time as f32;
            },
            AdsrState::Sustain => {
                self.current_state = AdsrState::Sustain;
                self.next_state_change = 0;
                self.multiplier = self.sustain_level;
                self.multiplier_delta = 0.0;
            },
            AdsrState::Release => {
                self.current_state = AdsrState::Release;
                self.next_state_change = t + self.release_time;
                self.multiplier_delta = (0.0 - self.multiplier) /
                    self.release_time as f32;
            },
            AdsrState::Silent => {
                self.current_state = AdsrState::Silent;
                self.next_state_change = 0;
                self.multiplier = 0.0;
                self.multiplier_delta = 0.0;
            }
        }
    }
}

impl AudioDevice for Adsr {
    fn num_inputs(&self) -> usize {
        self.num_channels
    }

    fn num_outputs(&self) -> usize {
        self.num_channels
    }

    fn tick(&mut self, t: Time, inputs: &[Sample], outputs: &mut[Sample]) {
        // Handle any state changes
        if self.next_state_change == t {
            let next_state = self.current_state.next();
            self.handle_state_change(next_state, t);
        }
        self.last_time = t;

        // Update the multiplier
        self.multiplier += self.multiplier_delta;

        // Apply the envelope
        for (i,s) in inputs.iter().enumerate() {
            outputs[i] = s*self.multiplier;
        }
    }
}
