use crate::{osc::clock::Clock, sample::Frame};

pub trait Instrument {
    fn tick(&mut self, clock: &Clock) -> Frame;
}

pub struct ChannelRack<const SIZE: usize> {
    channels: [Option<Box<dyn Instrument>>; SIZE],
}
