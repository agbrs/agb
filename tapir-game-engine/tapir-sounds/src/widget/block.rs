use eframe::egui;

use crate::{state, widget};

pub struct BlockResponse {
    pub alter_input: Vec<(usize, state::Input)>,
    pub delete: bool,
    pub selected: bool,
}

pub fn block(
    ctx: &egui::Context,
    block: &state::Block,
    is_selected: bool,
    display: Option<&Vec<f64>>,
) -> BlockResponse {
    let id = egui::Id::new(block.id());

    let mut alter_input = vec![];

    let response = egui::Area::new(id).show(ctx, |ui| {
        egui::Frame::popup(&ctx.style())
            .fill(if is_selected {
                egui::Color32::LIGHT_GREEN
            } else {
                ctx.style().visuals.faint_bg_color
            })
            .show(ui, |ui| {
                ui.label(block.name());

                output(ui, block.id(), display);

                let inputs = block.inputs();

                ui.vertical(|ui| {
                    for (index, (input_name, input_value)) in inputs.iter().enumerate() {
                        let response =
                            widget::input(ui, input_name, input_value, block.id(), index);

                        if let Some(change) = response {
                            alter_input.push((index, change));
                        }
                    }
                });
            });
    });

    BlockResponse {
        alter_input,
        delete: false,
        selected: response.response.double_clicked(),
    }
}

fn output(ui: &mut egui::Ui, block_id: state::Id, display: Option<&Vec<f64>>) {
    ui.horizontal(|ui| {
        egui::widgets::plot::Plot::new(egui::Id::new(block_id).with("plot"))
            .center_y_axis(true)
            .include_y(1.2)
            .include_y(-1.2)
            .auto_bounds_x()
            .clamp_grid(true)
            .width(200.0)
            .height(50.0)
            .show(ui, |plot_ui| {
                if let Some(display) = display {
                    let line = egui::widgets::plot::PlotPoints::from_ys_f64(display);
                    plot_ui.line(egui::widgets::plot::Line::new(line));
                }
            });

        widget::port(ui, block_id, 0, widget::PortDirection::Output)
    });
}
