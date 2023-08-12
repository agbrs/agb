use eframe::egui;

use crate::state;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PortDirection {
    Input,
    Output,
}

pub fn port(
    ui: &mut egui::Ui,
    block_id: state::Id,
    index: usize,
    direction: PortDirection,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(ui.spacing().interact_size, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui
            .style()
            .interact_selectable(&response, response.hovered());

        let radius = rect.height() / 2.0;
        ui.painter()
            .circle(rect.center(), radius, visuals.bg_fill, visuals.fg_stroke);
    }

    let position = rect.center();

    super::CableState::from_ctx(ui.ctx(), |cable_state| {
        cable_state.set_port_position(block_id, index, direction, position)
    });

    response
}
