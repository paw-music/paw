use crate::{
    osc::clock::Freq,
    param::f32::{SignedUnitInterval, UnitInterval},
};
use core::f32;
use mod_pack::ModTarget;

pub mod env;
pub mod lfo;
pub mod mod_pack;

pub trait Modulate {
    fn modulated(&self, f: impl FnMut(ModTarget) -> ModValue) -> Self;
}

pub fn fm(freq: Freq, m: f32) -> Freq {
    freq * m.exp2()
}

pub fn am(output: f32, m: f32) -> f32 {
    output * m.powf(f32::consts::E)
}

pub fn rm(output: f32, m: f32) -> f32 {
    output * m
}

/// Modulation with known source, this is now only used to distinguish envelope modulation from others, because envelope modulation is defining whereas others are additive or multiplying
#[derive(Default, Clone, Copy, Debug)]
pub enum ModValue {
    #[default]
    None,
    Env(UnitInterval),
    Lfo(SignedUnitInterval),
    // // Generic UnitInterval
    // UnitInterval(UnitInterval),
}

impl ModValue {
    pub fn as_ui(&self) -> UnitInterval {
        match self {
            ModValue::None => UnitInterval::MIN,
            ModValue::Env(env) => *env,
            ModValue::Lfo(lfo) => lfo.remap_into_ui(),
        }
    }

    pub fn as_sui(&self) -> SignedUnitInterval {
        match self {
            ModValue::None => SignedUnitInterval::MIN,
            ModValue::Env(env) => env.remap_into_signed(),
            ModValue::Lfo(lfo) => *lfo,
        }
    }

    // pub fn or(self, other: Self)

    pub const fn env(value: Option<UnitInterval>) -> Self {
        if let Some(value) = value {
            Self::Env(value)
        } else {
            Self::None
        }
    }

    pub const fn lfo(value: Option<SignedUnitInterval>) -> Self {
        if let Some(value) = value {
            Self::Lfo(value)
        } else {
            Self::None
        }
    }
}

// impl ModValue {
//     pub fn modulate<T: Into<f32> + From<f32>>(
//         &self,
//         value: T,
//         velocity: Option<UnitInterval>,
//     ) -> T {
//         match self {
//             ModValue::None => value,
//             ModValue::Env(ui) => T::from(value.into() * ui.inner()),
//             ModValue::Lfo(sui) => {
//                 T::from(value.into() * sui.inner())
//             },
//         }
//     }
// }
