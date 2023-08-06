use eframe::egui;

pub struct Block {
    name: String,
    key: usize,
}

impl Block {
    pub fn new(name: &str, key: usize) -> Self {
        Self {
            name: name.to_owned(),
            key,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::Area::new(format!("block-{}", self.key))
            .movable(true)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(egui::Color32::GREEN)
                    .show(ui, |ui| {
                        ui.label(&self.name);
                    });
            });
    }
}
