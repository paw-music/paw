use paw::{
    daw::Daw, midi::note::Note, param::f32::UnitInterval,
    wavetable::synth::create_basic_wavetable_synth,
};

const SAMPLE_RATE: u32 = 48_000;
const CHANNEL_RACK_SIZE: usize = 4;
const MIXER_SIZE: usize = 4;
const VOICES: usize = 16;
const LFOS: usize = 3;
const ENVS: usize = 3;
const OSCS: usize = 3;
const FX_SLOTS: usize = 4;

fn main() {
    let mut daw = Daw::<CHANNEL_RACK_SIZE, MIXER_SIZE, FX_SLOTS>::new(SAMPLE_RATE);

    daw.rack_mut()
        .push_instrument(Box::new(create_basic_wavetable_synth::<
            VOICES,
            LFOS,
            ENVS,
            OSCS,
        >(SAMPLE_RATE)))
        .unwrap();

    for note in Note::each() {
        daw.note_on(note, UnitInterval::MAX);

        for _ in 0..SAMPLE_RATE {
            daw.tick_internal();
        }

        daw.note_off(note, UnitInterval::MAX);
    }
}
