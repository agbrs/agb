use eframe::egui;

use crate::state;

mod block;
mod input;
mod port;

pub use block::block;
pub use input::input;
pub use port::port;

impl From<state::Id> for egui::Id {
    fn from(val: state::Id) -> Self {
        egui::Id::new(val)
    }
}
