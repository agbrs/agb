use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

use crate::audio;
use crate::calculate;
use crate::save_load;
use crate::state;
use crate::widget;

#[derive(Clone, Copy)]
enum SaveState {
    Open,
    SaveAs,
    Export,
}

pub struct TapirSoundApp {
    state: state::State,
    calculator: calculate::Calculator,
    last_updated_audio_id: Option<calculate::CalculationId>,

    block_factory: state::BlockFactory,

    pan: egui::Vec2,

    open_file_dialog: Option<(egui_file::FileDialog, SaveState)>,
    file_path: Option<PathBuf>,
    file_dirty: bool,

    audio: Arc<audio::Audio>,
    _audio_device: Box<dyn tinyaudio::BaseAudioOutputDevice>,

    midi_input_ports: midir::MidiInput,
    selected_midi_device: Option<midir::MidiInputPort>,
    midi_connection: Option<midir::MidiInputConnection<()>>,
}

impl TapirSoundApp {
    pub const MAX_NODE_SIZE: [f32; 2] = [200.0, 200.0];

    pub(crate) fn new(_cc: &eframe::CreationContext<'_>, file_path: Option<String>) -> Self {
        let audio: Arc<audio::Audio> = Default::default();
        let device = Self::start_sound(audio.clone());

        let file_path: Option<PathBuf> = file_path.map(|path| path.into());

        let mut midi_input_ports =
            midir::MidiInput::new("tapir sounds").expect("failed to create midi input");
        midi_input_ports.ignore(midir::Ignore::None);

        let mut app = Self {
            state: Default::default(),
            calculator: Default::default(),
            block_factory: state::BlockFactory::new(),
            pan: Default::default(),
            last_updated_audio_id: None,

            open_file_dialog: None,
            file_path: file_path.clone(),
            file_dirty: false,

            audio,
            _audio_device: device,

            midi_connection: None,
            selected_midi_device: None,
            midi_input_ports,
        };

        if let Some(file_path) = file_path {
            app.open(&file_path);
        }

        app
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
        let file_path = self.file_path.clone();

        self.open_file_dialog
            .get_or_insert_with(|| (Self::save_dialog(file_path), SaveState::SaveAs));
    }

    fn save(&mut self) {
        if let Some(path) = &self.file_path {
            save_load::save(&self.state, path);
            self.file_dirty = false;
        } else {
            self.save_as();
        }
    }

    fn open(&mut self, filepath: &Path) {
        self.state = save_load::load(filepath, &self.block_factory);
        let average_location = self.state.average_location();
        self.pan = -egui::vec2(average_location.0, average_location.1);
        self.file_dirty = false;
    }

    fn open_as(&mut self) {
        let file_path = self.file_path.clone();
        self.open_file_dialog
            .get_or_insert_with(|| (Self::open_dialog(file_path), SaveState::Open));
    }

    fn export_as(&mut self) {
        if self.calculator.is_calculating() {
            return;
        };

        let filepath = self.file_path.as_ref().map(|fp| fp.with_extension("wav"));
        self.open_file_dialog
            .get_or_insert_with(|| (Self::save_dialog(filepath), SaveState::Export));
    }

    fn export(&self, filepath: &Path) {
        let Some(results) = self.calculator.results() else {
            return;
        };

        let Some(data) = self
            .state
            .selected_block()
            .and_then(|id| results.for_block(id))
        else {
            return;
        };

        save_load::export(filepath, data, self.state.frequency());
    }

    fn save_dialog(path: Option<PathBuf>) -> egui_file::FileDialog {
        let mut dialog = egui_file::FileDialog::save_file(
            path.clone()
                .and_then(|path| path.parent().map(|parent| parent.to_owned())),
        );

        if let Some(path) = path {
            if let Some(filename) = path.file_name() {
                dialog = dialog.default_filename(filename.to_string_lossy());
            }
        }

        dialog.open();

        dialog
    }

    fn open_dialog(path: Option<PathBuf>) -> egui_file::FileDialog {
        let mut dialog = egui_file::FileDialog::open_file(
            path.clone()
                .and_then(|path| path.parent().map(|parent| parent.to_owned())),
        );

        if let Some(path) = path {
            if let Some(filename) = path.file_name() {
                dialog = dialog.default_filename(filename.to_string_lossy());
            }
        }
        dialog.open();

        dialog
    }

    fn midi_combo_box(&mut self, ui: &mut egui::Ui) {
        let mut display_name = self
            .selected_midi_device
            .as_ref()
            .and_then(|port| self.midi_input_ports.port_name(port).ok())
            .unwrap_or("No midi input".to_owned())
            .to_string();
        display_name.truncate(25);

        let original_selected_device = self.selected_midi_device.clone();

        egui::ComboBox::from_id_source("midi input combobox")
            .selected_text(display_name)
            .width(200.0)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.selected_midi_device, None, "No midi input");

                for in_port in self.midi_input_ports.ports() {
                    let Ok(port_name) = self.midi_input_ports.port_name(&in_port) else {
                        continue;
                    };

                    ui.selectable_value(&mut self.selected_midi_device, Some(in_port), port_name);
                }
            });

        if self.selected_midi_device != original_selected_device {
            if let Some(in_port) = &self.selected_midi_device {
                let midi_input = midir::MidiInput::new("Tapir sounds - midi input").unwrap();
                let audio = self.audio.clone();

                fn midi_to_speed(key: u8) -> f64 {
                    2.0f64.powf(((key as f64) - 69.0) / 12.0)
                }

                let mut current_note = 0u8;

                self.midi_connection = Some(
                    midi_input
                        .connect(
                            in_port,
                            "tapir-sounds-in",
                            move |_, message, _| {
                                let event = midly::live::LiveEvent::parse(message).unwrap();

                                if let midly::live::LiveEvent::Midi { message, .. } = event {
                                    match message {
                                        midly::MidiMessage::NoteOn { key, .. } => {
                                            audio.play_at_speed(midi_to_speed(key.into()));
                                            current_note = key.into();
                                        }
                                        midly::MidiMessage::NoteOff { key, .. } => {
                                            let key: u8 = key.into();
                                            if current_note == key {
                                                audio.stop_playing();
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            },
                            (),
                        )
                        .expect("Failed to connect midi port"),
                );
            } else {
                self.midi_connection = None;
            }
        }
    }

    fn file_dialog_handling(&mut self, ctx: &egui::Context) {
        let save_action = if let Some((dialog, save_state)) = &mut self.open_file_dialog {
            dialog.show(ctx);

            if dialog.selected() {
                Some((dialog.path().map(|path| path.to_owned()), *save_state))
            } else if matches!(
                dialog.state(),
                egui_file::State::Closed | egui_file::State::Cancelled
            ) {
                Some((None, *save_state))
            } else {
                None
            }
        } else {
            None
        };

        // Need to do this to make the borrow checker happy
        match save_action {
            Some((Some(path), save_state)) => {
                match save_state {
                    SaveState::Open => {
                        self.open(&path);
                        self.file_path = Some(path);
                    }
                    SaveState::SaveAs => {
                        self.file_path = Some(path.with_extension("tapir_sound"));
                        self.save();
                    }
                    SaveState::Export => {
                        self.export(&path.with_extension("wav"));
                    }
                }

                self.open_file_dialog = None;
            }
            Some((None, _)) => {
                self.open_file_dialog = None;
            }
            None => {}
        }
    }
}

impl eframe::App for TapirSoundApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked()
                        || ui.input(|i| i.modifiers.command && i.key_down(egui::Key::N))
                    {
                        self.state = state::State::default();
                        self.file_path = None;
                        self.file_dirty = false;
                        ui.close_menu();
                    }

                    if ui.button("Open").clicked() {
                        self.open_as();
                        ui.close_menu();
                    }

                    if self.file_path.is_some() {
                        if ui.button("Save").clicked() {
                            self.save();
                            ui.close_menu();
                        }
                    } else {
                        ui.add_enabled(false, egui::Button::new("Save"));
                    }

                    if ui.button("Save as...").clicked() {
                        self.save_as();
                        ui.close_menu();
                    }

                    if ui.button("Export").clicked() {
                        self.export_as();
                        ui.close_menu();
                    }

                    ui.separator();

                    if ui.button("Quit").clicked() {
                        frame.close();
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let mut should_loop = self.audio.should_loop();
                ui.checkbox(&mut should_loop, "Loop");
                self.audio.set_should_loop(should_loop);

                self.midi_combo_box(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.calculator.is_calculating() {
                        ui.spinner();
                    }
                });
            });
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

                    self.file_dirty = true;
                }

                if response.selected {
                    self.state.set_selected_block(*id);
                    selected_changed = true;

                    self.file_dirty = true;
                }

                if response.drag_delta.length_sq() > 0.0 {
                    let block = self.state.get_block_mut(*id).unwrap();
                    block.pos_delta((response.drag_delta.x, response.drag_delta.y));

                    self.file_dirty = true;
                }

                if response.delete {
                    self.state.remove_block(*id);
                    self.file_dirty = true;
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

                self.file_dirty = true;
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

        if ctx.input(|i| i.modifiers.command && i.key_down(egui::Key::S)) {
            self.save();
        }

        if ctx.input(|i| i.modifiers.command && i.key_down(egui::Key::E)) {
            self.export_as();
        }

        if ctx.input(|i| i.modifiers.command && i.key_down(egui::Key::O)) {
            self.open_as();
        }

        let results = self.calculator.results();
        if results.map(|result| result.id()) != self.last_updated_audio_id {
            self.update_audio();
        }

        if let Some(file_path) = self.file_path.as_ref().and_then(|fp| fp.file_name()) {
            let display_str = file_path.to_string_lossy();
            frame.set_window_title(&format!(
                "Tapir sounds - {display_str}{}",
                if self.file_dirty { "*" } else { "" }
            ));
        } else {
            frame.set_window_title("Tapir sounds");
        }

        self.file_dialog_handling(ctx);
    }
}
