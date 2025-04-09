use paw::{
    midi::{event::MidiEventListener, note::Note},
    osc::clock::Clock,
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
const SAMPLE_RATE: u32 = 48_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut synth = create_basic_wavetable_synth::<1, 0, 0, 1>(SAMPLE_RATE);

    let root = SVGBackend::new(OUTPUT_FILE, (1920, 1080)).into_drawing_area();

    const NOTE: Note = Note::A4;
    const WAVE_REPEAT: usize = 10;
    const WAVE_LENGTH: usize =
        (WAVE_REPEAT as f32 * SAMPLE_RATE as f32 / NOTE.freq().inner()) as usize + 1;
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
        (0..WAVE_LENGTH).map(|x| {
            let p = (x as f32, *synth.tick(&clock).left());
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
