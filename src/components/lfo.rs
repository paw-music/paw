use crate::{
    param::{f32::UnitInterval, ui::UiComponent},
    value::freq::Freq,
};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LfoWaveform {
    /// Pulse waveform with specific pulse width
    Pulse(UnitInterval),
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

impl<const SAMPLE_RATE: u32> UiComponent for Lfo<SAMPLE_RATE> {
    fn ui(&mut self, ui: &mut impl crate::param::ui::ParamUi) {
        ui.freq(
            "Frequency",
            &mut self.freq,
            Some((Freq::from_num(0.01), Freq::from_num(100.0))),
        );
        ui.select(
            "Waveform",
            &mut self.waveform,
            &[
                ("Pulse", LfoWaveform::Pulse(UnitInterval::new(0.0))),
                ("Sine", LfoWaveform::Sine),
                ("Triangle", LfoWaveform::Triangle),
                ("Saw", LfoWaveform::Saw),
                ("Reverse saw", LfoWaveform::ReverseSaw),
            ],
        );

        if let LfoWaveform::Pulse(pulse_width) = &mut self.waveform {
            ui.unit_interval("Pulse width", pulse_width);
        }
    }
}

impl<const SAMPLE_RATE: u32> Lfo<SAMPLE_RATE> {
    pub fn new() -> Self {
        Self {
            freq: Freq::ZERO,
            waveform: LfoWaveform::default(),
        }
    }
}
