use crate::{
    midi::event::MidiEventListener,
    osc::clock::Clock,
    param::{
        f32::{SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
};

use super::mod_pack::ModTarget;

// TODO: LUT?

#[derive(Debug, Clone, Default, PartialEq)]
pub enum LfoWaveform {
    // TODO: Preserve previously select pulse width
    /// Pulse waveform with specific pulse width
    Pulse(UnitInterval),
    #[default]
    Sine,
    Triangle,
    Saw,
    ReverseSaw,
}

// TODO: Sync
// pub enum LfoTrigger {
//     /// LFO restarts on each note
//     Trigger,
//     /// LFO acts like an envelope running once on each note
//     Envelope,
//     /// LFO does not retrigger and keeps running in a loop
//     Loop,
// }

// #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
// pub enum LfoTarget {
//     /// Voice level
//     #[default]
//     GlobalLevel,
//     /// Voice pitch
//     GlobalPitch,
//     /// Wavetable position
//     WtPos(usize),
// }

#[derive(Debug, Clone)]
pub struct LfoProps {
    pub index: usize,
    pub enabled: bool,
    pub amount: UnitInterval,
    // TODO: Store sample length instead of frequency
    pub freq: f32,
    pub waveform: LfoWaveform,
    // TODO: Multiple targets?
    pub target: ModTarget,
}

impl LfoProps {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            enabled: false,
            amount: UnitInterval::MAX,
            freq: 1.0,
            waveform: LfoWaveform::default(),
            target: ModTarget::default(),
        }
    }

    pub fn with_freq(&self, freq: f32) -> Self {
        let mut this = self.clone();
        this.freq = freq;
        this
    }
}

impl UiComponent for LfoProps {
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        _ui_params: &crate::param::ui::UiParams,
    ) {
        ui.v_stack(|ui| {
            ui.wave(|x| Lfo::at(x, self));

            ui.checkbox(&format!("LFO{} enabled", self.index), &mut self.enabled);

            if !self.enabled {
                return;
            }

            ui.unit_interval("Amount", &mut self.amount);
            ui.freq("Frequency", &mut self.freq, Some((0.01, 10.0)));
            ui.select(
                "Waveform",
                &mut self.waveform,
                [
                    // TODO: Preserve pulse width
                    ("Pulse", LfoWaveform::Pulse(UnitInterval::EQUILIBRIUM)),
                    ("Sine", LfoWaveform::Sine),
                    ("Triangle", LfoWaveform::Triangle),
                    ("Saw", LfoWaveform::Saw),
                    ("Reverse saw", LfoWaveform::ReverseSaw),
                ]
                .into_iter(),
            );

            if let LfoWaveform::Pulse(pulse_width) = &mut self.waveform {
                ui.unit_interval("Pulse width", pulse_width);
            }

            // TODO
            // ui.select(
            //     "Target",
            //     &mut self.target,
            //     [
            //         ("Pitch", ModTarget::GlobalPitch),
            //         ("Level", ModTarget::GlobalLevel),
            //     ]
            //     .into_iter()
            //     .chain(
            //         (0..OSCS)
            //             .map(|osc_index| ("Wavetable position", ModTarget::OscWtPos(osc_index))),
            //     ),
            // );
        });
    }
}

// TODO: Delay and rise
// TODO: Set freq by rate (1/4, 1/2, etc.). Need synth BPM for that
pub struct Lfo {
    state: bool,
    // phase: f32,
    last_cycle: u32,
    // TODO: Start phase?
}

impl MidiEventListener for Lfo {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = velocity;
        let _ = note;
        // self.phase = 0.0;
        self.last_cycle = 0;
        self.state = true;
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = clock;
        let _ = velocity;
        let _ = note;
        // self.phase = 0.0;
        self.state = false;
    }
}

impl Lfo {
    pub fn new() -> Self {
        Self {
            // phase: 0.0,
            last_cycle: 0,
            state: false,
        }
    }

    pub fn at(phase: f32, params: &LfoProps) -> f32 {
        match params.waveform {
            LfoWaveform::Pulse(pulse_width) => {
                // let pulse_width = ;
                if phase < pulse_width.inner() {
                    1.0
                } else {
                    -1.0
                }
            }
            LfoWaveform::Sine => (phase * core::f32::consts::TAU).sin(),
            LfoWaveform::Triangle => 1.0 - 2.0 * (2.0 * phase - 1.0).abs(),
            LfoWaveform::Saw => (phase * 2.0) - 1.0,
            LfoWaveform::ReverseSaw => 1.0 - (phase * 2.0),
        }
    }

    pub fn tick(&mut self, clock: &Clock, params: &LfoProps) -> Option<SignedUnitInterval> {
        if !params.enabled {
            return None;
        }

        let phase = clock.phase(params.freq, &mut self.last_cycle);

        // Continue one cycle of LFO even if it is not triggered to avoid clicking. So here we stop non-triggered LFO only when phase is zero, i.e. the cycle is complete
        if !self.state && phase == 0.0 {
            return None;
        }

        let value = Self::at(phase, params) * params.amount.inner();
        let value = SignedUnitInterval::new_checked(value);

        Some(value)
    }
}

pub struct LfoPack<const SIZE: usize> {
    lfos: [Lfo; SIZE],
}

impl<const SIZE: usize> MidiEventListener for LfoPack<SIZE> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.lfos
            .iter_mut()
            .for_each(|lfo| lfo.note_on(clock, note, velocity));
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        self.lfos
            .iter_mut()
            .for_each(|lfo| lfo.note_off(clock, note, velocity));
    }
}

impl<const SIZE: usize> LfoPack<SIZE> {
    pub fn new() -> Self {
        Self {
            lfos: core::array::from_fn(|_| Lfo::new()),
        }
    }

    pub fn tick(
        &mut self,
        clock: &Clock,
        target: ModTarget,
        params: &[LfoProps],
    ) -> Option<SignedUnitInterval> {
        debug_assert_eq!(params.len(), self.lfos.len());

        // Find first LFO with specified target. There should be only one LFO with this target
        // LFO does not tick (advance) unless the target matches. So tick can be called multiple times per cycle for each target individually
        params
            .iter()
            .zip(self.lfos.iter_mut())
            .filter_map(|(params, lfo)| {
                if params.target == target {
                    lfo.tick(clock, params)
                } else {
                    None
                }
            })
            .next()
    }
}
