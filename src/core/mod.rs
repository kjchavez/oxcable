//! Core definitions, components, and functions for creating audio devices.

#![experimental]

pub type Sample = f64;
pub type Time   = u64;

pub mod complex;
pub mod fft;
