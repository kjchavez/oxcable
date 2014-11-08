//! Defines AudioDevices for mixing signals

#![experimental]

pub use self::adder::Adder;
pub use self::gain::Gain;
pub use self::multiplier::Multiplier;
pub use self::multiplexer::Multiplexer;

pub mod adder;
pub mod gain;
pub mod multiplexer;
pub mod multiplier;
