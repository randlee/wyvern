//! Golden wire tests for `wyvern:viewport-bounds` IPC payload (d.6).

use wyvern_viewer::{HiddenUntilResize, ViewportBounds, FALLBACK_VIEWPORT};

#[test]
fn viewport_bounds_payload_matches_golden_shape() {
    let bounds = ViewportBounds::new(1920, 1080).expect("non-zero");
    let payload = bounds.to_json_object();
    let value: serde_json::Value = serde_json::from_str(&payload).expect("json");
    assert_eq!(value["available_width"], 1920);
    assert_eq!(value["available_height"], 1080);
    assert!(value["available_width"].as_u64().unwrap() > 0);
    assert!(value["available_height"].as_u64().unwrap() > 0);
    assert_eq!(
        payload,
        r#"{"available_width":1920,"available_height":1080}"#
    );
}

#[test]
fn viewport_bounds_payload_rejects_zero() {
    assert!(ViewportBounds::new(0, 1080).is_none());
    assert!(ViewportBounds::new(1920, 0).is_none());
    assert!(ViewportBounds::new(0, 0).is_none());
}

#[test]
fn viewport_bounds_fallback_is_nonzero() {
    const {
        assert!(FALLBACK_VIEWPORT.available_width() > 0);
        assert!(FALLBACK_VIEWPORT.available_height() > 0);
    };
    let value: serde_json::Value =
        serde_json::from_str(&FALLBACK_VIEWPORT.to_json_object()).expect("json");
    assert!(value.get("available_width").is_some());
    assert!(value.get("available_height").is_some());
}

#[test]
fn viewport_bounds_dispatch_script_is_eval_ready() {
    let script = ViewportBounds::new(1280, 720)
        .expect("non-zero")
        .dispatch_script();
    assert!(script.contains("available_width:1280"));
    assert!(script.contains("available_height:720"));
    assert!(script.contains("wyvern:viewport-bounds"));
}

#[test]
fn hidden_until_first_resize_present_gate() {
    let mut gate = HiddenUntilResize::new();
    // AC3: bootstrap stays hidden — never flash 320×240 before content resize.
    assert!(gate.starts_hidden());
    assert!(!gate.is_presented());
    // First content resize IPC → present exactly once.
    assert!(gate.note_content_resize());
    assert!(gate.is_presented());
    assert!(!gate.note_content_resize());
    // Navigate resets: next page also waits for resize.
    gate.note_navigate();
    assert!(gate.starts_hidden());
    assert!(gate.note_content_resize());
}
