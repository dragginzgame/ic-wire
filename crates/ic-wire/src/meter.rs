use crate::{Error, LimitKind, Limits};
use core::cell::Cell;

pub(super) struct Budget {
    nodes: Cell<usize>,
    units: Cell<usize>,
    depth: usize,
    refused: Cell<Option<LimitKind>>,
}

impl Budget {
    pub(super) fn new(limits: Limits) -> Self {
        Self {
            nodes: Cell::new(limits.nodes),
            units: Cell::new(limits.allocation.saturating_sub(limits.bytes) / 8),
            depth: limits.depth,
            refused: Cell::new((limits.allocation < limits.bytes).then_some(LimitKind::Allocation)),
        }
    }

    #[inline(always)]
    pub(super) fn enter(&self, depth: usize, units: usize) -> bool {
        self.enter_many(depth, units, 1)
    }

    // Keep one copy of the reason-selecting debit across codec visitors at opt-z.
    #[inline(never)]
    pub(super) fn enter_many(&self, depth: usize, units: usize, count: usize) -> bool {
        let nodes = self.nodes.get();
        let available = self.units.get();
        if self.refused.get().is_some() {
            return false;
        }
        let reason = if depth > self.depth {
            Some(LimitKind::Depth)
        } else if nodes < count {
            Some(LimitKind::Nodes)
        } else if available < units {
            Some(LimitKind::Allocation)
        } else {
            None
        };
        if reason.is_some() {
            self.refused.set(reason);
            return false;
        }
        self.units.set(available - units);
        self.nodes.set(nodes - count);

        true
    }

    pub(super) fn refuse(&self) {
        if self.refused.get().is_none() {
            self.refused.set(Some(LimitKind::Allocation));
        }
    }

    #[inline(always)]
    pub(super) fn charge_buffer(&self, units: usize) -> bool {
        // A buffer debit can succeed after a prior refusal without clearing it.
        // Refusal remains sticky for subsequent visits and error classification.
        match self.units.get().checked_sub(units) {
            Some(remaining) => {
                self.units.set(remaining);

                true
            }
            None => {
                self.refuse();

                false
            }
        }
    }

    pub(super) fn error(&self) -> Error {
        self.refused.get().map_or(Error::Malformed, Error::Limit)
    }
}
