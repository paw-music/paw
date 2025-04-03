use crate::{
    modulation::{ModValue, Modulate},
    param::f32::SignedUnitInterval,
    sample::Sample,
};
use micromath::F32Ext as _;
use num::Zero;

pub mod osc;
pub mod synth;

// /// The minimum amount of deviation from integer for lerp to be applied to neighbor samples
// pub const WAVETABLE_LERP_DELTA_THRESHOLD: f32 = 0.001;

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
    #[inline]
    pub fn lerp(&self, phase: f32) -> f32 {
        // debug_assert!(phase >= 0.0 && phase < 1.0, "Malformed phase {phase}");

        // FIXME: phase of 1.0 * LENGTH is max size, but phase is never 1.0, will it happen?
        let left_index = (phase * LENGTH as f32) as usize % LENGTH;
        let right_index = (left_index + 1) % LENGTH;

        let right_index_factor = phase.fract();
        let left_index_factor = 1.0 - right_index_factor;

        // if right_index_factor > WAVETABLE_LERP_DELTA_THRESHOLD {
        //     self.samples[right_index]
        // } else if left_index_factor > WAVETABLE_LERP_DELTA_THRESHOLD {
        //     self.samples[left_index]
        // } else {
        self.samples[left_index].amp(left_index_factor)
            + self.samples[right_index].amp(right_index_factor)
        // }

        // self.samples[left_index]
        //     + phase.fract() * (self.samples[right_index] - self.samples[left_index])
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

    #[inline(always)]
    pub fn at(&self, depth: usize, phase: f32) -> f32 {
        // debug_assert!(phase >= 0.0 && phase < 1.0, "Malformed phase {phase}");

        let row = &self.rows[depth % DEPTH];

        row.lerp(phase)
    }
}

impl<const DEPTH: usize, const LENGTH: usize> Wavetable<DEPTH, LENGTH> {
    pub fn new(amp: impl Fn(usize, f32) -> f32) -> Self {
        let rows = core::array::from_fn(|depth| {
            let row = WavetableRow::new(|phase| amp(depth, phase));

            row
        });

        Self { rows }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WavetableProps<'a, const DEPTH: usize, const LENGTH: usize> {
    pub osc_index: usize,
    pub wavetable: &'a Wavetable<DEPTH, LENGTH>,
    pub depth: usize,
    pub depth_offset: SignedUnitInterval,
}

impl<'a, const DEPTH: usize, const LENGTH: usize> Modulate for WavetableProps<'a, DEPTH, LENGTH> {
    fn modulated(
        &self,
        mut f: impl FnMut(crate::modulation::mod_pack::ModTarget) -> ModValue,
    ) -> Self {
        let depth_offset = self.depth_offset
            + f(crate::modulation::mod_pack::ModTarget::OscWtPos(
                self.osc_index,
            ))
            .as_sui();

        Self {
            depth_offset,
            ..*self
        }
    }
}

#[cfg(feature = "egui")]
impl<'a, const DEPTH: usize, const LENGTH: usize> crate::param::ui::EguiComponent
    for WavetableProps<'a, DEPTH, LENGTH>
{
    fn egui(&mut self, ui: &mut egui::Ui, params: crate::param::ui::DefaultUiParams) {
        crate::param::ui::egui_wave(ui, |x| self.lerp(x));

        ui.add(
            egui::Slider::from_get_set(0.0..=DEPTH as f64 - 1.0, |new_value| {
                if let Some(new_value) = new_value {
                    self.depth = new_value as usize;
                }
                self.depth as f64
            })
            .text("Depth"),
        );
    }
}

impl<'a, const DEPTH: usize, const LENGTH: usize> WavetableProps<'a, DEPTH, LENGTH> {
    pub fn new(osc_index: usize, wavetable: &'a Wavetable<DEPTH, LENGTH>) -> Self {
        Self {
            osc_index,
            wavetable,
            depth: 0,
            depth_offset: SignedUnitInterval::EQUILIBRIUM,
        }
    }

    #[inline]
    pub fn modulated_depth(&self) -> f32 {
        let depth_offset = self.depth_offset.inner();
        if depth_offset.is_zero() {
            self.depth as f32
        } else {
            (DEPTH as f32 + self.depth as f32 + DEPTH as f32 * depth_offset) % DEPTH as f32
        }
    }

    #[inline]
    pub fn lerp(&self, phase: f32) -> f32 {
        let depth = self.modulated_depth();

        let left_depth = depth as usize;

        // Note: Integer depth optimization
        // TODO: Should be is_zero or approx with small number?
        if depth.fract().is_zero() {
            self.wavetable.at(left_depth, phase)
        } else {
            let right_depth = (left_depth + 1) % DEPTH;

            let right_depth_factor = depth.fract();
            let left_depth_factor = 1.0 - right_depth_factor;

            self.wavetable.at(left_depth, phase) * left_depth_factor
                + self.wavetable.at(right_depth, phase) * right_depth_factor
        }
    }

    // pub fn with_depth_offset(&self, depth_offset: SignedUnitInterval) -> Self {
    //     Self {
    //         depth_offset,
    //         ..*self
    //     }
    // }
}
