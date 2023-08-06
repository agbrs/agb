use eframe::egui;

use crate::widget;

#[derive(Default)]
pub struct TapirSoundApp {
    blocks: Vec<widget::Block>,
}

impl TapirSoundApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        Default::default()
    }
}

impl eframe::App for TapirSoundApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                })
            })
        });

        egui::SidePanel::left("input_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Blocks");

                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if ui.button("Sine").clicked() {
                        self.blocks
                            .push(widget::Block::new("sine", self.blocks.len()));
                    }

                    if ui.button("Square").clicked() {
                        self.blocks
                            .push(widget::Block::new("square", self.blocks.len()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            for block in &mut self.blocks {
                block.show(ctx);
            }
        });
    }
}
