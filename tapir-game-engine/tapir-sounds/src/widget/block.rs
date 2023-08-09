use eframe::egui;

use crate::state;

pub fn block(ctx: &egui::Context, block: &mut state::Block) {
    egui::Area::new(block.id).show(ctx, |ui| {
        egui::Frame::popup(&ctx.style()).show(ui, |ui| {
            ui.label(&block.name);
        });
    });
}
