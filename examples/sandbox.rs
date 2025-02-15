use nannou::prelude::*;
use nannou_audio::{self as audio, Buffer};
use nannou_egui::Egui;
use paw::{
    midi::{event::MidiEventListener as _, note::Note},
    osc::clock::Clock,
    param::{
        f32::UnitInterval,
        ui::{UiComponent, UiParams},
    },
    wavetable::{synth::WtSynth, Wavetable, WavetableRow},
};
use std::{
    sync::{Arc, LazyLock, Mutex},
    time::{Duration, Instant},
};

type Sample = f32;
const WAVETABLE_DEPTH: usize = 4;
const WAVETABLE_LENGTH: usize = 1024;
const SAMPLE_RATE: u32 = 44_100;
const VOICES: usize = 8;
const LFOS: usize = 2;
const ENVS: usize = 2;

type GlobalWavetable = LazyLock<Wavetable<WAVETABLE_DEPTH, WAVETABLE_LENGTH>>;

fn note_from_nannou_key(key: nannou::event::Key) -> Result<Note, ()> {
    match key {
        // First octave
        Key::Z => Ok(Note::C0),
        Key::S => Ok(Note::Cs0),
        Key::X => Ok(Note::D0),
        Key::D => Ok(Note::Ds0),
        Key::C => Ok(Note::E0),
        Key::V => Ok(Note::F0),
        Key::G => Ok(Note::Fs0),
        Key::B => Ok(Note::G0),
        Key::H => Ok(Note::Gs0),
        Key::N => Ok(Note::A0),
        Key::J => Ok(Note::As0),
        Key::M => Ok(Note::B0),

        // Part of second octave
        Key::Comma => Ok(Note::C1),
        Key::L => Ok(Note::Cs1),
        Key::Period => Ok(Note::D1),
        Key::Semicolon => Ok(Note::Ds1),
        Key::Slash => Ok(Note::E1),

        // Second octave
        Key::Q => Ok(Note::C1),
        Key::Key2 => Ok(Note::Cs1),
        Key::W => Ok(Note::D1),
        Key::Key3 => Ok(Note::Ds1),
        Key::E => Ok(Note::E1),
        Key::R => Ok(Note::F1),
        Key::Key5 => Ok(Note::Fs1),
        Key::T => Ok(Note::G1),
        Key::Key6 => Ok(Note::Gs1),
        Key::Y => Ok(Note::A1),
        Key::Key7 => Ok(Note::As1),
        Key::U => Ok(Note::B1),

        // Part of third octave
        Key::I => Ok(Note::C2),
        Key::Key9 => Ok(Note::Cs2),
        Key::O => Ok(Note::D2),
        Key::Key0 => Ok(Note::Ds2),
        Key::P => Ok(Note::E2),

        _ => Err(()),
    }
}

/// Get keyboard key id by PC keyboard key. This key is not related to MIDI notes in any way, every key used for MIDI input just needs its identifier, includes those producing the same note.
/// I'm sure there're less than 128 physical keys :)
fn key_id_from_nannou_key(key: nannou::event::Key) -> Result<u8, ()> {
    match key {
        // First octave
        Key::Z => Ok(0),
        Key::S => Ok(1),
        Key::X => Ok(2),
        Key::D => Ok(3),
        Key::C => Ok(4),
        Key::V => Ok(5),
        Key::G => Ok(6),
        Key::B => Ok(7),
        Key::H => Ok(8),
        Key::N => Ok(9),
        Key::J => Ok(10),
        Key::M => Ok(11),

        // Part of second octave
        Key::Comma => Ok(12),
        Key::L => Ok(13),
        Key::Period => Ok(14),
        Key::Semicolon => Ok(15),
        Key::Slash => Ok(16),

        // Second octave
        Key::Q => Ok(17),
        Key::Key2 => Ok(18),
        Key::W => Ok(19),
        Key::Key3 => Ok(20),
        Key::E => Ok(21),
        Key::R => Ok(22),
        Key::Key5 => Ok(23),
        Key::T => Ok(24),
        Key::Key6 => Ok(25),
        Key::Y => Ok(26),
        Key::Key7 => Ok(27),
        Key::U => Ok(28),

        // Part of third octave
        Key::I => Ok(29),
        Key::Key9 => Ok(30),
        Key::O => Ok(31),
        Key::Key0 => Ok(32),
        Key::P => Ok(33),

        _ => Err(()),
    }
}

static BASIC_WAVES_TABLE: GlobalWavetable = GlobalWavetable::new(|| {
    Wavetable::from_rows([
        // Sine
        WavetableRow::new(|phase| (TAU * phase).sin()),
        // Square
        WavetableRow::new(|phase| if phase < 0.5 { 1.0 } else { -1.0 }),
        // Triangle
        WavetableRow::new(|phase| 2.0 * (2.0 * (phase - (phase + 0.5).floor())).abs() - 1.0),
        // Saw
        WavetableRow::new(|phase| 2.0 * (phase - (phase + 0.5).floor())),
    ])
});

struct Synth {
    synth: WtSynth<WAVETABLE_DEPTH, WAVETABLE_LENGTH, VOICES, LFOS, ENVS>,
    clock: Clock,
}

// impl Synth {
//     pub fn has_note(&self, note: Note) -> bool {
//         self.notes
//             .iter()
//             .copied()
//             .find(|active| *active == Some(note))
//             .is_some()
//     }

//     pub fn add_note(&mut self, note: Note) {
//         if self.has_note(note) {
//             return;
//         }

//         if let Some(first_free) = self.notes.iter().position(|note| note.is_none()) {
//             self.notes[first_free] = Some(note);
//         }
//     }

//     pub fn remove_note(&mut self, note: Note) {
//         if let Some(note_pos) = self.notes.iter().position(|active| *active == Some(note)) {
//             self.notes[note_pos] = None;
//         }
//     }
// }

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    stream: audio::Stream<AudioModel>,
    octave: u8,
    synth: Arc<Mutex<Synth>>,
    draw_voices: bool,
    egui: Egui,
    /// Mapping from pressed key (identified by key id from `key_id_from_nannou_key`) to played note. This is needed when user presses a key, changes octave (or transposes) but keeps key pressed, and to "note off" this note when key is released, not to lose actual note transposition.
    pressed_keys_notes: Vec<Option<Note>>,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .raw_event(raw_window_event)
        .event(event)
        .title("SANDBOX")
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    let audio_host = audio::Host::new();

    let synth = Synth {
        synth: WtSynth::new(SAMPLE_RATE, &BASIC_WAVES_TABLE),
        clock: Clock {
            sample_rate: SAMPLE_RATE,
            tick: 0,
        },
    };
    let synth = Arc::new(Mutex::new(synth));

    let audio_model = AudioModel {
        synth: Arc::clone(&synth),
    };

    let stream = audio_host
        .new_output_stream(audio_model)
        .render(audio)
        // .sample_rate(SAMPLE_RATE)
        .build()
        .unwrap();

    // println!("Stream conf: {:?}", stream.cpal_config());

    stream.play().unwrap();

    let egui = Egui::from_window(&window);

    Model {
        stream,
        octave: 0,
        synth,
        egui,
        draw_voices: false,
        pressed_keys_notes: vec![None; 128],
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    let synth = &mut model.synth.lock().unwrap().synth;

    match event {
        KeyPressed(key) => {
            println!("Key press {key:?}");
            if let Ok(note) = note_from_nannou_key(key) {
                let note = note.saturating_add(model.octave as i16 * 12);

                // All MIDI mapped notes must have identifiers
                let key_id = key_id_from_nannou_key(key).unwrap() as usize;

                // // Stop previously played note on the same physical key in case when two note-on received for the same key without note-off
                // if let Some(prev_note) = model.pressed_keys_notes[key_id].take() {
                //     if prev_note != note {
                //         synth.note_off(prev_note, UnitInterval::MAX);
                //     }
                // }

                if model.pressed_keys_notes[key_id].is_some() {
                    return;
                }

                synth.note_on(note, UnitInterval::MAX);

                assert!(model.pressed_keys_notes[key_id].replace(note).is_none());
            } else {
                // match key {
                //     Key::Space => synth.paused = !synth.paused,
                //     _ => {}
                // }
            }
        }
        KeyReleased(key) => {
            println!("Key release {key:?}");
            if let Ok(note) = note_from_nannou_key(key) {
                if let Some(prev_note) =
                    model.pressed_keys_notes[key_id_from_nannou_key(key).unwrap() as usize].take()
                {
                    synth.note_off(prev_note, UnitInterval::MAX);
                }

                synth.note_off(
                    note.saturating_add(model.octave as i16 * 12),
                    UnitInterval::MIN,
                );
            } else {
                // TODO: Transpose played notes instead of reset
                match key {
                    Key::LBracket => model.octave = (model.octave.saturating_sub(1)).clamp(0, 8),
                    Key::RBracket => model.octave = (model.octave.saturating_add(1)).clamp(0, 8),
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    // Let egui handle things like keyboard and mouse input.
    model.egui.handle_raw_event(event);
}

// fn get_clock() -> Clock {
//     static START: LazyLock<Instant> = LazyLock::new(|| Instant::now());

//     const ONE_SAMPLE: Duration = Duration::from_nanos(1_000_000_000 / SAMPLE_RATE as u64);

//     let clock = Clock::new(
//         SAMPLE_RATE,
//         std::time::Instant::now()
//             .duration_since(*START)
//             .div_duration_f32(ONE_SAMPLE) as u32,
//     );

//     println!("Clock {}", clock.tick);

//     clock
// }

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    let synth = &mut model.synth.lock().unwrap().synth;

    nannou_egui::egui::Window::new("Synth")
        .fixed_size((250.0, 500.0))
        .show(&ctx, |ui| {
            synth.ui(
                ui,
                &UiParams {
                    clock: synth.clock(),
                },
            );
        });
}

struct AudioModel {
    synth: Arc<Mutex<Synth>>,
}

fn audio(audio: &mut AudioModel, buffer: &mut Buffer) {
    assert_eq!(buffer.sample_rate(), SAMPLE_RATE);

    let synth = &mut audio.synth.lock().unwrap().synth;

    // let volume = 0.5;
    for frame in buffer.frames_mut() {
        // let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        // audio.phase += audio.hz / sample_rate;
        // audio.phase %= sample_rate;
        // for channel in frame {
        //     *channel = sine_amp * volume;
        // }

        let sample = synth.tick().unwrap();
        for channel in frame {
            *channel = sample.inner();
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    const WAVE_HEIGHT: f32 = 200.0;

    let window_width = app.main_window().inner_size_points().0;
    // let synth = &mut model.synth.lock().unwrap().synth;

    draw.line()
        .start(Vec2::new(-window_width / 2.0, 0.0))
        .end(Vec2::new(window_width / 2.0, 0.0))
        .weight(1.0)
        .color(GREY);

    draw.line()
        .start(pt2(-window_width / 2.0, -WAVE_HEIGHT))
        .end(pt2(window_width / 2.0, -WAVE_HEIGHT))
        .color(GREY)
        .weight(1.0);

    draw.line()
        .start(pt2(-window_width / 2.0, WAVE_HEIGHT))
        .end(pt2(window_width / 2.0, WAVE_HEIGHT))
        .color(GREY)
        .weight(1.0);

    // let row = synth.voice_n(0).osc().current_row();

    // draw.polyline()
    //     .weight(2.0)
    //     .points_colored((0..window_width as usize).map(|x| {
    //         // debug_assert!(sample >= -1.0 && sample <= 1.0, "Malformed sample {sample}");

    //         let y = row.lerp(x as f32 / window_width as f32);

    //         (pt2(x as f32 - window_width / 2.0, y * WAVE_HEIGHT), GREEN)
    //     }));

    // ADSR //
    // let adsr_params = synth.env(0);

    // const ADSR_1SEC_WIDTH: f32 = 200.0;
    // const ADSR_HEIGHT: f32 = 100.0;
    // const ADSR_SUSTAIN_LENGTH: usize = 50;
    // const ADSR_DRAW_DECIMATION: usize = 100;
    // let adsr_pos = pt2(200.0, 200.0);

    // let adsr_scale_ratio = ADSR_1SEC_WIDTH / SAMPLE_RATE as f32;

    // let mut adsr = Env::new();

    // adsr.note_on(1.0);
    // let mut adsr_levels = (0..(adsr_params.before_sustain()).inner())
    //     .map(|_| adsr.tick(adsr_params).unwrap())
    //     .step_by(ADSR_DRAW_DECIMATION)
    //     .collect::<Vec<_>>();
    // adsr_levels.extend((0..ADSR_SUSTAIN_LENGTH).map(|_| adsr_params.sustain.inner()));
    // adsr.note_off();
    // adsr_levels.extend(
    //     (0..adsr_params.release.inner())
    //         .map(|_| adsr.tick(adsr_params).unwrap())
    //         .step_by(ADSR_DRAW_DECIMATION),
    // );

    // draw.polyline()
    //     .weight(1.0)
    //     .points_colored(adsr_levels.iter().enumerate().map(|(i, level)| {
    //         (
    //             pt2(
    //                 i as f32 * adsr_scale_ratio * ADSR_DRAW_DECIMATION as f32,
    //                 level * ADSR_HEIGHT,
    //             ) + adsr_pos,
    //             GOLDENROD,
    //         )
    //     }));

    // LFO //
    // let lfo_width = 200.0;
    // let lfo_height = 100.0;
    // let lfo_pos = pt2(-200.0, 250.0);
    // let mut draw_lfo = Lfo::new();
    // draw_lfo.note_on();

    // let lfo_params = synth
    //     .lfo(0)
    //     .with_freq(Freq::from_num(SAMPLE_RATE as f32 / lfo_width));

    // let lfo_points = (0..lfo_width as usize)
    //     .map(|_| draw_lfo.tick(&lfo_params))
    //     .enumerate()
    //     .map(|(index, sample)| {
    //         let x = index as f32;
    //         let y = sample / 2.0 * lfo_height;

    //         (pt2(x, y) + lfo_pos, CADETBLUE)
    //     });

    // draw.line()
    //     .weight(1.0)
    //     .start(lfo_pos)
    //     .end(lfo_pos + pt2(lfo_width, 0.0))
    //     .color(GREY);
    // draw.polyline().weight(1.0).points_colored(lfo_points);

    // if model.draw_voices {
    //     synth
    //         .voices
    //         .voices_detune()
    //         .for_each(|(voice_detune, voice_amp)| {
    //             // Get detune amount from detune factor
    //             let x = (1.0 - voice_detune) * window_width;
    //             let is_center = voice_detune == 1.0;
    //             draw.line()
    //                 .start(Vec2::new(x, 0.0))
    //                 .end(Vec2::new(x, -blend_height * voice_amp))
    //                 .color(if is_center { LIGHTPINK } else { PURPLE })
    //                 .weight(if is_center { 2.0 } else { 1.0 });
    //         });
    // }

    // let unison_pos = pt2(200.0, -250.0);
    // let unison_width = 250.0;
    // let blend_height = 100.0;

    // if let Some(voices_detune) = synth.voices.voices_detune() {
    //     voices_detune.for_each(|(detune, blend)| {
    //         let x = (1.0 - detune) * unison_width;
    //         let is_center = detune == 1.0;

    //         draw.line()
    //             .start(pt2(x, 0.0) + unison_pos)
    //             .end(pt2(x, -blend_height * blend) + unison_pos)
    //             .color(if is_center { LIGHTPINK } else { PURPLE })
    //             .weight(if is_center { 2.0 } else { 1.0 });
    //     });

    //     draw.ellipse().radius(5.0).color(LIGHTCORAL).xy(unison_pos);
    // }

    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}
