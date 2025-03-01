use crate::{osc::clock::Clock, sample::Frame};

pub mod chorus;
pub mod delay;
pub mod dist;
pub mod filter;
pub mod fx_pack;

pub trait Fx {
    fn tick(&mut self, input: Frame, clock: &Clock) -> Frame;
}
