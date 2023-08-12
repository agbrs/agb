use eframe::egui;

use crate::widget;

pub fn cables(ui: &mut egui::Ui, cables: impl Iterator<Item = (widget::PortId, widget::PortId)>) {
    ui.with_layer_id(
        egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("cables")),
        |ui| {
            let painter = ui.painter();

            let cable_stroke = ui.style().visuals.window_stroke();

            for (source, target) in cables {
                let Some((source_pos, target_pos)) =
                    widget::CableState::from_ctx(ui.ctx(), |state| {
                        Some((
                            state.get_port_position(&source)?,
                            state.get_port_position(&target)?,
                        ))
                    })
                else {
                    continue;
                };

                painter.line_segment([source_pos, target_pos], cable_stroke);
            }

            if let Some(in_progress_cable_pos) =
                widget::CableState::from_ctx(ui.ctx(), |state| state.in_progress_cable_pos())
            {
                if let Some(cursor_pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                    painter.line_segment([in_progress_cable_pos, cursor_pos], cable_stroke);
                }
            }

            if ui
                .ctx()
                .input(|i| i.pointer.button_clicked(egui::PointerButton::Secondary))
            {
                widget::CableState::from_ctx(ui.ctx(), |state| state.clear_in_progress_cable());
            }
        },
    );
}
