use crate::{Error, Limits};
use core::cell::Cell;

pub(super) struct Budget {
    nodes: Cell<usize>,
    units: Cell<usize>,
    depth: usize,
    refused: Cell<bool>,
}

impl Budget {
    pub(super) fn new(limits: Limits) -> Self {
        Self {
            nodes: Cell::new(limits.nodes),
            units: Cell::new(limits.allocation.saturating_sub(limits.bytes) / 8),
            depth: limits.depth,
            refused: Cell::new(limits.allocation < limits.bytes),
        }
    }

    pub(super) fn enter(&self, depth: usize, units: usize) -> bool {
        self.enter_many(depth, units, 1)
    }

    pub(super) fn enter_many(&self, depth: usize, units: usize, count: usize) -> bool {
        if self.refused.get() || depth > self.depth || self.nodes.get() < count {
            self.refused.set(true);
            return false;
        }
        if !self.charge_buffer(units) {
            return false;
        }
        self.nodes.set(self.nodes.get() - count);
        true
    }

    pub(super) fn refuse(&self) {
        self.refused.set(true);
    }

    pub(super) fn charge_buffer(&self, units: usize) -> bool {
        // Preserve the reference's buffer debit even after a prior refusal.
        // Refusal remains sticky for subsequent visits and error classification.
        match self.units.get().checked_sub(units) {
            Some(remaining) => {
                self.units.set(remaining);
                true
            }
            None => {
                self.refused.set(true);
                false
            }
        }
    }

    pub(super) fn error(&self) -> Error {
        if self.refused.get() {
            Error::Limit
        } else {
            Error::Malformed
        }
    }
}
