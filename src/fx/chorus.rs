use crate::sample::Frame;

pub struct ChorusParams {
    voices: u8,
}

pub struct Chorus {
    index: usize,
}

impl Chorus {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn tick(&mut self, input: Frame, params: &ChorusParams) -> Frame {
        // for voice in 0..params.voices {

        // }
        todo!()
    }
}
