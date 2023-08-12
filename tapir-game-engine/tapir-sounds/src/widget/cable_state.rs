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
    pub fn from_ctx<F, T>(ctx: &egui::Context, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        ctx.data_mut(|data| f(data.get_temp_mut_or_default::<Self>(egui::Id::null())))
    }

    pub fn set_port_position(&mut self, port_id: &PortId, position: egui::Pos2) {
        self.inner
            .lock()
            .unwrap()
            .port_positions
            .insert(port_id.clone(), position);
    }

    pub fn get_port_position(&self, port_id: &PortId) -> Option<egui::Pos2> {
        self.inner
            .lock()
            .unwrap()
            .port_positions
            .get(port_id)
            .copied()
    }

    pub fn set_in_progress_cable(&mut self, port_id: &PortId) {
        self.inner.lock().unwrap().in_progress_cable = Some(port_id.clone());
    }

    pub fn in_progress_cable_pos(&self) -> Option<egui::Pos2> {
        let inner = self.inner.lock().unwrap();
        let in_progress_cable = &inner.in_progress_cable.as_ref();

        in_progress_cable
            .and_then(|port_id| inner.port_positions.get(port_id))
            .copied()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PortId {
    block_id: state::Id,
    index: usize,
    direction: super::PortDirection,
}

impl PortId {
    pub fn new(block_id: state::Id, index: usize, direction: super::PortDirection) -> Self {
        Self {
            block_id,
            index,
            direction,
        }
    }
}

#[derive(Debug, Default)]
struct CableStateInner {
    port_positions: HashMap<PortId, egui::Pos2>,
    in_progress_cable: Option<PortId>,
}
