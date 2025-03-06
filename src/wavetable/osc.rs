use super::WavetableProps;
use crate::{osc::Osc, sample::Sample};

// TODO: The open question is if depth can possibly be non-global for voices such that some modulation controls spread of the depths.
// TODO: Per-voice depth so different unison voices can play different WT?
// TODO: Local wavetable modulation (per-voice)

#[derive(Clone, Copy)]
pub struct WavetableOsc<const DEPTH: usize, const LENGTH: usize> {
    // TODO: Move to voice, and use
    // start_phase: f32,
}

impl<const DEPTH: usize, const LENGTH: usize> Default for WavetableOsc<DEPTH, LENGTH> {
    fn default() -> Self {
        Self {}
    }
}

impl<const DEPTH: usize, const LENGTH: usize> Osc for WavetableOsc<DEPTH, LENGTH> {
    type Props<'a> = WavetableProps<'a, DEPTH, LENGTH>;

    fn tick<'a>(&mut self, phase: f32, params: &Self::Props<'a>) -> f32 {
        let sample = params.lerp(phase);

        sample
    }
}
