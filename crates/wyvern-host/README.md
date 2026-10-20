# wyvern-host

HTTP dialog host for [Wyvern](https://github.com/randlee/wyvern).

Binds a loopback server, serves the packaged dialog UI, and waits for `POST /api/result`. This crate does not open a webview itself; the CLI and `wyvern-viewer` do that.

## Documentation

<https://docs.rs/wyvern-host>
