# tracing-godot

A tiny crate that adds a [tracing-subscriber](https://crates.io/crates/tracing-subscriber) layer that records tracing events to the Godot console.

## Usage

```rust
tracing_subscriber::registry().with(tracing_godot::GodotLayer::new()).init()
```
