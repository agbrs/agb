use eframe::egui;

use crate::{state, widget};

pub struct BlockResponse {
    pub alter_input: Vec<(usize, state::Input)>,
    pub delete: bool,
    pub selected: bool,
    pub drag_delta: egui::Vec2,
}

pub fn block(
    ui: &mut egui::Ui,
    block: &state::Block,
    is_selected: bool,
    display: Option<&Vec<f64>>,
) -> BlockResponse {
    let mut alter_input = vec![];

    let mut delete = false;

    let response = draggable_block(ui, block.id(), is_selected, |ui| {
        ui.horizontal(|ui| {
            ui.label(&block.name().name);
            if ui.button("Remove").clicked() {
                delete = true;
            }
        });

        output(ui, block.id(), display);

        let inputs = block.inputs();

        ui.vertical(|ui| {
            for (index, (input_name, input_value)) in inputs.iter().enumerate() {
                let response = widget::input(ui, input_name, input_value, block.id(), index);

                if let Some(change) = response {
                    alter_input.push((index, change));
                }
            }
        });
    });

    BlockResponse {
        alter_input,
        delete,
        selected: response.response.double_clicked(),
        drag_delta: response.response.drag_delta(),
    }
}

fn output(ui: &mut egui::Ui, block_id: state::Id, display: Option<&Vec<f64>>) {
    ui.horizontal(|ui| {
        egui::widgets::plot::Plot::new(egui::Id::new(block_id).with("plot"))
            .center_y_axis(true)
            .include_y(1.2)
            .include_y(-1.2)
            .auto_bounds_x()
            .clamp_grid(true)
            .width(200.0)
            .height(50.0)
            .show(ui, |plot_ui| {
                if let Some(display) = display {
                    let line = egui::widgets::plot::PlotPoints::from_ys_f64(display);
                    plot_ui.line(egui::widgets::plot::Line::new(line));
                }
            });

        widget::port(ui, block_id, 0, widget::PortDirection::Output)
    });
}

fn draggable_block<T>(
    ui: &mut egui::Ui,
    block_id: state::Id,
    is_selected: bool,
    content: impl FnOnce(&mut egui::Ui) -> T,
) -> egui::InnerResponse<T> {
    let margin = egui::vec2(15.0, 5.0);

    // Allocate the shape ahead of time to paint below the contents
    let background_shape = ui.painter().add(egui::epaint::Shape::Noop);

    let outer_rect_bounds = ui.available_rect_before_wrap();

    #[derive(Clone)]
    struct OuterRectMemory(egui::Rect);

    let interaction_rect = ui
        .ctx()
        .memory_mut(|mem| {
            mem.data
                .get_temp::<OuterRectMemory>(block_id.into())
                .map(|stored| stored.0)
        })
        .unwrap_or(outer_rect_bounds);

    let window_response = ui.interact(
        interaction_rect,
        egui::Id::new(block_id).with("window"),
        egui::Sense::click_and_drag(),
    );

    let inner_rect = outer_rect_bounds.shrink2(margin);
    let mut child_ui = ui.child_ui(inner_rect, *ui.layout());

    let response = content(&mut child_ui);

    let outer_rect = child_ui.min_rect().expand2(margin);

    // save the outer rect to memory
    ui.ctx().memory_mut(|mem| {
        mem.data
            .insert_temp(block_id.into(), OuterRectMemory(outer_rect))
    });

    let bg_colour = ui.ctx().style().visuals.widgets.noninteractive.bg_fill;

    let bg_stroke = if is_selected {
        egui::epaint::Stroke {
            width: 2.0,
            color: catppuccin_egui::FRAPPE.green,
        }
    } else {
        ui.ctx().style().visuals.widgets.noninteractive.bg_stroke
    };

    let body_shape = egui::epaint::Shape::Rect(egui::epaint::RectShape {
        rect: outer_rect,
        rounding: egui::epaint::Rounding::same(3.0),
        fill: bg_colour,
        stroke: bg_stroke,
    });

    ui.painter().set(background_shape, body_shape);

    egui::InnerResponse {
        response: window_response,
        inner: response,
    }
}
