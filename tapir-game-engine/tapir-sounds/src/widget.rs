use eframe::egui;

use crate::state;

mod block;
mod input;

pub use block::block;
pub use input::input;

impl From<state::Id> for egui::Id {
    fn from(val: state::Id) -> Self {
        egui::Id::new(val)
    }
}
