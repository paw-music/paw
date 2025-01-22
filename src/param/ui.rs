use super::f32::{HalfUnitInterval, UnitInterval};
use crate::{sample::time::SampleCount, value::freq::Freq};
use core::ops::RangeInclusive;

pub trait ParamUi {
    fn unit_interval(&mut self, name: &str, value: &mut UnitInterval);
    fn half_unit_interval(&mut self, name: &str, value: &mut HalfUnitInterval);
    fn sample_count<const SAMPLE_RATE: u32>(
        &mut self,
        name: &str,
        sample_count: &mut SampleCount<SAMPLE_RATE>,
        clamp: Option<(SampleCount<SAMPLE_RATE>, SampleCount<SAMPLE_RATE>)>,
    );
    fn freq(&mut self, name: &str, freq: &mut Freq, clamp: Option<(Freq, Freq)>);
    fn select<T: PartialEq + Clone>(&mut self, name: &str, value: &mut T, options: &[(&str, T)]);
    fn count(&mut self, name: &str, value: &mut usize, clamp: (usize, usize));
    // fn select(&mut self)
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

    fn sample_count<const SAMPLE_RATE: u32>(
        &mut self,
        name: &str,
        sample_count: &mut SampleCount<SAMPLE_RATE>,
        clamp: Option<(SampleCount<SAMPLE_RATE>, SampleCount<SAMPLE_RATE>)>,
    ) {
        let range = clamp
            .map(|clamp| clamp.0.inner() as f64..=clamp.1.inner() as f64)
            .unwrap_or_else(|| {
                // Default range for sample count is from 0 to 10 seconds
                0.0..=SAMPLE_RATE as f64 * 10.0
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
                let millis = value * 1_000 / SAMPLE_RATE;

                if value == 0 {
                    format!("0")
                } else if millis == 0 {
                    format!("{}t", value)
                } else if value < SAMPLE_RATE {
                    format!("{}ms", millis)
                } else {
                    format!("{}s", value / SAMPLE_RATE)
                }
            })
            .text(name),
        );
    }

    fn freq(&mut self, name: &str, freq: &mut Freq, clamp: Option<(Freq, Freq)>) {
        let range = clamp
            .map(|clamp| clamp.0.to_num()..=clamp.1.to_num())
            .unwrap_or_else(|| 0.0..=20_000.0);

        let logarithmic = range.end() - range.start() >= 1_000.0;

        self.add(
            egui::Slider::from_get_set(range, |new_value| {
                if let Some(new_value) = new_value {
                    *freq = Freq::from_num(new_value);
                }

                freq.to_num()
            })
            .logarithmic(logarithmic)
            .custom_formatter(|value, _| {
                let khz = value / 1_000.0;

                if value == 0.0 {
                    format!("0Hz")
                } else if khz < 0.0 {
                    format!("{value:.2}Hz")
                } else {
                    format!("{khz:.2}kHz")
                }
            })
            .text(name),
        );
    }

    fn select<T: PartialEq + Clone>(&mut self, name: &str, value: &mut T, options: &[(&str, T)]) {
        self.heading(name);
        options.iter().for_each(|option| {
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
}

pub trait UiComponent {
    fn ui(&mut self, ui: &mut impl ParamUi);
}
