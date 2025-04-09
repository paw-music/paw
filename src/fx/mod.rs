use crate::{
    midi::event::MidiEventListener,
    osc::clock::{Clock, Tick},
    sample::Frame,
};

pub mod chorus;
pub mod delay;
pub mod dist;
pub mod filter;

pub trait Fx: MidiEventListener + Send {
    fn tick(&mut self, clock: &Clock, input: Frame) -> Frame;
    fn name(&self) -> &str;

    #[inline]
    fn process_buffer(&mut self, clock: &Clock, buffer: &mut [Frame]) {
        buffer.iter_mut().enumerate().for_each(|(offset, frame)| {
            *frame = self.tick(&clock.sub_tick(offset as Tick), *frame)
        });
    }

    #[cfg(feature = "egui")]
    fn egui(&mut self, ui: &mut egui::Ui, params: (Clock,));
}
