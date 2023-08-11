use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use eframe::egui;

use crate::state;

#[derive(Clone, Debug, Default)]
pub struct CableState {
    inner: Arc<Mutex<CableStateInner>>,
}

impl CableState {
    pub fn set_port_position(
        &mut self,
        block_id: state::Id,
        index: usize,
        direction: super::PortDirection,
        position: egui::Pos2,
    ) {
        self.inner.lock().unwrap().port_positions.insert(
            PortId {
                block_id,
                index,
                direction,
            },
            position,
        );
    }

    pub fn get_port_position(
        &self,
        block_id: state::Id,
        index: usize,
        direction: super::PortDirection,
    ) -> Option<egui::Pos2> {
        self.inner
            .lock()
            .unwrap()
            .port_positions
            .get(&PortId {
                block_id,
                index,
                direction,
            })
            .copied()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct PortId {
    block_id: state::Id,
    index: usize,
    direction: super::PortDirection,
}

#[derive(Debug, Default)]
struct CableStateInner {
    port_positions: HashMap<PortId, egui::Pos2>,
}
