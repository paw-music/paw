use crate::{
    midi::event::MidiEventListener,
    modulation::{am, fm, rm, ModValue, Modulate},
    param::f32::UnitInterval,
};
use clock::{Clock, Freq};

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
#[derive(Clone, Copy)]
pub struct OscProps<'a, O: Osc, const OSCS: usize> {
    index: usize,
    enabled: bool,
    kind: O::Props<'a>,
    output: OscOutput,
    // TODO: Mix (better balanced between oscs)
    // TODO: Tuning
    tune_semitones: i8,
    tune_cents: i8,
}

#[cfg(feature = "egui")]
impl<'a, O: Osc, const OSCS: usize> crate::param::ui::EguiComponent for OscProps<'a, O, OSCS> {
    fn egui(&mut self, ui: &mut egui::Ui, params: crate::param::ui::DefaultUiParams) {
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

impl<'a, O: Osc, const OSCS: usize> Modulate for OscProps<'a, O, OSCS> {
    fn modulated(&self, f: impl FnMut(crate::modulation::mod_pack::ModTarget) -> ModValue) -> Self {
        Self {
            kind: self.kind.modulated(f),
            ..*self
        }
    }
}

impl<'a, O: Osc, const OSCS: usize> OscProps<'a, O, OSCS> {
    pub fn new(index: usize, osc: O::Props<'a>) -> Self {
        Self {
            index,
            enabled: index == 0,
            kind: osc,
            output: OscOutput::Direct,
            tune_semitones: 0,
            tune_cents: 0,
        }
    }

    pub fn kind_mut(&mut self) -> &mut O::Props<'a> {
        &mut self.kind
    }
}

pub struct OscParams<'a, O: Osc, const OSCS: usize> {
    pub props: OscProps<'a, O, OSCS>,
    pub pitch_mod: ModValue,
}

impl<'a, O: Osc, const OSCS: usize> OscParams<'a, O, OSCS> {
    fn tune_mod(&self) -> f32 {
        self.pitch_mod.as_sui().inner()
            + self.props.tune_semitones as f32 / 12.0
            + self.props.tune_cents as f32 / 1200.0
    }
}

// The state of a single oscillator
#[derive(Debug, Clone, Copy)]
pub struct OscState {
    last_cycle: u32,
}

// TODO: Pitch + Fine pitch, Syncing between oscillators (now synced through reset call), osc mix
pub struct OscPack<O: Osc, const OSCS: usize> {
    oscs: [O; OSCS],
    states: [OscState; OSCS],
}

impl<O: Osc, const OSCS: usize> MidiEventListener for OscPack<O, OSCS> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = note;
        let _ = velocity;
        self.states
            .iter_mut()
            .for_each(|state| state.last_cycle = 0);
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = note;
        let _ = velocity;
    }
}

impl<O: Osc + 'static, const OSCS: usize> OscPack<O, OSCS> {
    pub fn new(osc: impl Fn(usize) -> O) -> Self {
        Self {
            oscs: core::array::from_fn(osc),
            states: core::array::from_fn(|_| OscState {
                last_cycle: 0,
                // freq: 0.0,
            }),
        }
    }

    pub fn tick<'a>(
        &mut self,
        clock: &Clock,
        freq: Freq,
        params: &[OscParams<'a, O, OSCS>],
    ) -> Option<f32> {
        let (_, output) = self
            .oscs
            .iter_mut()
            .zip(params)
            .zip(self.states.iter_mut())
            .fold(
                (OscMod::Direct(0.0), 0.0),
                |(modulation, mix), ((osc, params), state)| {
                    if !params.props.enabled {
                        return (OscMod::None, mix);
                    }

                    let osc_fm = match modulation {
                        OscMod::FM(m) => m,
                        _ => 0.0,
                    };

                    let freq = fm(freq, params.tune_mod() + osc_fm);

                    let phase = clock.phase(freq, &mut state.last_cycle);

                    let output = osc.tick(phase, &params.props.kind);

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
            );

        Some(output)
    }
}
