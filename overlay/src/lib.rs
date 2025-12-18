//! Baras Overlay Library
//!
//! Cross-platform overlay rendering for combat log statistics.
//! Provides platform abstraction for Wayland, X11, Windows, and macOS,
//! with software rendering using tiny-skia and cosmic-text.

pub mod manager;
pub mod platform;
pub mod renderer;

// Re-export commonly used types
pub use manager::{MeterEntry, MeterOverlay, OverlayWindow};
pub use platform::{NativeOverlay, OverlayConfig, OverlayPlatform, PlatformError};
pub use renderer::{Renderer, colors};
