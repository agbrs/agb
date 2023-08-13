use std::collections::HashMap;
use std::sync::Arc;

use eframe::egui;

use crate::audio;
use crate::calculate;
use crate::state;
use crate::widget;

pub struct TapirSoundApp {
    state: state::State,
    calculator: calculate::Calculator,
    audio: Arc<audio::Audio>,

    _audio_device: Box<dyn tinyaudio::BaseAudioOutputDevice>,
}

impl TapirSoundApp {
    pub(crate) fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let audio: Arc<audio::Audio> = Default::default();
        let device = Self::start_sound(audio.clone());

        Self {
            state: Default::default(),
            audio,
            _audio_device: device,
            calculator: Default::default(),
        }
    }

    fn start_sound(audio: Arc<audio::Audio>) -> Box<dyn tinyaudio::BaseAudioOutputDevice> {
        let params = tinyaudio::OutputDeviceParameters {
            channels_count: 2,
            sample_rate: 44100,
            channel_sample_count: 441,
        };

        tinyaudio::run_output_device(params, move |data| {
            audio.play(data, params.channels_count, params.sample_rate as f64);
        })
        .unwrap()
    }

    fn update_audio(&self) {
        if let Some(selected) = self.state.selected_block().and_then(|id| {
            self.calculator
                .results()
                .and_then(|result| result.for_block(id).cloned())
        }) {
            self.audio.set_buffer(selected, self.state.frequency());
        } else {
            self.audio
                .set_buffer(Default::default(), self.state.frequency());
        }
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

            let mut should_loop = self.audio.should_loop();
            ui.checkbox(&mut should_loop, "Loop");
            self.audio.set_should_loop(should_loop);
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

        let mut selected_changed = false;

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
                    selected_changed = true;
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

        if selected_changed {
            self.update_audio();
        }

        if self.state.is_dirty() && self.calculator.calculate(&self.state) {
            self.state.clean();
            self.update_audio();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.audio.start_playing();
        }
    }
}
