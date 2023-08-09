use eframe::egui;

use crate::{state, widget};

pub fn block(ctx: &egui::Context, block: &mut state::Block, display: Option<&Vec<f64>>) {
    let id = egui::Id::new(block.id());

    egui::Area::new(id).show(ctx, |ui| {
        egui::Frame::popup(&ctx.style()).show(ui, |ui| {
            ui.label(block.name());

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
                        let line: egui::widgets::plot::PlotPoints = display
                            .iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, *v])
                            .collect();
                        plot_ui.line(egui::widgets::plot::Line::new(line));
                    }
                });

            let inputs = block.inputs();

            ui.vertical(|ui| {
                for (input_name, input_value) in inputs {
                    let response = widget::input(ui, &input_name, input_value);

                    if let Some(new_value) = response.change {
                        block.set_input(&input_name, new_value);
                    }

                    if response.drag_start {
                        println!("drag started {:?} {input_name}", block.id());
                    }

                    if response.dropped {
                        println!("dropped {:?} {input_name}", block.id());
                    }
                }
            })
        });
    });
}
