use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

use crate::audio;
use crate::calculate;
use crate::save_load;
use crate::state;
use crate::widget;

pub struct TapirSoundApp {
    state: state::State,
    calculator: calculate::Calculator,
    audio: Arc<audio::Audio>,
    last_updated_audio_id: Option<calculate::CalculationId>,

    block_factory: state::BlockFactory,

    pan: egui::Vec2,

    file_path: Option<PathBuf>,

    _audio_device: Box<dyn tinyaudio::BaseAudioOutputDevice>,
}

impl TapirSoundApp {
    pub const MAX_NODE_SIZE: [f32; 2] = [200.0, 200.0];

    pub(crate) fn new(cc: &eframe::CreationContext<'_>, file_path: Option<String>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let audio: Arc<audio::Audio> = Default::default();
        let device = Self::start_sound(audio.clone());
        let block_factory = state::BlockFactory::new();

        let file_path: Option<PathBuf> = file_path.map(|path| path.into());
        let state = file_path
            .as_ref()
            .map(|path| save_load::load(path, &block_factory))
            .unwrap_or_default();

        let average_location = state.average_location();
        let pan = -egui::vec2(average_location.0, average_location.1);

        Self {
            state,
            audio,
            _audio_device: device,
            calculator: Default::default(),
            block_factory,
            pan,
            last_updated_audio_id: None,

            file_path,
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

    fn update_audio(&mut self) {
        let results = self.calculator.results();
        self.last_updated_audio_id = results.as_ref().map(|results| results.id());

        if let Some(selected) = self
            .state
            .selected_block()
            .and_then(|id| results.and_then(|result| result.for_block(id).cloned()))
        {
            self.audio.set_buffer(selected, self.state.frequency());
        } else {
            self.audio
                .set_buffer(Default::default(), self.state.frequency());
        }
    }

    fn save_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("tapir sound", &["tapir_sound"])
            .save_file()
        {
            let path = path.with_extension("tapir_sound");
            save_load::save(&self.state, &path);
            self.file_path = Some(path);
        }
    }
}

impl eframe::App for TapirSoundApp {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked()
                        || ui.input(|i| i.modifiers.command && i.key_down(egui::Key::N))
                    {
                        self.state = state::State::default();
                    }

                    if ui.button("Open").clicked() {
                        if let Some(filepath) = rfd::FileDialog::new()
                            .add_filter("tapir sound", &["tapir_sound"])
                            .pick_file()
                        {
                            self.state = save_load::load(&filepath, &self.block_factory);
                            let average_location = self.state.average_location();
                            self.pan = -egui::vec2(average_location.0, average_location.1);
                        }
                    }

                    if let Some(save_target) = &self.file_path {
                        if ui.button("Save").clicked() {
                            save_load::save(&self.state, save_target);
                        }

                        if ui.input(|i| i.modifiers.command && i.key_down(egui::Key::S)) {
                            save_load::save(&self.state, save_target);
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Save"));

                        if ui.input(|i| i.modifiers.command && i.key_down(egui::Key::S)) {
                            self.save_as();
                        }
                    }

                    if ui.button("Save as...").clicked() {
                        self.save_as();
                    }

                    ui.separator();

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

        let pan = self.pan + ctx.available_rect().size() / 2.0;

        egui::SidePanel::left("input_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Blocks");

                ui.separator();

                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    for block_type in self.block_factory.available_blocks() {
                        if ui.button(&block_type.name).clicked() {
                            let block_pos = ui.clip_rect().center() - pan;
                            self.state.add_block(
                                self.block_factory
                                    .make_block(block_type, (block_pos.x, block_pos.y)),
                            );
                        }
                    }
                });
            });

        let mut selected_changed = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            // need to allocate this first so it gets lowest priority
            let background_response =
                ui.allocate_rect(ui.min_rect(), egui::Sense::click_and_drag());

            let results = self.calculator.results();

            let selected_block = self.state.selected_block();

            let responses = self
                .state
                .blocks()
                .map(|block| {
                    let block_pos = block.pos();
                    let mut child_ui = ui.child_ui_with_id_source(
                        egui::Rect::from_min_size(
                            egui::pos2(block_pos.0, block_pos.1) + pan,
                            Self::MAX_NODE_SIZE.into(),
                        ),
                        egui::Layout::default(),
                        block.id(),
                    );

                    child_ui.set_clip_rect(ui.max_rect());

                    let block_response = widget::block(
                        &mut child_ui,
                        block,
                        selected_block == Some(block.id()),
                        results
                            .as_ref()
                            .and_then(|result| result.for_block(block.id())),
                    );

                    (block.id(), block_response)
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

                if response.drag_delta.length_sq() > 0.0 {
                    let block = self.state.get_block_mut(*id).unwrap();
                    block.pos_delta((response.drag_delta.x, response.drag_delta.y));
                }
            }

            let mut cable_ui = ui.child_ui(ui.max_rect(), *ui.layout());
            cable_ui.set_clip_rect(ui.min_rect());
            let cable_response = widget::cables(
                &mut cable_ui,
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

            if background_response.dragged() && ui.ctx().input(|i| i.pointer.middle_down()) {
                self.pan += ui.ctx().input(|i| i.pointer.delta());
            }
        });

        if selected_changed {
            self.update_audio();
        }

        if self.state.is_dirty() && self.calculator.calculate(&self.state) {
            self.state.clean();
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            self.audio.toggle_playing();
        }

        let results = self.calculator.results();
        if results.map(|result| result.id()) != self.last_updated_audio_id {
            self.update_audio();
        }
    }
}
