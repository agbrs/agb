use eframe::egui;

use crate::state;

pub fn block(ctx: &egui::Context, block: &mut state::Block, key: usize) {
    let frame_id = egui::Id::new("block").with(key);

    egui::Area::new(frame_id).show(ctx, |ui| {
        egui::Frame::popup(&ctx.style()).show(ui, |ui| {
            ui.label(&block.name);
        });
    });
}
