use std::collections::HashMap;

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
                            self.state.add_block(state::Block::new(Box::new(
                                state::FundamentalShapeBlock::new(fundamental_shape_type),
                            )));
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let results = self.calculator.results();

            let selected_block = self.state.selected_block();

            let responses = self
                .state
                .blocks()
                .map(|block| {
                    (
                        block.id(),
                        widget::block(
                            ctx,
                            block,
                            selected_block == Some(block.id()),
                            results
                                .as_ref()
                                .and_then(|result| result.for_block(block.id())),
                        ),
                    )
                })
                .collect::<HashMap<_, _>>();

            for (id, response) in responses.iter() {
                if !response.alter_input.is_empty() {
                    let block = self.state.get_block_mut(*id).unwrap();
                    for (alteration_index, alteration_value) in &response.alter_input {
                        block.set_input(*alteration_index, alteration_value);
                    }
                }

                if response.selected {
                    self.state.set_selected_block(*id);
                }
            }

            let cable_response = widget::cables(
                ui,
                self.state
                    .connections()
                    .map(|(output_block_id, (input_block_id, index))| {
                        (
                            widget::PortId::new(output_block_id, 0, widget::PortDirection::Output),
                            widget::PortId::new(
                                input_block_id,
                                index,
                                widget::PortDirection::Input,
                            ),
                        )
                    }),
            );
            if let Some((output, input)) = cable_response.new_connection {
                self.state
                    .add_connection((output.block_id, (input.block_id, input.index)));
            }
        });

        if self.state.is_dirty() && self.calculator.calculate(&self.state) {
            self.state.clean();
        }
    }
}
