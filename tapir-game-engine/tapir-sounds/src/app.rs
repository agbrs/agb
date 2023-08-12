use std::iter;

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
                    for fundamental_shape_type in state::FundamentalShapeType::all() {
                        if ui.button(fundamental_shape_type.name()).clicked() {
                            self.state.blocks.push_back(state::Block::new(Box::new(
                                state::FundamentalShapeBlock::new(fundamental_shape_type),
                            )));
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let results = self.calculator.results();

            let responses = self
                .state
                .blocks
                .iter()
                .map(|block| {
                    widget::block(
                        ctx,
                        block,
                        results
                            .as_ref()
                            .and_then(|result| result.for_block(block.id())),
                    )
                })
                .collect::<Vec<_>>();

            for (i, response) in responses.iter().enumerate() {
                if !response.alter_input.is_empty() {
                    let block = self.state.blocks.get_mut(i).unwrap();
                    for (alteration_index, alteration_value) in &response.alter_input {
                        block.set_input(*alteration_index, alteration_value);
                    }
                }
            }

            widget::cables(ui, iter::empty());
        });

        if self.state.is_dirty() && self.calculator.calculate(&self.state) {
            self.state.clean();
        }
    }
}
