use eframe::egui;

use crate::state;

pub fn input(ui: &mut egui::Ui, name: &str, input: state::Input) -> Option<state::Input> {
    match input {
        state::Input::Toggle(toggled) => {
            let mut toggled = toggled;

            if ui.checkbox(&mut toggled, name).changed() {
                return Some(state::Input::Toggle(toggled));
            }
        }
        state::Input::Frequency(frequency) => {
            let mut frequency = frequency;

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
        }
        state::Input::Amplitude(amplitude) => {
            let mut amplitude = amplitude;
            if ui
                .add(egui::DragValue::new(&mut amplitude).clamp_range(0..=1))
                .changed()
            {
                return Some(state::Input::Amplitude(amplitude));
            }
        }
    }

    None
}
