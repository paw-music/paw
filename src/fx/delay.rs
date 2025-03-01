use super::filter::one_pole::OnePole;
use crate::{
    osc::clock::Clock,
    param::f32::{SignedUnitInterval, UnitInterval},
    sample::{time::SampleCount, Frame},
};

#[derive(Debug, Clone, Copy)]
pub enum DelayKind {
    PingPong,
    Stereo,
}

#[derive(Debug, Clone, Copy)]
pub struct DelayParams {
    // TODO: Dry/wet instead of amount
    pub amount: UnitInterval,
    pub feedback: UnitInterval,
    pub time: SampleCount,
    pub kind: DelayKind,
    // TODO: Tone
}

pub struct Delay<const SIZE: usize> {
    bb: Frame<[f32; SIZE]>,
    flt: Frame<OnePole>,
}

impl<const SIZE: usize> Delay<SIZE> {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            bb: Frame::from_fn(|_| [0.0; SIZE]),
            flt: Frame::from_fn(|_| {
                let mut flt = OnePole::new();
                flt.set_cutoff(2_000.0, sample_rate);
                flt
            }),
        }
    }

    pub fn tick(&mut self, clock: &Clock, input: Frame, params: &DelayParams) -> Frame {
        let time = (params.time.inner() as usize).clamp(0, SIZE);

        let index = clock.tick as usize % time;

        // TODO: Lerp index

        // Read feedback
        let feedback = self.bb.at(index);

        // Write new feedback
        let write = input
            .zip(feedback, |&input, &feedback| {
                (input.inner() + feedback) * params.feedback.inner()
            })
            .zip_mut(&mut self.flt, |feedback, flt| flt.process(*feedback));

        let write = match params.kind {
            DelayKind::PingPong => write.swapped(),
            DelayKind::Stereo => write,
        };

        self.bb.set(index, write);

        let mix = input + feedback.map(|val| SignedUnitInterval::new(val * params.amount.inner()));

        mix
    }
}
