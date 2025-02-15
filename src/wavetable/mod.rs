use crate::sample::Sample;

pub mod osc;
pub mod synth;

#[derive(Debug)]
pub struct WavetableRow<const LENGTH: usize> {
    samples: [f32; LENGTH],
}

impl<const LENGTH: usize> WavetableRow<LENGTH> {
    pub fn new(f: impl Fn(f32) -> f32) -> Self {
        Self {
            samples: core::array::from_fn(|index| {
                let phase = index as f32 / LENGTH as f32;
                f(phase)
            }),
        }
    }
}

impl<const LENGTH: usize> WavetableRow<LENGTH> {
    pub fn lerp(&self, phase: f32) -> f32 {
        debug_assert!(phase >= 0.0 && phase <= 1.0, "Malformed phase {phase}");

        // FIXME: phase of 1.0 * LENGTH is max size
        let left_index = (phase * LENGTH as f32) as usize % LENGTH;
        let right_index = (left_index + 1) % LENGTH;

        let right_index_factor = phase.fract();
        let left_index_factor = 1.0 - right_index_factor;

        self.samples[left_index].amp(left_index_factor)
            + self.samples[right_index].amp(right_index_factor)
    }
}

#[derive(Debug)]
pub struct Wavetable<const DEPTH: usize, const LENGTH: usize> {
    rows: [WavetableRow<LENGTH>; DEPTH],
}

impl<const DEPTH: usize, const LENGTH: usize> Wavetable<DEPTH, LENGTH> {
    pub fn from_rows(rows: [WavetableRow<LENGTH>; DEPTH]) -> Self {
        Self { rows }
    }

    // TODO: f32 depth so we interpolate in depth too?
    pub fn at(&self, depth: usize, phase: f32) -> f32 {
        debug_assert!(phase >= 0.0 && phase <= 1.0, "Malformed phase {phase}");

        let row = &self.rows[depth % DEPTH];

        row.lerp(phase)
    }
}

impl<const DEPTH: usize, const LENGTH: usize> Wavetable<DEPTH, LENGTH> {
    pub fn new(amp: impl Fn(usize, f32) -> f32) -> Self {
        // let pitch_step = (max_pitch - min_pitch) / DEPTH.to_fixed::<Freq>();
        let rows = core::array::from_fn(|depth| {
            // let freq = pitch_step * depth.to_fixed::<Freq>();
            // let wave = WavetableRow::new(core::array::from_fn(|index| {
            //     amp(depth, (index as f32) / (LENGTH as f32))
            // }));

            let row = WavetableRow::new(|phase| amp(depth, phase));

            row
        });

        Self { rows }
    }

    // pub fn equal_freq(pitch: Freq, amp: impl Fn(usize, f32) -> f32) -> Self {
    //     Self::new(pitch, pitch, amp)
    // }
}
