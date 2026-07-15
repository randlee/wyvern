//! Private browser-history stack for [`crate::WizardSession`].
//!
//! Not part of the public API — host must not import this module (ADR-0007).

use wyvern_schema::{WizardPageDescriptor, WizardStackEntry};

/// One history entry: page descriptor + opaque data.
#[derive(Debug, Clone)]
pub(crate) struct HistoryEntry {
    pub(crate) page: WizardPageDescriptor,
    pub(crate) data: serde_json::Value,
}

impl HistoryEntry {
    pub(crate) fn new(page: WizardPageDescriptor) -> Self {
        Self {
            page,
            data: serde_json::json!({}),
        }
    }

    pub(crate) fn to_stack_entry(&self) -> WizardStackEntry {
        WizardStackEntry {
            page: self.page.clone(),
            data: self.data.clone(),
        }
    }
}

/// Browser-history model: visited entries + cursor (ADR-0005).
#[derive(Debug, Clone)]
pub(crate) struct History {
    pub(crate) entries: Vec<HistoryEntry>,
    pub(crate) cursor: usize,
}

impl History {
    /// Seed with the first page at cursor 0.
    pub(crate) fn seed(first_page: WizardPageDescriptor) -> Self {
        Self {
            entries: vec![HistoryEntry::new(first_page)],
            cursor: 0,
        }
    }

    pub(crate) fn current(&self) -> &HistoryEntry {
        &self.entries[self.cursor]
    }

    /// Prior entries only (`entries[0..cursor]`), exclusive of current (REQ-0024).
    pub(crate) fn prior_stack(&self) -> Vec<WizardStackEntry> {
        self.entries[..self.cursor]
            .iter()
            .map(HistoryEntry::to_stack_entry)
            .collect()
    }
}
