use eframe::egui;

use crate::state;

mod block;

pub use block::block;

impl From<state::Id> for egui::Id {
    fn from(val: state::Id) -> Self {
        egui::Id::new(val)
    }
}
