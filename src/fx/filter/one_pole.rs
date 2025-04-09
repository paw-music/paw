use micromath::F32Ext as _;

#[derive(Clone, Copy)]
pub struct OnePole {
    a0: f32,
    b1: f32,
    z1: f32,
}

impl OnePole {
    pub fn new() -> OnePole {
        OnePole {
            a0: 0.0,
            b1: 0.0,
            z1: 0.0,
        }
    }

    pub fn set_cutoff(&mut self, cutoff: f32, sample_rate: u32) {
        let freq = cutoff / sample_rate as f32;
        self.b1 = (-2.0 * core::f32::consts::PI * freq).exp();
        self.a0 = 1.0 - self.b1;
    }

    pub fn process(&mut self, sample: f32) -> f32 {
        self.z1 = sample * self.a0 + self.z1 * self.b1;
        self.z1
    }
}
