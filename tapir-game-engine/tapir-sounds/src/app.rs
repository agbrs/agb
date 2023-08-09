use eframe::egui;

use crate::calculate;
use crate::state;
use crate::widget;

#[derive(Default)]
pub struct TapirSoundApp {
    state: state::State,
    calculator: calculate::Calculator,
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
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if self.calculator.is_calculating() {
                ui.spinner();
            }
        });

        egui::SidePanel::left("input_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Blocks");

                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    if ui.button("Sine").clicked() {
                        self.state.blocks.push_back(state::Block::new(Box::new(
                            state::FundamentalShapeBlock::new(state::FundamentalShapeType::Sine),
                        )));
                    }

                    if ui.button("Square").clicked() {
                        self.state.blocks.push_back(state::Block::new(Box::new(
                            state::FundamentalShapeBlock::new(state::FundamentalShapeType::Square),
                        )));
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |_ui| {
            for block in self.state.blocks.iter_mut() {
                widget::block(ctx, block);
            }
        });

        if self.state.is_dirty() && self.calculator.calculate(&self.state) {
            self.state.clean();
        }
    }
}
