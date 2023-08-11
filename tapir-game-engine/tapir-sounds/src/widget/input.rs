use eframe::egui;

use crate::{
    state::{self, Input},
    widget,
};

fn droppable_input(
    ui: &mut egui::Ui,
    block_id: state::Id,
    index: usize,
    f: impl FnOnce(&mut egui::Ui) -> Option<state::Input>,
) -> Option<state::Input> {
    ui.horizontal(|ui| {
        widget::port(ui, block_id, index, widget::PortDirection::Input);

        f(ui)
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
) -> Option<state::Input> {
    match input {
        state::Input::Toggle(toggled) => {
            drop_point_gap(ui);
            let mut toggled = *toggled;

            if ui.checkbox(&mut toggled, name).changed() {
                return Some(Input::Toggle(toggled));
            }

            None
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
                    return Some(state::Input::Periods(periods));
                }

                None
            })
            .inner
        }
    }
}
