use paw::{
    daw::{channel_rack::Instrument, Daw},
    midi::{event::MidiEventListener, note::Note},
    modx::{lfo::LfoWaveform, mod_pack::ModTarget},
    osc::clock::{Clock, Freq},
    param::f32::UnitInterval,
    wavetable::synth::create_basic_wavetable_synth,
};
use plotters::{
    chart::ChartBuilder,
    prelude::{IntoDrawingArea, SVGBackend},
    series::LineSeries,
    style::{IntoFont, BLUE, WHITE},
};
use std::{
    env::{temp_dir, var},
    process::Command,
};

const OUTPUT_FILE: &str = "./examples/notebook.svg";
const SAMPLE_RATE: u32 = 24_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut synth = create_basic_wavetable_synth::<1, 1, 0, 1>(SAMPLE_RATE);

    synth.props_mut()[0].kind_mut().depth = 0;

    let lfo0 = &mut synth.lfo_mut()[0];
    lfo0.enabled = true;
    lfo0.amount = UnitInterval::MAX;
    lfo0.freq = Freq::Hz(5);
    lfo0.target = ModTarget::GlobalPitch;
    lfo0.waveform = LfoWaveform::Sine;

    let root = SVGBackend::new(OUTPUT_FILE, (1080, 720)).into_drawing_area();

    const NOTE: Note = Note::A4;
    const WAVE_REPEAT: usize = 10;
    const WAVE_LENGTH: usize =
        (WAVE_REPEAT as f32 * SAMPLE_RATE as f32 / NOTE.freq().inner()) as usize;
    println!("WaveLen: {WAVE_LENGTH}");

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .right_y_label_area_size(40)
        .margin(2)
        .caption("Kek", ("sans-serif", 50.0).into_font())
        .build_cartesian_2d(0.0f32..WAVE_LENGTH as f32, -1.0f32..1.0)?;
    root.fill(&WHITE)?;

    chart.configure_mesh().draw()?;

    let mut clock = Clock::zero(SAMPLE_RATE);

    // // Large clock tick to see if wave changes over time
    // clock.set((WAVE_LENGTH as u32 - 1) * 100_000);

    synth.note_on(&clock, NOTE, UnitInterval::MAX);

    chart.draw_series(LineSeries::new(
        (0..=WAVE_LENGTH).map(|x| {
            let p = (x as f32, synth.tick(&clock).mono_sum());
            clock.tick();
            p
        }),
        &BLUE,
    ))?;

    root.present()?;

    let _ = var("EDITOR").or(var("VISUAL")).map(|editor| {
        let mut path = temp_dir();
        path.push(OUTPUT_FILE);
        Command::new(editor)
            .arg(&path)
            .status()
            .expect("Something went wrong");
    });

    Ok(())
}
