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
    let port_id = super::PortId::new(block_id, index, direction);

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
        cable_state.set_port_position(&port_id, position)
    });

    if response.hovered()
        && ui
            .ctx()
            .input(|i| i.pointer.button_pressed(egui::PointerButton::Primary))
    {
        super::CableState::from_ctx(ui.ctx(), |cable_state| {
            cable_state.set_in_progress_cable(&port_id)
        });
    }

    response
}
