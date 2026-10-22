# wyvern-schema

JSON types, validation, and error messages for [Wyvern](https://github.com/randlee/wyvern) dialog commands.

This crate is pure logic: no windowing and no I/O. Call `validate` on loaded JSON before opening a host session.

## Documentation

<https://docs.rs/wyvern-schema>

## Example

```rust
use serde_json::json;
use wyvern_schema::validate;

let command = validate(&json!({
    "type": "message",
    "title": "Hello",
    "message": "Wyvern works",
    "level": "info",
    "buttons": "ok"
}))?;
```
