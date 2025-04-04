use core::{f32::EPSILON, ops::Mul};
use micromath::F32Ext as _;

#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub sample_rate: u32,
    // Note: u32 is enough for one day clocking at 48kHz
    pub tick: u32,
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

    #[inline(always)]
    pub fn set(&mut self, tick: u32) {
        self.tick = tick;
    }

    #[inline(always)]
    pub fn with_tick(self, tick: u32) -> Self {
        Self { tick, ..self }
    }

    #[inline(always)]
    pub fn phase(&self, freq: Freq, last_sync: &mut u32) -> f32 {
        let delta = self.tick - *last_sync;

        let phase = delta as f32 * freq.inner() / self.sample_rate as f32;

        if (1.0 - phase) <= 1.0 / freq.inner() {
            *last_sync = self.tick;
        }

        phase.fract()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Freq(f32);

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
    fn from(value: f32) -> Self {
        Self(value)
    }
}

impl Into<f32> for Freq {
    fn into(self) -> f32 {
        self.0
    }
}
