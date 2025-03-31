#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(test)]
// extern crate std;

#[macro_use]
extern crate num_derive;

#[macro_use]
extern crate alloc;

pub mod buffer;
pub mod daw;
pub mod fx;
pub mod macros;
pub mod midi;
pub mod modulation;
pub mod osc;
pub mod param;
pub mod sample;
pub mod synth;
pub mod voice;
pub mod wavetable;
