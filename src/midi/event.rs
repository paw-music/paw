use crate::param::f32::UnitInterval;

use super::note::Note;

pub trait MidiEventListener {
    fn note_on(&mut self, note: Note, velocity: UnitInterval);
    fn note_off(&mut self, note: Note, velocity: UnitInterval);
}
