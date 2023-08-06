use eframe::egui;

pub struct Block {
    name: String,
    area: egui::Area,
}

impl Block {
    pub fn new(name: &str, key: usize) -> Self {
        Self {
            name: name.to_owned(),
            area: egui::Area::new(format!("block-{}", key)).movable(true),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        self.area.show(ctx, |ui| {
            egui::Frame::popup(&ctx.style()).show(ui, |ui| {
                ui.label(&self.name);
            });
        });
    }
}
