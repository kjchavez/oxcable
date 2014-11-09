//! Defines AudioDevices for dynamics processing

#![experimental]

pub use self::compressor::Compressor;
pub use self::limiter::Limiter;
pub use self::noise_gate::NoiseGate;

pub mod compressor;
pub mod level_detector;
pub mod limiter;
pub mod noise_gate;
