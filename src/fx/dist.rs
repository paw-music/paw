use crate::{
    param::f32::{SignedUnitInterval, UnitInterval},
    sample::Frame,
};

pub enum DistKind {
    HardClip,
    SoftClip,
    Exp,
    HalfWaveRect,
}

pub struct DistParams {
    pub kind: DistKind,
    pub input: UnitInterval,
}

// TODO: Filters
pub struct Dist {}

impl Dist {
    pub fn new() -> Self {
        Self {}
    }

    pub fn tick(&mut self, input: Frame, params: &DistParams) -> Frame {
        let input = input.map(|input| *input * params.input.inner());

        let output = match params.kind {
            DistKind::HardClip => {
                const THRESHOLD: f32 = 0.5;

                input.map(|&input| {
                    let input = input;
                    if input > THRESHOLD {
                        THRESHOLD
                    } else if input < -THRESHOLD {
                        -THRESHOLD
                    } else {
                        input
                    }
                })
            }
            DistKind::SoftClip => {
                const THRESHOLD1: f32 = 1.0 / 3.0;
                const THRESHOLD2: f32 = 2.0 / 3.0;

                input.map(|&input| {
                    0.5 * if input > THRESHOLD2 {
                        1.0
                    } else if input > THRESHOLD1 {
                        1.0 - (2.0 - 3.0 * input).powf(2.0) / 3.0
                    } else if input < -THRESHOLD2 {
                        -1.0
                    } else if input < -THRESHOLD1 {
                        -1.0 + (2.0 + 3.0 * input).powf(2.0) / 3.0
                    } else {
                        2.0 * input
                    }
                })
            }
            DistKind::Exp => input.map(|&input| {
                if input > 0.0 {
                    1.0 - (-input).exp()
                } else {
                    -1.0 + input.exp()
                }
            }),
            DistKind::HalfWaveRect => input.map(|&input| if input > 0.0 { input } else { 0.0 }),
        };

        output
    }
}
