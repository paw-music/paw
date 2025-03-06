use crate::{midi::event::MidiEventListener, osc::clock::Clock, sample::Frame};

pub mod chorus;
pub mod delay;
pub mod dist;
pub mod filter;

pub trait Fx: MidiEventListener + Send {
    fn tick(&mut self, clock: &Clock, input: Frame) -> Frame;
    fn name(&self) -> &str;

    #[cfg(feature = "egui")]
    fn egui(&mut self, ui: &mut egui::Ui, params: (Clock,));
}
