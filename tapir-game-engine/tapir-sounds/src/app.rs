use eframe::egui;

#[derive(Default)]
pub struct TapirSoundApp {}
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

        egui::SidePanel::left("input_panel").show(ctx, |ui| {
            ui.heading("Blocks");

            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.button("Sine");
                ui.button("Square");
                ui.button("Triangle");
                ui.button("Saw");
            });
        });
    }
}
