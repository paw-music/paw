use crate::{
    modulation::{env::EnvProps, lfo::LfoProps, mod_pack::ModPack, Modulate as _},
    osc::{clock::Clock, Osc, OscParams, OscProps},
    param::f32::SignedUnitInterval,
    sample::Frame,
    voice::{controller::VoicesController, Voice, VoiceParams},
};

pub struct Synth<
    O: Osc,
    const VOICES: usize,
    const LFOS: usize = 1,
    const ENVS: usize = 1,
    const OSCS: usize = 1,
> {
    lfo_props: [LfoProps; LFOS],
    env_props: [EnvProps; ENVS],

    /// Global ADSRs (envelopes) and LFOs
    mods: ModPack<LFOS, ENVS, OSCS>,

    osc_props: [OscProps<'static, O, OSCS>; OSCS],

    voices: VoicesController<O, VOICES, LFOS, ENVS, OSCS>,
}

impl<
        O: Osc + 'static,
        const VOICES: usize,
        const LFOS: usize,
        const ENVS: usize,
        const OSCS: usize,
    > Synth<O, VOICES, LFOS, ENVS, OSCS>
{
    pub fn new(sample_rate: u32, osc_props: impl Fn(usize) -> O::Props<'static>) -> Self {
        Self {
            lfo_props: core::array::from_fn(|index| LfoProps::new(index)),
            env_props: core::array::from_fn(|index| EnvProps::new(index, sample_rate)),
            mods: ModPack::new(),
            osc_props: core::array::from_fn(|index| OscProps::new(index, osc_props(index))),
            voices: VoicesController::new(|_| Voice::new(|_| O::default())),
        }
    }

    pub fn tick(&mut self, clock: &Clock) -> Option<Frame> {
        // Note: Need array allocation because we cannot pass slice (params are modulated) and don't want a vector
        let osc_params = core::array::from_fn(|index| OscParams {
            props: self.osc_props[index].modulated(|target| {
                self.mods
                    .tick(clock, target, &self.lfo_props, &self.env_props)
            }),
            pitch_mod: self
                .mods
                .tick(
                    clock,
                    crate::modulation::mod_pack::ModTarget::GlobalPitch,
                    &self.lfo_props,
                    &self.env_props,
                )
                .unwrap_or(SignedUnitInterval::EQUILIBRIUM),
        });

        let amp_mod = self
            .mods
            .tick(
                clock,
                crate::modulation::mod_pack::ModTarget::GlobalLevel,
                &self.lfo_props,
                &self.env_props,
            )
            .map(|amp_mod| amp_mod.remap_into_unsigned());

        let frame = self.voices.tick(
            clock,
            &VoiceParams {
                env_params: &self.env_props,
                lfo_params: &self.lfo_props,
                osc_params,
                amp_mod,
            },
        );

        frame
    }
}
