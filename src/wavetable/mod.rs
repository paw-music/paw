use crate::modx::{ModValue, Modulate};
use micromath::F32Ext;
// use micromath::F32Ext;
use num_traits::float::FloatCore;

pub mod osc;
pub mod synth;

// /// The minimum amount of deviation from integer for lerp to be applied to neighbor samples
// pub const WAVETABLE_LERP_DELTA_THRESHOLD: f32 = 0.001;

pub const WAVETABLE_DEPTH_LERP_THRESHOLD: f32 = 0.001;

// /// Note: Static assertions are impossible in current stable to check LENGTH. LENGTH MUST BE a power of two for modulo optimization
#[derive(Debug)]
pub struct WavetableRow<const LENGTH: usize> {
    samples: [f32; LENGTH],
}

impl<const LENGTH: usize> WavetableRow<LENGTH> {
    const LENGTH_F: f32 = LENGTH as f32;

    pub fn new(f: impl Fn(f32) -> f32) -> Self {
        Self {
            samples: core::array::from_fn(|index| {
                let phase = index as f32 / Self::LENGTH_F;
                f(phase)
            }),
        }
    }

    // #[inline(always)]
    pub fn lerp(&self, phase: f32) -> f32 {
        // FIXME: phase of 1.0 * LENGTH is max size, but phase is never 1.0, will it happen?
        let index = phase * Self::LENGTH_F;
        let left_index = index as usize;
        // // Note: Modulo optimization, x % LENGTH == x % (LENGTH - 1) for LENGTH being a power of two
        // let right_index = (left_index + 1) & (LENGTH - 1);

        let right_index = (left_index + 1) % LENGTH;

        let right_index_factor = F32Ext::fract(index);
        let left_index_factor = 1.0 - right_index_factor;

        // if right_index_factor > WAVETABLE_LERP_DELTA_THRESHOLD {
        //     self.samples[right_index]
        // } else if left_index_factor > WAVETABLE_LERP_DELTA_THRESHOLD {
        //     self.samples[left_index]
        // } else {
        self.samples[left_index] * left_index_factor
            + self.samples[right_index] * right_index_factor
        // }
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
        // unsafe { self.rows.get_unchecked(depth).lerp(phase) }
        self.rows[depth % DEPTH].lerp(phase)
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
    osc_index: usize,
    wavetable: &'a Wavetable<DEPTH, LENGTH>,
    pub depth: usize,
    depth_lerp: Option<(f32, f32)>,
}

impl<'a, const DEPTH: usize, const LENGTH: usize> Modulate for WavetableProps<'a, DEPTH, LENGTH> {
    #[inline]
    fn modulated(
        &self,
        mut f: impl FnMut(crate::modx::mod_pack::ModTarget) -> Option<ModValue>,
    ) -> Self {
        if let Some(depth_mod) = f(crate::modx::mod_pack::ModTarget::OscWtPos(
            self.osc_index,
        )) {
            let depth_offset = depth_mod.as_sui();

            let depth = (Self::DEPTH_F + self.depth as f32 + Self::DEPTH_F * depth_offset.inner())
                % Self::DEPTH_F;
            let left_depth = depth as usize;

            let right_depth_factor = depth.fract();
            let left_depth_factor = 1.0 - right_depth_factor;

            Self {
                depth: left_depth,
                depth_lerp: Some((left_depth_factor, right_depth_factor)),
                ..*self
            }
        } else {
            *self
        }
    }
}

#[cfg(feature = "egui")]
impl<'a, const DEPTH: usize, const LENGTH: usize> crate::param::ui::EguiComponent
    for WavetableProps<'a, DEPTH, LENGTH>
{
    fn egui(&mut self, ui: &mut egui::Ui, _params: crate::param::ui::DefaultUiParams) {
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
    const DEPTH_F: f32 = DEPTH as f32;

    pub fn new(osc_index: usize, wavetable: &'a Wavetable<DEPTH, LENGTH>) -> Self {
        Self {
            osc_index,
            wavetable,
            depth: 0,
            depth_lerp: None,
        }
    }

    // #[inline(always)]
    pub fn lerp(&self, phase: f32) -> f32 {
        if let Some((left_depth_factor, right_depth_factor)) = self.depth_lerp {
            self.wavetable.at(self.depth, phase) * left_depth_factor
                + self.wavetable.at(self.depth + 1, phase) * right_depth_factor
        } else {
            self.wavetable.at(self.depth, phase)
        }
    }

    // pub fn with_depth_offset(&self, depth_offset: SignedUnitInterval) -> Self {
    //     Self {
    //         depth_offset,
    //         ..*self
    //     }
    // }
}
