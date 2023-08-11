use eframe::egui;

pub fn drop_point(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(ui.spacing().interact_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui
            .style()
            .interact_selectable(&response, response.hovered());

        let radius = rect.height() / 2.0;
        ui.painter()
            .circle(rect.center(), radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}
