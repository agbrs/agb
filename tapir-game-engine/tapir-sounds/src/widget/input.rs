use eframe::egui;

use crate::state::{self, Input};

pub struct InputResponse {
    pub change: Option<state::Input>,
    pub dropped: bool,
    pub drag_start: bool,
    pub is_hovered: bool,
    pub drop_center: Option<egui::Pos2>,
}

impl InputResponse {
    fn unchanged() -> Self {
        Self {
            change: None,
            dropped: false,
            drag_start: false,
            drop_center: None,
            is_hovered: false,
        }
    }

    fn changed(change: state::Input) -> Self {
        Self {
            change: Some(change),
            dropped: false,
            drag_start: false,
            drop_center: None,
            is_hovered: false,
        }
    }

    fn with_drop(change: Option<state::Input>, drop_point: egui::Response) -> Self {
        Self {
            change,
            dropped: drop_point.drag_released_by(egui::PointerButton::Primary),
            drag_start: drop_point.drag_started_by(egui::PointerButton::Primary),
            drop_center: Some(drop_point.rect.center()),
            is_hovered: drop_point.hovered(),
        }
    }
}

fn drop_point(
    ui: &mut egui::Ui,
    f: impl FnOnce(&mut egui::Ui) -> Option<state::Input>,
) -> InputResponse {
    ui.horizontal(|ui| {
        let (rect, response) = ui.allocate_exact_size(
            ui.spacing().interact_size,
            egui::Sense::click_and_drag().union(egui::Sense::hover()),
        );

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            let radius = rect.height() / 2.0;
            ui.painter()
                .circle(rect.center(), radius, visuals.bg_fill, visuals.fg_stroke);
        }

        InputResponse::with_drop(f(ui), response)
    })
    .inner
}

fn drop_point_gap(ui: &mut egui::Ui) {
    ui.add_space(ui.spacing().interact_size.x);
}

pub fn input(ui: &mut egui::Ui, name: &str, input: state::Input) -> InputResponse {
    match input {
        state::Input::Toggle(toggled) => {
            drop_point_gap(ui);
            let mut toggled = toggled;

            if ui.checkbox(&mut toggled, name).changed() {
                return InputResponse::changed(Input::Toggle(toggled));
            }

            InputResponse::unchanged()
        }
        state::Input::Frequency(frequency) => {
            let mut frequency = frequency;

            drop_point(ui, |ui| {
                ui.label(name);

                if ui
                    .add(
                        egui::DragValue::new(&mut frequency)
                            .clamp_range(0..=10000)
                            .suffix("Hz"),
                    )
                    .changed()
                {
                    return Some(state::Input::Frequency(frequency));
                }

                None
            })
        }
        state::Input::Amplitude(amplitude) => {
            let mut amplitude = amplitude;

            drop_point(ui, |ui| {
                ui.label(name);
                if ui
                    .add(
                        egui::DragValue::new(&mut amplitude)
                            .clamp_range(-1..=1)
                            .speed(0.005),
                    )
                    .changed()
                {
                    return Some(state::Input::Amplitude(amplitude));
                }

                None
            })
        }
        state::Input::Periods(periods) => {
            let mut periods = periods;

            drop_point(ui, |ui| {
                ui.label(name);

                if ui
                    .add(
                        egui::DragValue::new(&mut periods)
                            .clamp_range(0..=1000)
                            .speed(0.025)
                            .max_decimals(1),
                    )
                    .changed()
                {
                    return Some(state::Input::Periods(periods));
                }

                None
            })
        }
    }
}
