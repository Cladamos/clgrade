use std::collections::VecDeque;

use crate::{
    effect::{CropArea, SliderDatas, WheelDatas},
    ui::{color_mixer::ColorMixerPart, pipeline::ColorEffects},
};

pub struct Snapshot {
    pub slider_datas: SliderDatas,
    pub wheel_datas: WheelDatas,
    pub pipeline: Vec<ColorEffects>,
    pub color_mixer: Vec<ColorMixerPart>,
    pub crop_region: Option<CropArea>,
}

const MAX_ENTRIES: usize = 1024;
pub struct History {
    undo_stack: VecDeque<Snapshot>,
    redo_stack: VecDeque<Snapshot>,
}

impl History {
    // TODO: implement tree based history
    pub fn new() -> Self {
        History {
            undo_stack: VecDeque::with_capacity(MAX_ENTRIES),
            redo_stack: VecDeque::with_capacity(MAX_ENTRIES),
        }
    }

    pub fn push(&mut self, snapshot: Snapshot) {
        if self.undo_stack.len() >= MAX_ENTRIES {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(snapshot);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let prev = self.undo_stack.pop_back()?;
        if self.redo_stack.len() >= MAX_ENTRIES {
            self.redo_stack.pop_front();
        }
        self.redo_stack.push_back(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let next = self.redo_stack.pop_back()?;
        if self.undo_stack.len() >= MAX_ENTRIES {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(current);
        Some(next)
    }
}
