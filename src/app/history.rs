use crate::image::ColorGrade;
use crate::ui::pipeline::ColorEffects;

pub struct Snapshot {
    pub grade: ColorGrade,
    pub pipeline: Vec<ColorEffects>,
}

const MAX_ENTRIES: usize = 1024;
pub struct History {
    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,
}

impl History {
    // TODO: implement tree based history
    pub fn new() -> Self {
        History {
            undo_stack: Vec::with_capacity(MAX_ENTRIES),
            redo_stack: Vec::with_capacity(MAX_ENTRIES),
        }
    }

    pub fn push(&mut self, snapshot: Snapshot) {
        if self.undo_stack.len() >= MAX_ENTRIES {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(snapshot);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let prev = self.undo_stack.pop()?;
        if self.redo_stack.len() >= MAX_ENTRIES {
            self.redo_stack.remove(0);
        }
        self.redo_stack.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: Snapshot) -> Option<Snapshot> {
        let next = self.redo_stack.pop()?;
        if self.undo_stack.len() >= MAX_ENTRIES {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(current);
        Some(next)
    }
}
