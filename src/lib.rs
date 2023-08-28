//! # Usage
//! This crate provides utilities for working with the terminal.
//!
//! ## Terminal size
//!
//! ```
//! let size = terminal_utils::size().unwrap();
//! println!("The terminal is {}x{} characters.", size.width, size.height);
//! ```
//!
//! ## Raw mode
//!
//! ```
//! let raw_mode_guard = terminal_utils::enable_raw_mode().unwrap();
//! println!("Raw mode is enabled.");
//!
//! let is_raw_mode_enabled = terminal_utils::is_raw_mode_enabled().unwrap();
//! assert!(is_raw_mode_enabled);
//!
//! // Previous terminal mode is restored when the guard is dropped.
//! drop(raw_mode_guard);
//! println!("Raw mode is disabled.");
//! ```
//!
//! ## Resize signal
//! This feature is only available with the `tokio` feature. It is enabled by default.
//!
//! ```no_run
//! let mut resize_rx = terminal_utils::on_resize().unwrap();
//! tokio::spawn(async move {
//!     loop {
//!         resize_rx.changed().await.unwrap();
//!
//!         let size = resize_rx.borrow();
//!         println!("terminal size changed: {:?}", size);
//!     }
//! });
//! ```

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::io;

#[cfg(unix)]
use unix as sys;
#[cfg(windows)]
use windows as sys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,

    pub pixel_width: u16,
    pub pixel_height: u16,
}

/// Returns the size of the terminal.
pub fn size() -> Result<TerminalSize, io::Error> {
    sys::size()
}

/// Tells whether the raw mode is currently enabled.
pub fn is_raw_mode_enabled() -> Result<bool, io::Error> {
    sys::is_raw_mode_enabled()
}

/// Enables raw mode.
/// Once the returned guard is dropped, the previous mode is restored.
pub fn enable_raw_mode() -> Result<RawModeGuard, io::Error> {
    RawModeGuard::new()
}

/// Returns a receiver that receives a signal when the terminal is resized.
#[cfg(feature = "tokio")]
pub fn on_resize() -> Result<tokio::sync::watch::Receiver<TerminalSize>, io::Error> {
    let terminal_size = size()?;
    let (tx, rx) = tokio::sync::watch::channel(terminal_size);

    sys::spawn_on_resize_task(tx)?;

    Ok(rx)
}

/// A guard that restores the previous terminal mode when dropped.
pub struct RawModeGuard {
    original_state: sys::TerminalState,
}

impl RawModeGuard {
    fn new() -> Result<Self, io::Error> {
        let original_state = sys::enable_raw_mode()?;

        Ok(Self { original_state })
    }
}

impl Drop for RawModeGuard {
    /// Restores the previous mode.
    fn drop(&mut self) {
        let _ = sys::restore_mode(self.original_state);
    }
}
