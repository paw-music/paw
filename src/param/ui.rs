use az::{Cast, CastFrom};
use egui::{epaint::PathShape, pos2, vec2, Slider, Stroke};

use super::f32::{HalfUnitInterval, UnitInterval};
use crate::{macros::debug_assert_unit, osc::clock::Clock, sample::time::SampleCount};

pub trait ParamUi {
    fn unit_interval(&mut self, name: &str, value: &mut UnitInterval);
    fn half_unit_interval(&mut self, name: &str, value: &mut HalfUnitInterval);
    fn sample_count(
        &mut self,
        name: &str,
        sample_count: &mut SampleCount,
        clamp: Option<(SampleCount, SampleCount)>,
        clock: &Clock,
    );
    fn freq(&mut self, name: &str, freq: &mut f32, clamp: Option<(f32, f32)>);
    fn select<'a, T: PartialEq + Clone>(
        &mut self,
        name: &str,
        value: &mut T,
        options: impl Iterator<Item = (&'a str, T)>,
    );
    fn count(&mut self, name: &str, value: &mut usize, clamp: (usize, usize));
    fn int_map<T: PartialEq + Cast<f64> + CastFrom<f64> + Clone>(
        &mut self,
        name: &str,
        clamp: (T, T),
        set: impl FnMut(Option<T>) -> T,
    );
    fn checkbox(&mut self, name: &str, checked: &mut bool);

    // Draw wave from unit points (x and y must be in range [0.0; 1.0])
    fn wave(&mut self, point: impl FnMut(f32) -> f32);
    fn lines(&mut self, points: impl Iterator<Item = ((f32, f32), (f32, f32))>);

    fn h_stack(&mut self, f: impl FnMut(&mut Self));
    fn v_stack(&mut self, f: impl FnMut(&mut Self));
}

#[cfg(feature = "egui")]
impl ParamUi for egui::Ui {
    fn unit_interval(&mut self, name: &str, value: &mut UnitInterval) {
        self.add(
            egui::Slider::from_get_set(0.0..=1.0, |new_value| {
                if let Some(new_value) = new_value {
                    *value = UnitInterval::new(new_value as f32);
                }

                value.inner() as f64
            })
            .text(name),
        );
    }

    fn half_unit_interval(&mut self, name: &str, value: &mut HalfUnitInterval) {
        self.add(
            egui::Slider::from_get_set(0.0..=0.5, |new_value| {
                if let Some(new_value) = new_value {
                    *value = HalfUnitInterval::new(new_value as f32);
                }

                value.inner() as f64
            })
            .text(name),
        );
    }

    fn sample_count(
        &mut self,
        name: &str,
        sample_count: &mut SampleCount,
        clamp: Option<(SampleCount, SampleCount)>,
        clock: &Clock,
    ) {
        let range = clamp
            .map(|clamp| clamp.0.inner() as f64..=clamp.1.inner() as f64)
            .unwrap_or_else(|| {
                // Default range for sample count is from 0 to 10 seconds
                0.0..=clock.sample_rate as f64 * 10.0
            });

        // TODO: Do we need logarithmic parameter?
        let logarithmic = range.end() - range.start() >= 1_000.0;

        self.add(
            egui::Slider::from_get_set(range, |new_value| {
                if let Some(new_value) = new_value {
                    *sample_count = SampleCount::new(new_value as u32);
                }

                sample_count.inner() as f64
            })
            .integer()
            .logarithmic(logarithmic)
            .custom_formatter(|value, _| {
                let value = value as u32;
                let millis = value * 1_000 / clock.sample_rate;

                if value == 0 {
                    format!("0")
                } else if millis == 0 {
                    format!("{}t", value)
                } else if value < clock.sample_rate {
                    format!("{}ms", millis)
                } else {
                    format!("{}s", value / clock.sample_rate)
                }
            })
            .text(name),
        );
    }

    fn freq(&mut self, name: &str, freq: &mut f32, clamp: Option<(f32, f32)>) {
        let range = clamp
            .map(|clamp| clamp.0 as f64..=clamp.1 as f64)
            .unwrap_or_else(|| 0.01..=20_000.0);

        let logarithmic = range.end() - range.start() >= 1_000.0;

        self.add(
            egui::Slider::from_get_set(range, |new_value| {
                if let Some(new_value) = new_value {
                    *freq = new_value as f32;
                }

                *freq as f64
            })
            .logarithmic(logarithmic)
            .custom_formatter(|value, _| {
                let khz = value / 1_000.0;

                if value == 0.0 {
                    format!("0Hz")
                } else if khz < 1.0 {
                    format!("{value:.2}Hz")
                } else {
                    format!("{khz:.2}kHz")
                }
            })
            .text(name),
        );
    }

    fn select<'a, T: PartialEq + Clone>(
        &mut self,
        name: &str,
        value: &mut T,
        options: impl Iterator<Item = (&'a str, T)>,
    ) {
        self.heading(name);
        options.for_each(|option| {
            self.radio_value(value, option.1.clone(), option.0);
        });
    }

    fn count(&mut self, name: &str, value: &mut usize, clamp: (usize, usize)) {
        self.add(
            egui::Slider::from_get_set(clamp.0 as f64..=clamp.1 as f64, |new_value| {
                if let Some(new_value) = new_value {
                    *value = new_value as usize;
                }

                *value as f64
            })
            .integer()
            .text(name),
        );
    }

    fn int_map<T: PartialEq + Cast<f64> + CastFrom<f64> + Clone>(
        &mut self,
        name: &str,
        clamp: (T, T),
        mut set: impl FnMut(Option<T>) -> T,
    ) {
        let range = clamp.0.cast()..=clamp.1.cast();
        self.add(
            Slider::from_get_set(range, |new_value| {
                set(new_value.map(|value| T::cast_from(value))).cast()
            })
            .text(name)
            .integer(),
        );
    }

    fn checkbox(&mut self, name: &str, checked: &mut bool) {
        self.horizontal(|ui| {
            ui.checkbox(checked, name);
        });
    }

    fn wave(&mut self, mut point: impl FnMut(f32) -> f32) {
        egui::Frame::canvas(self.style()).show(self, |ui| {
            ui.ctx().request_repaint();

            let (_id, rect) = ui.allocate_space(vec2(150.0, 100.0));

            ui.painter()
                .with_clip_rect(rect)
                .add(egui::Shape::Path(PathShape::line(
                    (0..rect.width() as usize)
                        .map(|index| {
                            let x = index as f32 / rect.width();
                            let y = point(x);

                            debug_assert!(
                            x >= 0.0 && x <= 1.0 && y >= -1.0 && y <= 1.0,
                            "Wave point must be in range ([0.0; 1.0], [-1.0; 1.0]). Got ({x},{y})"
                        );

                            rect.min
                                + pos2(x * rect.width(), (y + 1.0) / 2.0 * rect.height()).to_vec2()
                        })
                        .collect(),
                    Stroke::new(1.0, egui::Color32::from_gray(255)),
                )))
        });
    }

    fn h_stack(&mut self, f: impl FnMut(&mut Self)) {
        self.horizontal_wrapped(f);
    }

    fn v_stack(&mut self, f: impl FnMut(&mut Self)) {
        self.vertical(f);
    }

    fn lines(&mut self, lines: impl Iterator<Item = ((f32, f32), (f32, f32))>) {
        egui::Frame::canvas(self.style()).show(self, |ui| {
            ui.ctx().request_repaint();

            let (_id, rect) = ui.allocate_space(vec2(150.0, 100.0));

            ui.painter()
                .with_clip_rect(rect)
                .extend(lines.map(|((x1, y1), (x2, y2))| {
                    debug_assert_unit!(x1, y1, x2, y2);

                    egui::Shape::line_segment(
                        [
                            rect.min
                                + pos2(
                                    (x1 + 1.0) * rect.width() / 2.0,
                                    (y1 + 1.0) * rect.height() / 2.0,
                                )
                                .to_vec2(),
                            rect.min
                                + pos2(
                                    (x2 + 1.0) * rect.width() / 2.0,
                                    (y2 + 1.0) * rect.height() / 2.0,
                                )
                                .to_vec2(),
                        ],
                        Stroke::new(1.0, egui::Color32::from_gray(255)),
                    )
                }));
        });
    }
}

pub struct UiParams {
    pub clock: Clock,
}

pub trait UiComponent {
    fn ui(&mut self, ui: &mut impl ParamUi, ui_params: &UiParams);
}
