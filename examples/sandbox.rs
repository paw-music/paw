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
    osc::Osc as _,
    voice::{Voice, VoiceStack},
    wavetable::{osc::WavetableOsc, Wavetable, WavetableRow},
};

type Sample = f32;
const WAVETABLE_DEPTH: usize = 4;
const WAVETABLE_LENGTH: usize = 2048;
const SAMPLE_RATE: u32 = 44_100;
const MAX_VOICES: usize = 8;

type GlobalWavetable = LazyLock<Wavetable<Sample, WAVETABLE_DEPTH, WAVETABLE_LENGTH>>;

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
    paused: bool,
    voices: VoiceStack<
        WavetableOsc<'static, Sample, SAMPLE_RATE, WAVETABLE_DEPTH, WAVETABLE_LENGTH>,
        MAX_VOICES,
    >,
}

fn main() {
    nannou::app(model).update(update).run();
}

struct Model {
    stream: audio::Stream<AudioModel>,
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

    let mut voices =
        VoiceStack::<_, MAX_VOICES>::new(|_| Voice::new(WavetableOsc::new(&BASIC_WAVES_TABLE)));
    voices.set_freq(440.0);
    let synth = Synth {
        paused: true,
        voices: voices,
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

    println!("Stream conf: {:?}", stream.cpal_config());

    stream.play().unwrap();

    let egui = Egui::from_window(&window);

    Model {
        stream,
        synth,
        egui,
        draw_voices: false,
    }
}

fn event(_app: &App, model: &mut Model, event: WindowEvent) {
    let mut synth = model.synth.lock().unwrap();

    match event {
        KeyPressed(key) => match key {
            Key::Space => synth.paused = !synth.paused,
            _ => {}
        },
        // KeyReleased(virtual_key_code) => todo!(),
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
            ui.add(
                Slider::from_get_set(20.0..=20_000.0, |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.set_freq(new_value as f32);
                    }
                    synth.voices.freq() as f64
                })
                .text("Frequency")
                .drag_value_speed(0.5)
                .trailing_fill(true)
                .logarithmic(true),
            );

            ui.add(
                Slider::from_get_set(0.0..=(WAVETABLE_DEPTH - 1) as f64, |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.iter_all_voices_mut().for_each(|voice| {
                            voice.osc_mut().set_depth(new_value as usize);
                        });
                    }

                    synth
                        .voices
                        .iter_active_voices()
                        .next()
                        .map(|voice| voice.osc().depth())
                        .unwrap() as f64
                })
                .integer()
                .text("Table depth"),
            );

            ui.add(
                Slider::from_get_set(0.0..=1.0, |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.iter_all_voices_mut().for_each(|voice| {
                            voice.osc_mut().set_start_phase(new_value as f32);
                        });
                    }

                    synth
                        .voices
                        .iter_active_voices()
                        .next()
                        .map(|voice| voice.osc().start_phase())
                        .unwrap() as f64
                })
                .text("Phase"),
            );

            let active_voices_res = ui.add(
                Slider::from_get_set(1.0..=(MAX_VOICES as f64), |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.set_active_voices(new_value as usize);
                    }

                    synth.voices.active_count() as f64
                })
                .text("Voices count")
                .integer(),
            );

            let detune_res = ui.add(
                Slider::from_get_set(0.0..=0.5, |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.set_detune(new_value as f32);
                    }

                    synth.voices.detune() as f64
                })
                .text("Detune"),
            );

            let blend_res = ui.add(
                Slider::from_get_set(0.0..=1.0, |new_value| {
                    if let Some(new_value) = new_value {
                        synth.voices.set_blend(new_value as f32);
                    }

                    synth.voices.blend() as f64
                })
                .text("Blend"),
            );

            model.draw_voices = active_voices_res.changed()
                || active_voices_res.dragged()
                || detune_res.hovered()
                || detune_res.dragged()
                || blend_res.dragged()
                || blend_res.hovered();

            ui.checkbox(&mut synth.paused, "Pause");

            ui.label(format!("{} active voices", synth.voices.active_count()));
            ui.label(format!("{}Hz", synth.voices.freq()));
        });
}

struct AudioModel {
    synth: Arc<Mutex<Synth>>,
}

fn audio(audio: &mut AudioModel, buffer: &mut Buffer) {
    assert_eq!(buffer.sample_rate(), SAMPLE_RATE);

    let mut synth = audio.synth.lock().unwrap();

    if synth.paused {
        return;
    }

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

    if !synth.paused {
        draw.polyline().weight(2.0).points_colored(
            synth
                .voices
                .replicate()
                .take(window_width as usize)
                .enumerate()
                .map(|(index, sample)| {
                    // debug_assert!(sample >= -1.0 && sample <= 1.0, "Malformed sample {sample}");

                    (
                        pt2(index as f32 - window_width / 2.0, sample * WAVE_HEIGHT),
                        GREEN,
                    )
                }),
        );

        let blend_height = 100.0;

        if model.draw_voices {
            synth
                .voices
                .voices_detune()
                .for_each(|(voice_detune, voice_amp)| {
                    // Get detune amount from detune factor
                    let x = (1.0 - voice_detune) * window_width;
                    let is_center = voice_detune == 1.0;
                    draw.line()
                        .start(Vec2::new(x, 0.0))
                        .end(Vec2::new(x, -blend_height * voice_amp))
                        .color(if is_center { LIGHTPINK } else { PURPLE })
                        .weight(if is_center { 2.0 } else { 1.0 });
                });
        }
    }

    draw.to_frame(app, &frame).unwrap();

    model.egui.draw_to_frame(&frame).unwrap();
}
