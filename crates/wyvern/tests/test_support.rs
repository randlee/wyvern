//! Shared test utilities for wyvern integration tests.

use wyvern::extensions::RequiresProbe;

/// Always reports every binary as absent from PATH.
pub struct AbsentProbe;

impl RequiresProbe for AbsentProbe {
    fn binary_on_path(&self, _name: &str) -> bool {
        false
    }
}

/// Always reports every binary as present on PATH.
pub struct PresentProbe;

impl RequiresProbe for PresentProbe {
    fn binary_on_path(&self, _name: &str) -> bool {
        true
    }
}
