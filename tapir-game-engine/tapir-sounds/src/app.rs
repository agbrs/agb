use eframe::egui;

use crate::state;
use crate::widget;

#[derive(Default)]
pub struct TapirSoundApp {
    state: state::State,
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
                        self.state
                            .blocks
                            .push_back(state::Block::new("Sine".to_owned()));
                    }

                    if ui.button("Square").clicked() {
                        self.state
                            .blocks
                            .push_back(state::Block::new("Square".to_owned()));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |_ui| {
            for (i, block) in self.state.blocks.iter_mut().enumerate() {
                widget::block(ctx, block, i);
            }
        });
    }
}
