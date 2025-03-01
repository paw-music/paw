use super::{osc::WavetableOsc, Wavetable, WavetableProps};
use crate::{
    fx::{
        delay::{Delay, DelayKind, DelayParams},
        dist::{Dist, DistParams},
    },
    midi::{event::MidiEventListener, note::Note},
    modulation::{env::EnvProps, lfo::LfoProps, mod_pack::ModPack, Modulate},
    osc::{clock::Clock, OscParams, OscProps},
    param::{
        f32::{SignedUnitInterval, UnitInterval},
        ui::UiComponent,
    },
    sample::{time::SampleCount, Frame},
    voice::{controller::VoicesController, Voice, VoiceParams},
};

// TODO: Turn into generic synthesizer
pub struct WtSynth<
    const WAVETABLE_DEPTH: usize,
    const WAVETABLE_LENGTH: usize,
    const VOICES: usize = 8,
    const LFOS: usize = 1,
    const ENVS: usize = 1,
    const OSCS: usize = 1,
> {
    lfo_props: [LfoProps; LFOS],
    env_props: [EnvProps; ENVS],

    /// Global ADSRs (envelopes) and LFOs
    mods: ModPack<LFOS, ENVS, OSCS>,

    osc_props: [OscProps<'static, WavetableOsc<WAVETABLE_DEPTH, WAVETABLE_LENGTH>, OSCS>; OSCS],

    voices:
        VoicesController<WavetableOsc<WAVETABLE_DEPTH, WAVETABLE_LENGTH>, VOICES, LFOS, ENVS, OSCS>,

    clock: Clock,

    delay: Delay<48_000>,
    delay_params: DelayParams,

    dist_params: DistParams,
    dist: Dist,
}

impl<
        const WAVETABLE_DEPTH: usize,
        const WAVETABLE_LENGTH: usize,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > UiComponent for WtSynth<WAVETABLE_DEPTH, WAVETABLE_LENGTH, VOICES, LFOS, ENVS, OSCS>
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
            self.lfo_props.iter_mut().for_each(|lfo| {
                lfo.ui(ui, ui_params);
            });

            self.env_props.iter_mut().for_each(|env| {
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
        self.mods.note_on(clock, note, velocity);
        self.voices.note_on(clock, note, velocity);
    }

    fn note_off(&mut self, clock: &Clock, note: Note, velocity: UnitInterval) {
        self.mods.note_off(clock, note, velocity);
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
            lfo_props: core::array::from_fn(|index| LfoProps::new(index)),
            env_props: core::array::from_fn(|index| EnvProps::new(index, sample_rate)),
            mods: ModPack::new(),
            osc_props: core::array::from_fn(|index| {
                OscProps::new(index, WavetableProps::new(index, &default_wavetable))
            }),
            voices: VoicesController::new(|_| Voice::new(|_| WavetableOsc::default())),
            clock: Clock {
                sample_rate,
                tick: 0,
            },

            delay: Delay::new(sample_rate),
            delay_params: DelayParams {
                amount: UnitInterval::EQUILIBRIUM,
                feedback: UnitInterval::new(0.5),
                time: SampleCount::from_millis(200, sample_rate),
                kind: DelayKind::PingPong,
            },

            dist_params: DistParams {
                kind: crate::fx::dist::DistKind::HalfWaveRect,
                input: UnitInterval::new(1.0),
            },
            dist: Dist::new(),
        }
    }

    pub fn env_props_mut(&mut self) -> &mut [EnvProps] {
        &mut self.env_props
    }

    pub fn lfo_props_mut(&mut self) -> &mut [LfoProps] {
        &mut self.lfo_props
    }

    pub fn tick(&mut self) -> Option<Frame> {
        // Note: Need array allocation because we cannot pass slice (params are modulated) and don't want a vector
        let osc_params = core::array::from_fn(|index| OscParams {
            props: self.osc_props[index].modulated(|target| {
                self.mods
                    .tick(&self.clock, target, &self.lfo_props, &self.env_props)
            }),
            pitch_mod: self
                .mods
                .tick(
                    &self.clock,
                    crate::modulation::mod_pack::ModTarget::GlobalPitch,
                    &self.lfo_props,
                    &self.env_props,
                )
                .unwrap_or(SignedUnitInterval::EQUILIBRIUM),
        });

        let amp_mod = self
            .mods
            .tick(
                &self.clock,
                crate::modulation::mod_pack::ModTarget::GlobalLevel,
                &self.lfo_props,
                &self.env_props,
            )
            .map(|amp_mod| amp_mod.remap_into_unsigned());

        let frame = self.voices.tick(
            &self.clock,
            &VoiceParams {
                env_params: &self.env_props,
                lfo_params: &self.lfo_props,
                osc_params,
                amp_mod,
            },
        );

        let frame = frame.map(|frame| self.delay.tick(&self.clock, frame, &self.delay_params));

        let frame = frame.map(|frame| self.dist.tick(frame, &self.dist_params));

        self.clock.tick = self.clock.tick.wrapping_add(1);

        frame
    }

    pub fn clock(&self) -> Clock {
        self.clock
    }
}
