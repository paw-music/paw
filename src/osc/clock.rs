use core::{fmt::Display, ops::Mul};
use num_traits::float::FloatCore;
// use micromath::F32Ext as _;

// Note: u32 is enough for one day clocking at 48kHz
pub type Tick = u32;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub sample_rate: u32,
    pub tick: Tick,
}

impl Clock {
    #[inline]
    pub fn zero(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            tick: 0,
        }
    }

    #[inline(always)]
    pub fn tick(&mut self) {
        self.tick += 1;
    }

    // #[inline(always)]
    // pub fn sub_tick(self, offset: Tick) -> Self {
    //     self.with_tick(self.tick + offset)
    // }

    /// Ticking method for processing a buffer. Does not increment clock counter but gives valid tick iterator starting from current clock state.
    #[inline(always)]
    pub fn for_buffer(self, buffer_len: usize) -> impl Iterator<Item = Self> {
        let buffer_len = buffer_len as Tick;
        (0..buffer_len).map(move |offset| self.with_tick(self.tick + offset))
    }

    /// Advances counter by buffer size. Must only be called when the whole buffer is processed by the system (in DAW).
    #[inline(always)]
    pub fn tick_for_buffer(&mut self, buffer_len: Tick) {
        self.tick += buffer_len;
    }

    #[inline(always)]
    pub fn set(&mut self, tick: Tick) {
        self.tick = tick;
    }

    #[inline(always)]
    pub fn with_tick(self, tick: Tick) -> Self {
        Self {
            tick,
            sample_rate: self.sample_rate,
        }
    }

    #[inline(always)]
    pub fn phase_fast(&self, phase_step: f32, last_sync: &mut Tick) -> f32 {
        let delta = self.tick.saturating_sub(*last_sync);

        let phase = delta as f32 * phase_step;

        if (1.0 - phase) <= phase_step {
            *last_sync = self.tick;
            phase.fract()
        } else {
            phase
        }
    }

    #[inline(always)]
    pub fn phase(&self, freq: Freq, last_sync: &mut Tick) -> f32 {
        let delta = self.tick.saturating_sub(*last_sync);

        let phase = delta as f32 * freq.inner() / self.sample_rate as f32;

        if (1.0 - phase) <= 1.0 / freq.inner() {
            *last_sync = self.tick;
        }

        phase.fract()
    }
}

// TODO: Get rid of Freq wrapper. Create FreqExt containing frequency-related methods for f32
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Freq(f32);

impl Display for Freq {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl PartialEq for Freq {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < 0.01
    }
}

impl Mul<f32> for Freq {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.inner() * rhs)
    }
}

impl Freq {
    pub const ZERO: Self = Self::new(0.0);
    pub const HZ: Self = Self::new(1.0);
    pub const MILLI_HZ: Self = Self::new(0.001);
    pub const KHZ: Self = Self::new(1_000.0);
    pub const MHZ: Self = Self::new(1_000_000.0);

    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value)
    }

    #[allow(non_snake_case)]
    #[inline]
    pub const fn mHz(value: u32) -> Self {
        Self(value as f32 / 1_000.0)
    }

    #[allow(non_snake_case)]
    #[inline]
    pub const fn Hz(value: u32) -> Self {
        Self(value as f32)
    }

    #[allow(non_snake_case)]
    #[inline]
    pub const fn kHz(value: u32) -> Self {
        Self(value as f32 * 1_000.0)
    }

    #[allow(non_snake_case)]
    #[inline]
    pub const fn MHz(value: u32) -> Self {
        Self(value as f32 * 1_000_000.0)
    }

    #[inline]
    pub const fn inner(&self) -> f32 {
        self.0
    }

    #[cfg(feature = "egui")]
    pub fn widget(&mut self, clamp: Option<core::ops::RangeInclusive<Freq>>) -> egui::Slider {
        let range = clamp
            .map(|clamp| clamp.start().inner() as f64..=clamp.end().inner() as f64)
            .unwrap_or_else(|| 0.01..=20_000.0);

        let logarithmic = range.end() - range.start() >= 1_000.0;

        egui::Slider::from_get_set(range, |new_value| {
            if let Some(new_value) = new_value {
                *self = Self::new(new_value as f32);
            }

            self.inner() as f64
        })
        .logarithmic(logarithmic)
        .custom_formatter(|value, _| {
            let khz = value / 1_000.0;

            if value == 0.0 {
                format!("0Hz")
            } else if khz < 1.0 {
                format!("{value:.2}Hz")
            } else {
                format!("{khz:.2}kHz")
            }
        })
    }
}

// impl Add for Freq {
//     type Output = Self;

//     fn add(self, rhs: Self) -> Self::Output {
//         todo!()
//     }
// }

impl From<f32> for Freq {
    #[inline]
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Into<f32> for Freq {
    #[inline]
    fn into(self) -> f32 {
        self.0
    }
}
