//! Reusable UI widgets for overlays
//!
//! These widgets provide building blocks for creating different overlay types.
//! Each widget knows how to render itself given an OverlayWindow.

mod progress_bar;

pub use progress_bar::ProgressBar;

use tiny_skia::Color;

/// Common widget configuration
#[derive(Debug, Clone)]
pub struct WidgetStyle {
    pub padding: f32,
    pub font_size: f32,
    pub corner_radius: f32,
    pub text_color: Color,
    pub background_color: Color,
}

impl Default for WidgetStyle {
    fn default() -> Self {
        use crate::renderer::colors;
        Self {
            padding: 8.0,
            font_size: 14.0,
            corner_radius: 4.0,
            text_color: colors::white(),
            background_color: colors::overlay_bg(),
        }
    }
}
