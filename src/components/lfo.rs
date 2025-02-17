use crate::{
    midi::event::MidiEventListener,
    osc::clock::Clock,
    param::{
        f32::{SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
};

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

// TODO?
// pub enum LfoTrigger {
//     /// LFO restarts on each note
//     Trigger,
//     /// LFO acts like an envelope running once on each note
//     Envelope,
//     /// LFO does not retrigger and keeps running in a loop
//     Loop,
// }

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LfoTarget {
    /// Voice level
    #[default]
    GlobalLevel,
    /// Voice pitch
    GlobalPitch,
    /// Wavetable position
    WtPos(usize),
}

#[derive(Debug, Clone)]
pub struct LfoParams<const OSCS: usize> {
    pub enabled: bool,
    pub amount: UnitInterval,
    // TODO: Store sample length instead of frequency
    pub freq: f32,
    pub waveform: LfoWaveform,
    pub target: LfoTarget,
}

impl<const OSCS: usize> LfoParams<OSCS> {
    pub fn with_freq(&self, freq: f32) -> Self {
        let mut this = self.clone();
        this.freq = freq;
        this
    }
}

impl<const OSCS: usize> Default for LfoParams<OSCS> {
    fn default() -> Self {
        Self {
            enabled: false,
            amount: UnitInterval::MAX,
            freq: 1.0,
            waveform: LfoWaveform::default(),
            target: LfoTarget::default(),
        }
    }
}

impl<const OSCS: usize> UiComponent for LfoParams<OSCS> {
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        _ui_params: &crate::param::ui::UiParams,
    ) {
        ui.v_stack(|ui| {
            ui.wave(|x| Lfo::at(x, self));

            ui.checkbox("Lfo enabled", &mut self.enabled);

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

            ui.select(
                "Target",
                &mut self.target,
                [
                    ("Pitch", LfoTarget::GlobalPitch),
                    ("Level", LfoTarget::GlobalLevel),
                ]
                .into_iter()
                .chain(
                    (0..OSCS).map(|osc_index| ("Wavetable position", LfoTarget::WtPos(osc_index))),
                ),
            );
        });
    }
}

// TODO: Delay and rise
// TODO: Set freq by rate (1/4, 1/2, etc.). Need synth BPM for that
pub struct Lfo<const OSCS: usize> {
    state: bool,
    // phase: f32,
    last_cycle: u32,
    // TODO: Start phase?
}

impl<const OSCS: usize> MidiEventListener for Lfo<OSCS> {
    fn note_on(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = velocity;
        let _ = note;
        // self.phase = 0.0;
        self.last_cycle = 0;
        self.state = true;
    }

    fn note_off(&mut self, clock: &Clock, note: crate::midi::note::Note, velocity: UnitInterval) {
        let _ = velocity;
        let _ = note;
        // self.phase = 0.0;
        self.state = false;
    }
}

impl<const OSCS: usize> Lfo<OSCS> {
    pub fn new() -> Self {
        Self {
            // phase: 0.0,
            last_cycle: 0,
            state: false,
        }
    }

    pub fn at(phase: f32, params: &LfoParams<OSCS>) -> f32 {
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

    pub fn tick(&mut self, clock: &Clock, params: &LfoParams<OSCS>) -> SignedUnitInterval {
        if !params.enabled {
            return SignedUnitInterval::EQUILIBRIUM;
        }

        // let length = SAMPLE_RATE as f32 / params.freq.to_num::<f32>();
        // let length = length.max(1.0);

        // let phase = self.index as f32 / SAMPLE_RATE as f32;

        let phase = clock.phase(params.freq, &mut self.last_cycle);

        if !self.state && phase == 0.0 {
            return SignedUnitInterval::EQUILIBRIUM;
        }

        let value = Self::at(phase, params);

        // self.index = (self.index + 1) % length as usize;

        SignedUnitInterval::new_checked(value * params.amount.inner())
    }
}

pub struct LfoPack<const SIZE: usize, const OSCS: usize> {
    lfos: [Lfo<OSCS>; SIZE],
}

impl<const SIZE: usize, const OSCS: usize> MidiEventListener for LfoPack<SIZE, OSCS> {
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

impl<const SIZE: usize, const OSCS: usize> LfoPack<SIZE, OSCS> {
    pub fn new() -> Self {
        Self {
            lfos: core::array::from_fn(|_| Lfo::new()),
        }
    }

    // pub fn tick<'a>(&mut self, params: impl Iterator<Item = &'a LfoParams<SAMPLE_RATE>>) -> f32 {
    //     self.lfos
    //         .iter_mut()
    //         .zip(params)
    //         .map(|(lfo, params)| lfo.tick(params))
    //         .sum::<f32>()
    //         / self.lfos.len() as f32
    // }

    pub fn tick(
        &mut self,
        clock: &Clock,
        target: LfoTarget,
        params: &[LfoParams<OSCS>],
    ) -> Option<SignedUnitInterval> {
        debug_assert_eq!(params.len(), self.lfos.len());

        // Find first LFO with specified target. There should be only one LFO with this target
        // LFO does not tick (advance) unless the target matches. So tick can be called multiple times per cycle for each target individually
        params
            .iter()
            .zip(self.lfos.iter_mut())
            .filter_map(|(params, lfo)| {
                if params.target == target {
                    Some(lfo.tick(clock, params))
                } else {
                    None
                }
            })
            .next()
    }
}
