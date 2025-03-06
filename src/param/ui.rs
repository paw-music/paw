use crate::osc::clock::Clock;

#[derive(Clone, Copy)]
pub struct DefaultUiParams {
    pub clock: Clock,
}

#[cfg(feature = "egui")]
pub trait EguiComponent<P = DefaultUiParams, R = ()> {
    fn egui(&mut self, ui: &mut egui::Ui, params: P) -> R;
}

// TODO: Custom size
#[cfg(feature = "egui")]
pub fn egui_wave(ui: &mut egui::Ui, f: impl Fn(f32) -> f32) {
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        ui.ctx().request_repaint();

        let (_id, rect) = ui.allocate_space(egui::vec2(150.0, 100.0));

        ui.painter()
            .with_clip_rect(rect)
            .add(egui::Shape::Path(egui::epaint::PathShape::line(
                (0..rect.width() as usize)
                    .map(|index| {
                        let x = index as f32 / rect.width();
                        let y = f(x);

                        debug_assert!(
                            x >= 0.0 && x <= 1.0 && y >= -1.0 && y <= 1.0,
                            "Wave point must be in range ([0.0; 1.0], [-1.0; 1.0]). Got ({x},{y})"
                        );

                        rect.min
                            + egui::pos2(x * rect.width(), (y + 1.0) / 2.0 * rect.height())
                                .to_vec2()
                    })
                    .collect(),
                egui::Stroke::new(1.0, egui::Color32::from_gray(255)),
            )))
    });
}
