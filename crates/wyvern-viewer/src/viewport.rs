//! Viewport bounds payload for page auto-size (REQ-V008 / ADR-0020).

/// Logical CSS-pixel viewport available to the dialog page.
///
/// Fields are private so callers cannot construct a zero-axis value that
/// bypasses [`ViewportBounds::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ViewportBounds {
    /// Available width in CSS pixels (non-zero).
    available_width: u32,
    /// Available height in CSS pixels (non-zero).
    available_height: u32,
}

impl ViewportBounds {
    /// Build bounds when both axes are non-zero.
    #[must_use]
    pub const fn new(available_width: u32, available_height: u32) -> Option<Self> {
        if available_width == 0 || available_height == 0 {
            return None;
        }
        Some(Self {
            available_width,
            available_height,
        })
    }

    /// Available width in CSS pixels.
    #[must_use]
    pub const fn available_width(&self) -> u32 {
        self.available_width
    }

    /// Available height in CSS pixels.
    #[must_use]
    pub const fn available_height(&self) -> u32 {
        self.available_height
    }

    /// Golden JSON object wire shape: `{ available_width, available_height }`.
    #[must_use]
    pub fn to_json_object(&self) -> String {
        format!(
            "{{\"available_width\":{},\"available_height\":{}}}",
            self.available_width, self.available_height
        )
    }

    /// Script that stores bounds and dispatches `wyvern:viewport-bounds` before page paint.
    #[must_use]
    pub fn dispatch_script(&self) -> String {
        format!(
            "(function(){{\
var d={{available_width:{w},available_height:{h}}};\
window.__wyvernViewportBounds=d;\
window.dispatchEvent(new CustomEvent('wyvern:viewport-bounds',{{detail:d}}));\
}})();",
            w = self.available_width,
            h = self.available_height
        )
    }

    /// Convert a physical monitor size + scale into logical CSS-pixel bounds.
    #[must_use]
    pub fn from_physical(width_px: u32, height_px: u32, scale_factor: f64) -> Option<Self> {
        let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        let w = (f64::from(width_px) / scale).round() as u32;
        let h = (f64::from(height_px) / scale).round() as u32;
        Self::new(w, h)
    }
}

/// Fallback when the OS reports no primary monitor (headless / CI).
pub const FALLBACK_VIEWPORT: ViewportBounds = ViewportBounds {
    available_width: 1920,
    available_height: 1080,
};

/// Hidden-until-first-resize presentation gate (REQ-V008 / AC3).
///
/// The viewer bootstrap size (320×240) must never be shown; the window stays
/// hidden until the first content `resize:` IPC is applied, then presents once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HiddenUntilResize {
    presented: bool,
}

impl HiddenUntilResize {
    /// New gate: not yet presented (window must start hidden).
    #[must_use]
    pub const fn new() -> Self {
        Self { presented: false }
    }

    /// Whether the window has already been shown after a content resize.
    #[must_use]
    pub const fn is_presented(&self) -> bool {
        self.presented
    }

    /// Bootstrap policy: window starts hidden (no 320×240 flash).
    #[must_use]
    pub const fn starts_hidden(&self) -> bool {
        !self.presented
    }

    /// Record that a content resize was applied.
    ///
    /// Returns `true` on the first call (caller must present the window);
    /// subsequent calls return `false`.
    pub fn note_content_resize(&mut self) -> bool {
        if self.presented {
            return false;
        }
        self.presented = true;
        true
    }

    /// Hide again after navigate so the next page also waits for resize.
    pub fn note_navigate(&mut self) {
        self.presented = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_bounds_rejects_zero_axes() {
        assert!(ViewportBounds::new(0, 1080).is_none());
        assert!(ViewportBounds::new(1920, 0).is_none());
        assert!(ViewportBounds::new(0, 0).is_none());
    }

    #[test]
    fn viewport_bounds_json_golden_shape() {
        let bounds = ViewportBounds::new(1920, 1080).expect("non-zero");
        assert_eq!(
            bounds.to_json_object(),
            r#"{"available_width":1920,"available_height":1080}"#
        );
    }

    #[test]
    fn viewport_bounds_from_physical_applies_scale() {
        let bounds = ViewportBounds::from_physical(3840, 2160, 2.0).expect("scaled");
        assert_eq!(bounds.available_width(), 1920);
        assert_eq!(bounds.available_height(), 1080);
    }

    #[test]
    fn dispatch_script_includes_payload_keys() {
        let script = ViewportBounds::new(1440, 900)
            .expect("non-zero")
            .dispatch_script();
        assert!(script.contains("available_width:1440"));
        assert!(script.contains("available_height:900"));
        assert!(script.contains("wyvern:viewport-bounds"));
        assert!(script.contains("__wyvernViewportBounds"));
    }

    #[test]
    fn accessors_match_constructed_axes() {
        let bounds = ViewportBounds::new(800, 600).expect("non-zero");
        assert_eq!(bounds.available_width(), 800);
        assert_eq!(bounds.available_height(), 600);
    }

    #[test]
    fn hidden_until_resize_starts_hidden_presents_once() {
        let mut gate = HiddenUntilResize::new();
        assert!(gate.starts_hidden());
        assert!(!gate.is_presented());
        assert!(gate.note_content_resize());
        assert!(gate.is_presented());
        assert!(!gate.note_content_resize());
        assert!(gate.is_presented());
    }

    #[test]
    fn hidden_until_resize_resets_on_navigate() {
        let mut gate = HiddenUntilResize::new();
        assert!(gate.note_content_resize());
        gate.note_navigate();
        assert!(gate.starts_hidden());
        assert!(gate.note_content_resize());
    }
}
