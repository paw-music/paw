#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub sample_rate: u32,
    // Note: u32 is enough for one day clocking at 48kHz
    pub tick: u32,
}

impl Clock {
    pub fn new(sample_rate: u32, tick: u32) -> Self {
        Self { sample_rate, tick }
    }

    pub fn phase(&self, freq: f32, last_cycle: &mut u32) -> f32 {
        let delta = self.tick - *last_cycle;

        let phase = delta as f32 * freq / self.sample_rate as f32;

        if phase > 1.0 {
            *last_cycle = self.tick;
        }

        phase.fract()
    }
}
