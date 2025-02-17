use super::Wavetable;
use crate::{
    osc::Osc,
    param::{f32::SignedUnitInterval, ui::UiComponent},
    sample::Sample,
};

#[derive(Debug, Clone, Copy)]
pub struct WavetableParams<'a, const DEPTH: usize, const LENGTH: usize> {
    pub wavetable: &'a Wavetable<DEPTH, LENGTH>,
    // TODO: Floating point depth with interpolation?
    pub depth: usize,
    pub depth_offset: SignedUnitInterval,
}

impl<'a, const DEPTH: usize, const LENGTH: usize> UiComponent
    for WavetableParams<'a, DEPTH, LENGTH>
{
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        _params: &crate::param::ui::UiParams,
    ) {
        ui.wave(|x| {
            // self.wavetable.rows[self.row_index()].samples[(x * LENGTH as f32) as usize].to_f32()
            self.lerp(x)
        });

        ui.int_map("Depth", (0, DEPTH - 1), |new_depth| {
            if let Some(new_depth) = new_depth {
                self.depth = new_depth
            }

            self.depth
        });
    }
}

impl<'a, const DEPTH: usize, const LENGTH: usize> WavetableParams<'a, DEPTH, LENGTH> {
    pub fn new(wavetable: &'a Wavetable<DEPTH, LENGTH>) -> Self {
        Self {
            wavetable,
            depth: 0,
            depth_offset: SignedUnitInterval::EQUILIBRIUM,
        }
    }

    pub fn modulated_depth(&self) -> f32 {
        (DEPTH as f32 + self.depth as f32 + DEPTH as f32 * self.depth_offset.inner()) % DEPTH as f32
    }

    pub fn lerp(&self, phase: f32) -> f32 {
        let depth = self.modulated_depth();

        let left_depth = depth as usize;
        let right_depth = (left_depth + 1) % DEPTH;

        let right_depth_factor = depth.fract();
        let left_depth_factor = 1.0 - right_depth_factor;

        self.wavetable.at(left_depth, phase) * left_depth_factor
            + self.wavetable.at(right_depth, phase) * right_depth_factor
    }

    pub fn with_depth_offset(&self, depth_offset: SignedUnitInterval) -> Self {
        Self {
            depth: self.depth,
            wavetable: self.wavetable,
            depth_offset,
        }
    }
}

// TODO: Wavetable, depth and other wavetable-related params should be passed by reference. It will allow us to make global for synth wavetable params without mutability issues. Do it as for LFO and Env: WavetableParams are passed to [`WavetableSource`] tick and [`WavetableSource`] only contains sampling-related states: frequency, phase.
// The open question is if depth can possibly be non-global for voices such that some modulation controls spread of the depths.
#[derive(Clone, Copy)]
pub struct WavetableOsc<const DEPTH: usize, const LENGTH: usize> {
    // TODO: Move to voice, and use
    // start_phase: f32,
}

impl<const DEPTH: usize, const LENGTH: usize> Osc for WavetableOsc<DEPTH, LENGTH> {
    type Props<'a> = WavetableParams<'a, DEPTH, LENGTH>;

    fn tick<'a>(&mut self, phase: f32, params: &Self::Props<'a>) -> SignedUnitInterval {
        let sample = params.lerp(phase);

        // self.phase = (self.phase + self.freq / params. as f32).fract();

        SignedUnitInterval::new_checked(sample)
    }

    // fn reset(&mut self) -> &mut Self {
    //     todo!()
    // }
}

impl<const DEPTH: usize, const LENGTH: usize> WavetableOsc<DEPTH, LENGTH> {
    pub fn new() -> Self {
        Self {}
    }

    // pub fn set_start_phase(&mut self, start_phase: f32) -> &mut Self {
    //     debug_assert!(
    //         start_phase >= 0.0 && start_phase <= 1.0,
    //         "Malformed start phase {start_phase}"
    //     );

    //     self.start_phase = start_phase;
    //     self
    // }

    // pub fn start_phase(&self) -> f32 {
    //     self.start_phase
    // }
}

// impl< const DEPTH: usize, const LENGTH: usize> Osc
//     for WavetableOsc< DEPTH, LENGTH>
// {
//     type Params<'a> = WavetableParams<'a,  DEPTH, LENGTH>;

//     fn freq(&self) -> Freq {
//         self.freq
//     }

//     fn set_freq(&mut self, freq: Freq) -> &mut Self {
//         // self.sample_duration = (LENGTH.to_fixed::<SampleDuration>()
//         //     * freq.to_fixed::<SampleDuration>())
//         //     / SAMPLE_RATE.to_fixed::<SampleDuration>();

//         debug_assert!(freq >= 0.0, "Malformed frequency {freq}");

//         self.freq = freq;

//         self
//     }

//     fn reset(&mut self) -> &mut Self {
//         self.phase = self.start_phase;
//         self
//     }

//     fn tick<'a>(&mut self, params: &Self::Params<'a>) -> Option<Self::Output> {
//         let sample = params.lerp(self.phase);

//         self.phase = (self.phase + self.freq.to_num::<f32>() / SAMPLE_RATE as f32).fract();

//         Some(sample)
//     }
// }
