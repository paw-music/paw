use crate::{param::Param, value::freq::Freq};

#[derive(Debug, Default)]
pub enum LfoWaveform {
    /// Pulse waveform with specific pulse width
    Pulse(u32),
    #[default]
    Sine,
    Triangle,
    Saw,
    ReverseSaw,
}

pub enum LfoTrigger {
    /// LFO restarts on each note
    Trigger,
    /// LFO acts like an envelope running once on each note
    Envelope,
    /// LFO does not retrigger and keeps running in a loop
    Loop,
}

// TODO: Delay and rise
// TODO: Set freq by rate (1/4, 1/2, etc.). Need synth BPM for that
pub struct Lfo<const SAMPLE_RATE: u32> {
    freq: Freq,
    waveform: LfoWaveform,
}

impl<const SAMPLE_RATE: u32> Lfo<SAMPLE_RATE> {
    pub fn new() -> Self {
        Self {
            freq: Freq::ZERO,
            waveform: LfoWaveform::default(),
        }
    }

    pub fn with_params(&mut self, mut f: impl FnMut(Param)) {
        // TODO: Rate
        f(Param::new("Frequency", &mut self.freq));
    }
}
