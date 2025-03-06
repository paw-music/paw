use core::f32::consts::TAU;

use lazy_static::lazy_static;

use crate::{
    synth::Synth,
    wavetable::{Wavetable, WavetableProps, WavetableRow},
};

use super::osc::WavetableOsc;

const WAVETABLE_DEPTH: usize = 4;
const WAVETABLE_LENGTH: usize = 1024;
const SAMPLE_RATE: u32 = 44_100;
const VOICES: usize = 8;
const LFOS: usize = 2;
const ENVS: usize = 2;
const OSCS: usize = 2;
pub type WavetableSynth = Synth<WavetableOsc<4, 1024>, 8, 2, 2, 3>;

// For test use only
// TODO: Remove
pub fn create_wavetable_synth(sample_rate: u32) -> WavetableSynth {
    lazy_static! {
        static ref BASIC_WAVES_TABLE: Wavetable<WAVETABLE_DEPTH, WAVETABLE_LENGTH> =
            Wavetable::from_rows([
                // Sine
                WavetableRow::new(|phase| (TAU * phase).sin()),
                // Square
                WavetableRow::new(|phase| if phase < 0.5 { 1.0 } else { -1.0 }),
                // Triangle
                WavetableRow::new(|phase| 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0),
                // Saw
                WavetableRow::new(|phase| 2.0 * (phase - (phase + 0.5).floor())),
            ]);
    }

    WavetableSynth::new(sample_rate, |index| {
        WavetableProps::new(index, &BASIC_WAVES_TABLE)
    })
}
