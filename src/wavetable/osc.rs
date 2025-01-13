use super::{Wavetable, WavetableRow};
use crate::{
    osc::Osc,
    sample::Sample,
    source::Source,
    value::{
        freq::Freq,
        time::{Phase, SampleDuration},
    },
};
use fixed::traits::ToFixed;
use num_traits::{real::Real, Float};

#[derive(Clone, Copy)]
pub struct WavetableOsc<
    'a,
    S: Sample,
    const SAMPLE_RATE: u32,
    const DEPTH: usize,
    const LENGTH: usize,
> {
    // TODO: Floating point depth with interpolation?
    depth: usize,
    wavetable: &'a Wavetable<S, DEPTH, LENGTH>,
    start_phase: f32,
    phase: f32,
    freq: f32,
}

impl<'a, S: Sample, const SAMPLE_RATE: u32, const DEPTH: usize, const LENGTH: usize>
    WavetableOsc<'a, S, SAMPLE_RATE, DEPTH, LENGTH>
{
    pub fn new(wavetable: &'a Wavetable<S, DEPTH, LENGTH>) -> Self {
        Self {
            depth: 0,
            wavetable,
            start_phase: 0.0,
            phase: 0.0,
            freq: 0.0,
            // start_phase: Phase::from_num(0.0),
            // phase: Phase::from_num(0.0),
            // sample_duration: SampleDuration::from_num(0.0),
        }
    }

    pub fn set_depth(&mut self, depth: usize) -> &mut Self {
        self.depth = depth;
        self
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn set_start_phase(&mut self, start_phase: f32) -> &mut Self {
        debug_assert!(
            start_phase >= 0.0 && start_phase <= 1.0,
            "Malformed start phase {start_phase}"
        );

        self.start_phase = start_phase;
        self
    }

    pub fn start_phase(&self) -> f32 {
        self.start_phase
    }

    pub fn wavetable(&self) -> &Wavetable<S, DEPTH, LENGTH> {
        self.wavetable
    }

    pub fn current_row(&self) -> &WavetableRow<S, LENGTH> {
        &self.wavetable.rows[self.depth]
    }
}

impl<'a, S: Sample, const SAMPLE_RATE: u32, const DEPTH: usize, const LENGTH: usize> Osc
    for WavetableOsc<'a, S, SAMPLE_RATE, DEPTH, LENGTH>
{
    fn freq(&self) -> f32 {
        self.freq
    }

    fn set_freq(&mut self, freq: f32) -> &mut Self {
        // self.sample_duration = (LENGTH.to_fixed::<SampleDuration>()
        //     * freq.to_fixed::<SampleDuration>())
        //     / SAMPLE_RATE.to_fixed::<SampleDuration>();

        debug_assert!(
            freq.is_finite() && freq >= 0.0,
            "Malformed frequency {freq}"
        );

        self.freq = freq;

        self
    }

    fn reset(&mut self) -> &mut Self {
        self.phase = self.start_phase;
        self
    }
}

impl<'a, S: Sample, const SAMPLE_RATE: u32, const DEPTH: usize, const LENGTH: usize> Iterator
    for WavetableOsc<'a, S, SAMPLE_RATE, DEPTH, LENGTH>
{
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.wavetable.at(self.depth, self.phase);

        self.phase = (self.phase + self.freq / SAMPLE_RATE as f32).fract();

        Some(sample)
    }
}

impl<'a, S: Sample, const SAMPLE_RATE: u32, const DEPTH: usize, const LENGTH: usize> Source
    for WavetableOsc<'a, S, SAMPLE_RATE, DEPTH, LENGTH>
{
}
