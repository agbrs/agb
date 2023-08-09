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

            return ui
                .horizontal(|ui| {
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
                .inner;
        }
        state::Input::Amplitude(amplitude) => {
            let mut amplitude = amplitude;

            return ui
                .horizontal(|ui| {
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
                .inner;
        }
        state::Input::Periods(periods) => {
            let mut periods = periods;

            return ui
                .horizontal(|ui| {
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
                .inner;
        }
    }

    None
}
