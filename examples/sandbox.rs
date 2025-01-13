use std::{
    cell::OnceCell,
    sync::{Arc, LazyLock, Mutex},
    thread,
    time::Duration,
};

use fixed::traits::ToFixed;
use nannou::{color, prelude::*};
use nannou_audio::{self as audio, Buffer};
use nannou_egui::{
    egui::{DragValue, Slider},
    Egui,
};
use paw::{
    adsr::Adsr,
    midi::note::Note,
    osc::Osc as _,
    sample::time::SampleCount,
    voice::{Voice, VoicesController},
    wavetable::{osc::WavetableOsc, Wavetable, WavetableRow},
};

type Sample = f32;
const WAVETABLE_DEPTH: usize = 4;
const WAVETABLE_LENGTH: usize = 2048;
const SAMPLE_RATE: u32 = 44_100;
const MAX_VOICES: usize = 8;
const MAX_NOTES: usize = 8;

type GlobalWavetable = LazyLock<Wavetable<Sample, WAVETABLE_DEPTH, WAVETABLE_LENGTH>>;

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
    voices: VoicesController<
        WavetableOsc<'static, Sample, SAMPLE_RATE, WAVETABLE_DEPTH, WAVETABLE_LENGTH>,
        MAX_VOICES,
        SAMPLE_RATE,
    >,
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

    let mut voices = VoicesController::<_, MAX_VOICES, SAMPLE_RATE>::new(|_| {
        Voice::new(WavetableOsc::new(&BASIC_WAVES_TABLE))
    });
    // voices.set_freq(440.0);
    let synth = Synth { voices };
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

    println!("Stream conf: {:?}", stream.cpal_config());

    stream.play().unwrap();

    let egui = Egui::from_window(&window);

    Model {
        stream,
        octave: 0,
        synth,
        egui,
        draw_voices: false,
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    let mut synth = model.synth.lock().unwrap();

    match event {
        KeyPressed(key) => {
            println!("Key press {key:?}");
            if let Ok(note) = note_from_nannou_key(key) {
                synth
                    .voices
                    .note_on(note.saturating_add(model.octave as i16 * 12), 1.0);
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
                synth
                    .voices
                    .note_off(note.saturating_add(model.octave as i16 * 12));
            } else {
                // TODO: Transpose played notes instead of reset
                match key {
                    Key::LBracket => {
                        synth.voices.stop_all();
                        model.octave = (model.octave.saturating_sub(1)).clamp(0, 8)
                    }
                    Key::RBracket => {
                        synth.voices.stop_all();
                        model.octave = (model.octave.saturating_add(1)).clamp(0, 8)
                    }
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

fn update(_app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);
    let ctx = egui.begin_frame();

    let mut synth = model.synth.lock().unwrap();

    nannou_egui::egui::Window::new("Synth")
        .fixed_size((250.0, 500.0))
        .show(&ctx, |ui| {
            ui.heading("ADSR");
            let adsr = synth.voices.adsr_mut();

            ui.add(
                Slider::from_get_set(0.0..=10_000.0, |new_value| {
                    if let Some(new_value) = new_value {
                        adsr.set_attack(SampleCount::from_millis(new_value as u32));
                    }

                    adsr.attack().inner() as f64
                })
                .text("Attack")
                .custom_formatter(|attack, _| {
                    SampleCount::<SAMPLE_RATE>::new(attack as u32).to_string()
                })
                .logarithmic(true)
                .integer(),
            );
            ui.add(
                Slider::from_get_set(0.0..=10_000.0, |new_value| {
                    if let Some(new_value) = new_value {
                        adsr.set_decay(SampleCount::from_millis(new_value as u32));
                    }

                    adsr.decay().inner() as f64
                })
                .text("Decay")
                .custom_formatter(|decay, _| {
                    SampleCount::<SAMPLE_RATE>::new(decay as u32).to_string()
                })
                .logarithmic(true)
                .integer(),
            );
            ui.add(
                Slider::from_get_set(0.0..=1.0, |new_value| {
                    if let Some(new_value) = new_value {
                        adsr.set_sustain(new_value as f32);
                    }

                    adsr.sustain() as f64
                })
                .text("Sustain"),
            );
            ui.add(
                Slider::from_get_set(0.0..=10_000.0, |new_value| {
                    if let Some(new_value) = new_value {
                        adsr.set_release(SampleCount::from_millis(new_value as u32));
                    }

                    adsr.release().inner() as f64
                })
                .text("Release")
                .custom_formatter(|release, _| {
                    SampleCount::<SAMPLE_RATE>::new(release as u32).to_string()
                })
                .logarithmic(true)
                .integer(),
            );

            // ui.add(
            //     Slider::from_get_set(20.0..=20_000.0, |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.set_freq(new_value as f32);
            //         }
            //         synth.voices.freq() as f64
            //     })
            //     .text("Frequency")
            //     .drag_value_speed(0.5)
            //     .trailing_fill(true)
            //     .logarithmic(true),
            // );

            // ui.add(
            //     Slider::from_get_set(0.0..=(WAVETABLE_DEPTH - 1) as f64, |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.iter_all_voices_mut().for_each(|voice| {
            //                 voice.osc_mut().set_depth(new_value as usize);
            //             });
            //         }

            //         synth
            //             .voices
            //             .iter_active_voices()
            //             .next()
            //             .map(|voice| voice.osc().depth())
            //             .unwrap() as f64
            //     })
            //     .integer()
            //     .text("Table depth"),
            // );

            // ui.add(
            //     Slider::from_get_set(0.0..=1.0, |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.iter_all_voices_mut().for_each(|voice| {
            //                 voice.osc_mut().set_start_phase(new_value as f32);
            //             });
            //         }

            //         synth
            //             .voices
            //             .iter_active_voices()
            //             .next()
            //             .map(|voice| voice.osc().start_phase())
            //             .unwrap() as f64
            //     })
            //     .text("Phase"),
            // );

            // let active_voices_res = ui.add(
            //     Slider::from_get_set(1.0..=(MAX_VOICES as f64), |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.set_unison(new_value as usize);
            //         }

            //         synth.voices.unison() as f64
            //     })
            //     .text("Voices count")
            //     .integer(),
            // );

            // let detune_res = ui.add(
            //     Slider::from_get_set(0.0..=0.5, |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.set_detune(new_value as f32);
            //         }

            //         synth.voices.detune() as f64
            //     })
            //     .text("Detune"),
            // );

            // let blend_res = ui.add(
            //     Slider::from_get_set(0.0..=1.0, |new_value| {
            //         if let Some(new_value) = new_value {
            //             synth.voices.set_blend(new_value as f32);
            //         }

            //         synth.voices.blend() as f64
            //     })
            //     .text("Blend"),
            // );

            // model.draw_voices = active_voices_res.changed()
            //     || active_voices_res.dragged()
            //     || detune_res.hovered()
            //     || detune_res.dragged()
            //     || blend_res.dragged()
            //     || blend_res.hovered();

            // ui.label(format!("{} active voices", synth.voices.unison()));
            // ui.label(format!("{}Hz", synth.voices.freq()));
        });
}

struct AudioModel {
    synth: Arc<Mutex<Synth>>,
}

fn audio(audio: &mut AudioModel, buffer: &mut Buffer) {
    assert_eq!(buffer.sample_rate(), SAMPLE_RATE);

    let mut synth = audio.synth.lock().unwrap();

    // let volume = 0.5;
    for frame in buffer.frames_mut() {
        // let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        // audio.phase += audio.hz / sample_rate;
        // audio.phase %= sample_rate;
        // for channel in frame {
        //     *channel = sine_amp * volume;
        // }

        let sample = synth.voices.next().unwrap();
        for channel in frame {
            *channel = sample;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    const WAVE_HEIGHT: f32 = 200.0;

    let window_width = app.main_window().inner_size_points().0;
    let synth = model.synth.lock().unwrap();

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

    let row = synth.voices.voice_n(0).osc().current_row();

    draw.polyline()
        .weight(2.0)
        .points_colored((0..window_width as usize).map(|x| {
            // debug_assert!(sample >= -1.0 && sample <= 1.0, "Malformed sample {sample}");

            let y = row.lerp(x as f32 / window_width as f32);

            (pt2(x as f32 - window_width / 2.0, y * WAVE_HEIGHT), GREEN)
        }));

    let adsr_params = synth.voices.adsr();

    const ADSR_1SEC_WIDTH: f32 = 200.0;
    const ADSR_HEIGHT: f32 = 100.0;
    const ADSR_SUSTAIN_LENGTH: usize = 50;
    const ADSR_DRAW_DECIMATION: usize = 100;
    let adsr_pos = pt2(200.0, 200.0);

    let adsr_scale_ratio = ADSR_1SEC_WIDTH / SAMPLE_RATE as f32;

    let mut adsr = Adsr::new();

    adsr.note_on(1.0);
    let mut adsr_levels = (0..(adsr_params.attack() + adsr_params.decay()).inner())
        .map(|_| adsr.tick(adsr_params).unwrap())
        .step_by(ADSR_DRAW_DECIMATION)
        .collect::<Vec<_>>();
    adsr_levels.extend((0..ADSR_SUSTAIN_LENGTH).map(|_| adsr_params.sustain()));
    adsr.note_off();
    adsr_levels.extend(
        (0..adsr_params.release().inner())
            .map(|_| adsr.tick(adsr_params).unwrap())
            .step_by(ADSR_DRAW_DECIMATION),
    );

    // let adsr_length = adsr_levels.len();
    draw.polyline()
        .weight(1.0)
        .points_colored(adsr_levels.iter().enumerate().map(|(i, level)| {
            (
                pt2(
                    i as f32 * adsr_scale_ratio * ADSR_DRAW_DECIMATION as f32,
                    level * ADSR_HEIGHT,
                ) + adsr_pos,
                GOLDENROD,
            )
        }));

    let blend_height = 100.0;

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

    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}
