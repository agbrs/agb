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

                paint_cable_curve(painter, source_pos, target_pos, cable_stroke);
            }

            if let Some((in_progress_cable_pos, in_progress_cable_id)) =
                widget::CableState::from_ctx(ui.ctx(), |state| state.in_progress_cable())
            {
                if let Some(cursor_pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                    if in_progress_cable_id.direction == widget::PortDirection::Output {
                        paint_cable_curve(painter, in_progress_cable_pos, cursor_pos, cable_stroke);
                    } else {
                        paint_cable_curve(painter, cursor_pos, in_progress_cable_pos, cable_stroke);
                    }
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

fn paint_cable_curve(
    painter: &egui::Painter,
    source_pos: egui::Pos2,
    target_pos: egui::Pos2,
    cable_stroke: egui::Stroke,
) {
    let curve = egui::epaint::CubicBezierShape::from_points_stroke(
        [
            source_pos,
            source_pos + egui::vec2(50.0, 0.0),
            target_pos - egui::vec2(50.0, 0.0),
            target_pos,
        ],
        false,
        egui::Color32::TRANSPARENT,
        cable_stroke,
    );

    painter.add(curve);
}
