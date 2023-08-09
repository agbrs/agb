use eframe::egui;

use crate::{state, widget};

pub fn block(ctx: &egui::Context, block: &mut state::Block) {
    egui::Area::new(block.id()).show(ctx, |ui| {
        egui::Frame::popup(&ctx.style()).show(ui, |ui| {
            ui.label(block.name());

            let inputs = block.inputs();

            ui.vertical(|ui| {
                for (input_name, input_value) in inputs {
                    if let Some(new_value) = widget::input(ui, &input_name, input_value) {
                        block.set_input(&input_name, new_value);
                    }
                }
            })
        });
    });
}
