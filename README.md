# terminal-utils

[![Crates.io](https://img.shields.io/crates/v/terminal-utils)](https://crates.io/crates/terminal-utils)
[![Documentation](https://docs.rs/terminal-utils/badge.svg)](https://docs.rs/terminal-utils)
[![License](https://img.shields.io/crates/l/terminal-utils)](LICENSE)

This crate provides utilities for working with terminals.

## Terminal size

```rust
let size = terminal_utils::size().unwrap();
println!("The terminal is {}x{} characters.", size.width, size.height);
```

## Raw mode

```rust
let raw_mode_guard = terminal_utils::enable_raw_mode().unwrap();
println!("Raw mode is enabled.");

let is_raw_mode_enabled = terminal_utils::is_raw_mode_enabled().unwrap();
assert!(is_raw_mode_enabled);

// Previous terminal mode is restored when the guard is dropped.
drop(raw_mode_guard);
println!("Raw mode is disabled.");
```

## Resize signal

This feature is only available with the `tokio` feature. It is enabled by default.

```rust
let mut resize_rx = terminal_utils::on_resize().unwrap();
tokio::spawn(async move {
    loop {
        resize_rx.changed().await.unwrap();

        let size = resize_rx.borrow();
        println!("terminal size changed: {:?}", size);
    }
});
```
