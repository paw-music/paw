use crate::{osc::clock::Clock, param::f32::UnitInterval};

use super::note::Note;

pub trait MidiEventListener {
    fn note_on(&mut self, clock: &Clock, note: Note, velocity: UnitInterval);
    fn note_off(&mut self, clock: &Clock, note: Note, velocity: UnitInterval);
}
