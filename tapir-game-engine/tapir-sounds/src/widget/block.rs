use std::borrow::Cow;

use eframe::egui;

use crate::{state, widget};

pub struct BlockResponse {
    pub alter_input: Vec<(usize, state::Input)>,

    pub output_for_connection: bool,
    pub input_for_connection: Option<Cow<'static, str>>,

    pub delete: bool,
}

pub fn block(
    ctx: &egui::Context,
    block: &state::Block,
    display: Option<&Vec<f64>>,
) -> BlockResponse {
    let id = egui::Id::new(block.id());

    let mut alter_input = vec![];

    let mut input_for_connection = None;

    let output_response = egui::Area::new(id)
        .show(ctx, |ui| {
            egui::Frame::popup(&ctx.style())
                .show(ui, |ui| {
                    ui.label(block.name());

                    let output_response = output(ui, id, display);

                    let inputs = block.inputs();

                    ui.vertical(|ui| {
                        for (index, (input_name, input_value)) in inputs.iter().enumerate() {
                            let response = widget::input(ui, input_name, input_value);

                            if let Some(change) = response.change {
                                alter_input.push((index, change));
                            }

                            if response.selected_for_connection {
                                input_for_connection = Some(input_name.clone());
                            }
                        }
                    });

                    output_response
                })
                .inner
        })
        .inner;

    BlockResponse {
        alter_input,
        delete: false,

        input_for_connection,
        output_for_connection: output_response.selected_for_connection,
    }
}

struct OutputResponse {
    selected_for_connection: bool,
}

fn output(ui: &mut egui::Ui, id: egui::Id, display: Option<&Vec<f64>>) -> OutputResponse {
    let response = ui
        .horizontal(|ui| {
            egui::widgets::plot::Plot::new(id.with("plot"))
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

            widget::drop_point(ui)
        })
        .inner;

    OutputResponse {
        selected_for_connection: response.clicked(),
    }
}
