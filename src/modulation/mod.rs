use crate::param::f32::{SignedUnitInterval, UnitInterval};
use core::f32;
use mod_pack::ModTarget;

pub mod env;
pub mod lfo;
pub mod mod_pack;

pub trait Modulate {
    fn modulated(&self, f: impl FnMut(ModTarget) -> Option<SignedUnitInterval>) -> Self;
}

pub fn fm(freq: f32, m: f32) -> f32 {
    freq * m.exp2()
}

pub fn am(output: SignedUnitInterval, m: UnitInterval) -> SignedUnitInterval {
    output * m.inner().powf(f32::consts::E)
}

pub fn rm(output: SignedUnitInterval, m: SignedUnitInterval) -> SignedUnitInterval {
    output * m.inner()
}
