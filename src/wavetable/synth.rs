use super::{osc::WavetableOsc, osc::WavetableParams, Wavetable};
use crate::{
    components::{
        env::{EnvPack, EnvParams},
        lfo::{LfoPack, LfoParams},
    },
    midi::{event::MidiEventListener, note::Note},
    osc::{clock::Clock, OscProps},
    param::{
        f32::{SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
    voice::{controller::VoicesController, Voice, VoiceParams},
};

// type GlobalWavetable< const WAVETABLE_DEPTH: usize, const WAVETABLE_LENGTH: usize> =
//     core::cell::LazyCell<Wavetable< WAVETABLE_DEPTH, WAVETABLE_LENGTH>>;

// static BASIC_WAVES_TABLE: GlobalWavetable = GlobalWavetable::new(|| {
//     Wavetable::from_rows([
//         // Sine
//         WavetableRow::new(|phase| (TAU * phase).sin()),
//         // Square
//         WavetableRow::new(|phase| if phase < 0.5 { 1.0 } else { -1.0 }),
//         // Triangle
//         WavetableRow::new(|phase| 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0),
//         // Saw
//         WavetableRow::new(|phase| 2.0 * (phase - (phase + 0.5).floor())),
//     ])
// });

pub struct WtSynth<
    const WAVETABLE_DEPTH: usize,
    const WAVETABLE_LENGTH: usize,
    const VOICES: usize = 8,
    const LFOS: usize = 1,
    const ENVS: usize = 1,
    const OSCS: usize = 1,
> {
    lfo_params: [LfoParams<OSCS>; LFOS],
    /// Global LFOs
    lfos: LfoPack<LFOS, OSCS>,

    env_params: [EnvParams<OSCS>; ENVS],
    /// Global ADSRs (envelopes)
    envs: EnvPack<ENVS>,

    osc_props: [OscProps<'static, WavetableOsc<WAVETABLE_DEPTH, WAVETABLE_LENGTH>>; OSCS],

    // oscs: [OscInstance<
    //     WavetableOsc<  WAVETABLE_DEPTH, WAVETABLE_LENGTH>,
    //     VOICES,
    //
    //     LFOS,
    //     ENVS,
    //     OSCS,
    // >; OSCS],
    voices:
        VoicesController<WavetableOsc<WAVETABLE_DEPTH, WAVETABLE_LENGTH>, VOICES, LFOS, ENVS, OSCS>,

    clock: Clock,
}

impl<
        const WAVETABLE_DEPTH: usize,
        const WAVETABLE_LENGTH: usize,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
    > UiComponent for WtSynth<WAVETABLE_DEPTH, WAVETABLE_LENGTH, VOICES, LFOS, ENVS>
{
    fn ui(
        &mut self,
        ui: &mut impl crate::param::ui::ParamUi,
        ui_params: &crate::param::ui::UiParams,
    ) {
        ui.h_stack(|ui| {
            self.voices.ui(ui, ui_params);
            self.osc_props
                .iter_mut()
                .for_each(|props| props.ui(ui, ui_params));
        });

        ui.h_stack(|ui| {
            self.lfo_params.iter_mut().for_each(|lfo| {
                lfo.ui(ui, ui_params);
            });

            self.env_params.iter_mut().for_each(|env| {
                env.ui(ui, ui_params);
            });
        });
    }
}

impl<
        const WAVETABLE_DEPTH: usize,
        const WAVETABLE_LENGTH: usize,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > MidiEventListener for WtSynth<WAVETABLE_DEPTH, WAVETABLE_LENGTH, VOICES, LFOS, ENVS, OSCS>
{
    fn note_on(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        self.envs.note_on(clock, note, velocity);
        self.lfos.note_on(clock, note, velocity);
        self.voices.note_on(clock, note, velocity);
    }

    fn note_off(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        self.envs.note_off(clock, note, velocity);
        self.lfos.note_off(clock, note, velocity);
        self.voices.note_off(clock, note, velocity);
    }
}

impl<
        const WAVETABLE_DEPTH: usize,
        const WAVETABLE_LENGTH: usize,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > WtSynth<WAVETABLE_DEPTH, WAVETABLE_LENGTH, VOICES, LFOS, ENVS, OSCS>
{
    pub fn new(
        sample_rate: u32,
        default_wavetable: &'static Wavetable<WAVETABLE_DEPTH, WAVETABLE_LENGTH>,
    ) -> Self {
        Self {
            lfo_params: core::array::from_fn(|_| LfoParams::default()),
            lfos: LfoPack::new(),
            env_params: core::array::from_fn(|_| EnvParams::new(sample_rate)),
            envs: EnvPack::new(),
            osc_props: core::array::from_fn(|_| {
                OscProps::new(WavetableParams::new(&default_wavetable))
            }),
            voices: VoicesController::new(|_| Voice::new(|_| WavetableOsc::new())),
            clock: Clock {
                sample_rate,
                tick: 0,
            },
        }
    }

    pub fn env(&self, index: usize) -> &EnvParams<OSCS> {
        &self.env_params[index]
    }

    pub fn lfo(&self, index: usize) -> &LfoParams<OSCS> {
        &self.lfo_params[index]
    }

    // pub fn stop_all(&mut self) {
    //     self.oscs.iter_mut().for_each(|osc| osc.voices.stop_all());
    // }

    pub fn tick(&mut self) -> Option<SignedUnitInterval> {
        let sample = self.voices.tick(
            &self.clock,
            &VoiceParams {
                osc_props: &self.osc_props,
                env_params: &self.env_params,
                lfo_params: &self.lfo_params,
            },
        );

        self.clock.tick += 1;

        sample
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }
}
