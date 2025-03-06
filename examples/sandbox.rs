use nannou::prelude::*;
use nannou_audio::{self as audio, Buffer};
use nannou_egui::Egui;
use paw::{
    midi::note::Note,
    param::{f32::UnitInterval, ui::EguiComponent as _},
};
use std::sync::{Arc, Mutex};

type Daw = paw::daw::Daw<8, 8, 4>;

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
    // Note: Need to store stream even unused for audio to play.
    stream: audio::Stream<AudioModel>,
    octave: u8,
    daw: Arc<Mutex<Daw>>,
    egui: Egui,
    /// Mapping from pressed key (identified by key id from `key_id_from_nannou_key`) to played note. This is needed when user presses a key, changes octave (or transposes) but keeps key pressed, and to "note off" this note when key is released, not to lose actual note transposition.
    pressed_keys_notes: Vec<Option<Note>>,
}

fn model(app: &App) -> Model {
    let window_id = app
        .new_window()
        .view(view)
        .size(1280, 720)
        .raw_event(raw_window_event)
        .event(event)
        .title("SANDBOX")
        .build()
        .unwrap();

    let window = app.window(window_id).unwrap();

    let audio_host = audio::Host::new();

    let daw = Daw::new();
    let daw = Arc::new(Mutex::new(daw));

    let audio_model = AudioModel {
        daw: Arc::clone(&daw),
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
        daw,
        egui,
        pressed_keys_notes: vec![None; 128],
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    let daw = &mut model.daw.lock().unwrap();

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

                daw.note_on(note, UnitInterval::MAX);

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
                    daw.note_off(prev_note, UnitInterval::MAX);
                }

                daw.note_off(
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

    let daw = &mut model.daw.lock().unwrap();

    nannou_egui::egui::Window::new("DAW")
        .auto_sized()
        .show(&ctx, |ui| {
            // synth.ui(
            //     ui,
            //     &UiParams {
            //         clock: synth.clock(),
            //     },
            // );

            daw.egui(ui, ());

            // // Mod matrix //
            // ui.vertical(|ui| {
            //     synth
            //         .env_props_mut()
            //         .iter_mut()
            //         .enumerate()
            //         .for_each(|(env_id, env)| {
            //             ui.horizontal(|ui| {
            //                 ui.label(format!("ENV {env_id}"));
            //                 ModTarget::each::<OSCS>().for_each(|target| {
            //                     ui.radio_value(&mut env.target, target, target.to_string());
            //                 });
            //             });
            //         });

            //     synth
            //         .lfo_props_mut()
            //         .iter_mut()
            //         .enumerate()
            //         .for_each(|(lfo_id, lfo)| {
            //             ui.horizontal(|ui| {
            //                 ui.label(format!("LFO {lfo_id}"));

            //                 ModTarget::each::<OSCS>().for_each(|target| {
            //                     ui.radio_value(&mut lfo.target, target, target.to_string());
            //                 });
            //             });
            //         });
            // });
        });
}

struct AudioModel {
    daw: Arc<Mutex<Daw>>,
}

fn audio(audio: &mut AudioModel, buffer: &mut Buffer) {
    let daw = &mut audio.daw.lock().unwrap();

    assert_eq!(buffer.sample_rate(), daw.clock().sample_rate);

    // let volume = 0.5;
    for frame in buffer.frames_mut() {
        // let sine_amp = (2.0 * PI * audio.phase).sin() as f32;
        // audio.phase += audio.hz / sample_rate;
        // audio.phase %= sample_rate;
        // for channel in frame {
        //     *channel = sine_amp * volume;
        // }

        let output = daw.tick_internal();
        for (channel, output) in frame.iter_mut().zip(output) {
            *channel = output;
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}
