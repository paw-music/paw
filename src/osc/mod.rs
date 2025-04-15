use crate::{
    midi::event::MidiEventListener,
    modx::{am, fm, rm, ModValue, Modulate},
    param::f32::UnitInterval,
    sample::Frame,
};
use clock::{Clock, Freq, Tick};

pub mod clock;

pub trait Osc: Sized + Default + Send {
    type Props<'a>: Copy + Modulate + Send;

    fn tick<'a>(&mut self, phase: f32, params: &Self::Props<'a>) -> f32;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OscOutput {
    Direct,
    FMNext,
    AMNext,
    RMNext,
    // TODO: PM? Phase modulation
}

#[derive(Debug, Clone, Copy)]
pub enum OscMod {
    None,
    Direct(f32),
    FM(f32),
    AM(f32),
    RM(f32),
}

/// The properties of oscillator component. Global for all oscillator instances.
pub struct OpProps<'a, O: Osc, const OSCS: usize> {
    index: usize,
    enabled: bool,
    osc: O::Props<'a>,
    output: OscOutput,
    // TODO: Mix (better balanced between oscs)
    // TODO: Tuning
    tune_semitones: i8,
    tune_cents: i8,
}

impl<'a, O: Osc, const OSCS: usize> Copy for OpProps<'a, O, OSCS> {}

impl<'a, O: Osc, const OSCS: usize> Clone for OpProps<'a, O, OSCS> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            enabled: self.enabled.clone(),
            osc: self.osc.clone(),
            output: self.output.clone(),
            tune_semitones: self.tune_semitones.clone(),
            tune_cents: self.tune_cents.clone(),
        }
    }
}

#[cfg(feature = "egui")]
impl<'a, O: Osc, const OSCS: usize> crate::param::ui::EguiComponent for OpProps<'a, O, OSCS> {
    fn egui(&mut self, ui: &mut egui::Ui, _params: crate::param::ui::DefaultUiParams) {
        ui.checkbox(&mut self.enabled, &format!("OSC{} enabled", self.index));

        if !self.enabled {
            return;
        }

        // self.kind.

        let next_osc = self.index + 1;
        if OSCS > next_osc {
            ui.radio_value(&mut self.output, OscOutput::Direct, "Direct output");
            ui.radio_value(
                &mut self.output,
                OscOutput::AMNext,
                format!("OSC{next_osc} AM"),
            );
            ui.radio_value(
                &mut self.output,
                OscOutput::FMNext,
                format!("OSC{next_osc} FM"),
            );
            ui.radio_value(
                &mut self.output,
                OscOutput::RMNext,
                format!("OSC{next_osc} RM"),
            );
        }

        ui.add(
            egui::Slider::from_get_set(-36.0..=36.0, |new_value| {
                if let Some(new_value) = new_value {
                    self.tune_semitones = new_value as i8;
                }

                self.tune_semitones as f64
            })
            .text("Tune semi"),
        );

        ui.add(
            egui::Slider::from_get_set(-50.0..=50.0, |new_value| {
                if let Some(new_value) = new_value {
                    self.tune_cents = new_value as i8;
                }

                self.tune_cents as f64
            })
            .text("Tune cents"),
        );
    }
}

impl<'a, O: Osc, const OSCS: usize> Modulate for OpProps<'a, O, OSCS> {
    #[inline]
    fn modulated(
        &self,
        f: impl FnMut(crate::modx::mod_pack::ModTarget) -> Option<ModValue>,
    ) -> Self {
        Self {
            osc: self.osc.modulated(f),
            ..*self
        }
    }
}

impl<'a, O: Osc, const OSCS: usize> OpProps<'a, O, OSCS> {
    pub fn new(index: usize, osc: O::Props<'a>) -> Self {
        Self {
            index,
            enabled: index == 0,
            osc,
            output: OscOutput::Direct,
            tune_semitones: 0,
            tune_cents: 0,
        }
    }

    #[inline]
    pub fn kind_mut(&mut self) -> &mut O::Props<'a> {
        &mut self.osc
    }
}

#[derive(Clone)]
pub struct OpParams<'a, O: Osc, const OSCS: usize> {
    pub props: OpProps<'a, O, OSCS>,
    pub pitch_mod: Option<ModValue>,
}

impl<'a, O: Osc, const OSCS: usize> OpParams<'a, O, OSCS> {
    #[inline]
    fn tune_mod(&self) -> f32 {
        self.pitch_mod
            .map(|pitch_mod| pitch_mod.as_sui().inner())
            .unwrap_or(0.0)
            + self.props.tune_semitones as f32 / 12.0
            + self.props.tune_cents as f32 / 1200.0
    }
}

// The state of a single oscillator
#[derive(Debug, Clone, Copy)]
pub struct OpState {
    last_cycle: Tick,
    last_freq: Freq,
    // min_phase_step: f32,
    phase_step: f32,
}

impl OpState {
    // #[inline(always)]
    fn update(&mut self, clock: &Clock, freq: Freq) {
        // TODO: float comparison?!
        if self.last_freq != freq {
            self.last_freq = freq;
            self.phase_step = freq.inner() / clock.sample_rate as f32;
        }
    }
}

// TODO: Pitch + Fine pitch, Syncing between oscillators (now synced through reset call), osc mix
#[derive(Clone)]
pub struct OperatorPack<O: Osc, const OSCS: usize> {
    oscs: [O; OSCS],
    states: [OpState; OSCS],
}

impl<O: Osc, const OSCS: usize> MidiEventListener for OperatorPack<O, OSCS> {
    #[inline]
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = note;
        let _ = velocity;
        self.states
            .iter_mut()
            .for_each(|state| state.last_cycle = 0);
    }

    #[inline]
    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = note;
        let _ = velocity;
    }
}

impl<O: Osc + 'static, const OSCS: usize> OperatorPack<O, OSCS> {
    #[inline]
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            oscs: core::array::from_fn(osc),
            states: core::array::from_fn(|_| OpState {
                last_cycle: 0,
                last_freq: Freq::ZERO,
                phase_step: 0.0,
                // freq: 0.0,
            }),
        }
    }

    // Note: Don't inline
    pub fn tick<'a>(&mut self, clock: &Clock, freq: Freq, params: &[OpParams<'a, O, OSCS>]) -> f32 {
        self.oscs
            .iter_mut()
            .zip(params)
            .zip(self.states.iter_mut())
            .fold(
                (OscMod::Direct(0.0), 0.0),
                |(modulation, mix), ((osc, params), state)| {
                    if !params.props.enabled {
                        // TODO: Why so? Modulation from previous enabled oscillator should be passed?
                        return (OscMod::None, mix);
                    }

                    let osc_fm = match modulation {
                        OscMod::FM(m) => m,
                        _ => 0.0,
                    };

                    let freq = fm(freq, params.tune_mod() + osc_fm);

                    state.update(clock, freq);
                    let phase = clock.phase_fast(state.phase_step, &mut state.last_cycle);

                    let output = osc.tick(phase, &params.props.osc);

                    let output = match modulation {
                        OscMod::AM(m) => am(output, m),
                        OscMod::RM(m) => rm(output, m),
                        _ => output,
                    };

                    let mix = if let OscOutput::Direct = params.props.output {
                        // Direct output mixes with other outputs
                        mix + output
                    } else {
                        // Non-direct outputs are used as modulation sources
                        mix
                    };

                    (
                        match params.props.output {
                            OscOutput::Direct => OscMod::Direct(output),
                            OscOutput::FMNext => OscMod::FM(output),
                            OscOutput::AMNext => OscMod::AM(output),
                            OscOutput::RMNext => OscMod::RM(output),
                        },
                        mix,
                    )
                },
            )
            .1
    }
}
