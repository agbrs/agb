use eframe::egui;

use crate::{
    state::{self, Input},
    widget,
};

pub struct InputResponse {
    pub change: Option<state::Input>,
    pub selected_for_connection: bool,
    pub drop_center: Option<egui::Pos2>,
}

impl InputResponse {
    fn unchanged() -> Self {
        Self {
            change: None,
            selected_for_connection: false,
            drop_center: None,
        }
    }

    fn changed(change: state::Input) -> Self {
        Self {
            change: Some(change),
            selected_for_connection: false,
            drop_center: None,
        }
    }

    fn with_drop(change: Option<state::Input>, drop_point: egui::Response) -> Self {
        Self {
            change,
            selected_for_connection: drop_point.clicked(),
            drop_center: Some(drop_point.rect.center()),
        }
    }
}

fn droppable_input(
    ui: &mut egui::Ui,
    block_id: state::Id,
    index: usize,
    f: impl FnOnce(&mut egui::Ui) -> Option<state::Input>,
) -> InputResponse {
    ui.horizontal(|ui| {
        let response = widget::port(ui, block_id, index, widget::PortDirection::Input);

        InputResponse::with_drop(f(ui), response)
    })
    .inner
}

fn drop_point_gap(ui: &mut egui::Ui) {
    ui.add_space(ui.spacing().interact_size.x + ui.spacing().item_spacing.x);
}

pub fn input(
    ui: &mut egui::Ui,
    name: &str,
    input: &state::Input,
    block_id: state::Id,
    index: usize,
) -> InputResponse {
    match input {
        state::Input::Toggle(toggled) => {
            drop_point_gap(ui);
            let mut toggled = *toggled;

            if ui.checkbox(&mut toggled, name).changed() {
                return InputResponse::changed(Input::Toggle(toggled));
            }

            InputResponse::unchanged()
        }
        state::Input::Frequency(frequency) => {
            let mut frequency = *frequency;

            droppable_input(ui, block_id, index, |ui| {
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
            let mut amplitude = *amplitude;

            droppable_input(ui, block_id, index, |ui| {
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
            let mut periods = *periods;

            ui.horizontal(|ui| {
                drop_point_gap(ui);
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
                    return InputResponse::changed(state::Input::Periods(periods));
                }

                InputResponse::unchanged()
            })
            .inner
        }
    }
}
