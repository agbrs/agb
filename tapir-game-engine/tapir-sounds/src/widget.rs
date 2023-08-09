use eframe::egui;

use crate::state;

mod block;
mod drop_point;
mod input;

pub use block::block;
pub use drop_point::drop_point;
pub use input::input;

impl From<state::Id> for egui::Id {
    fn from(val: state::Id) -> Self {
        egui::Id::new(val)
    }
}
