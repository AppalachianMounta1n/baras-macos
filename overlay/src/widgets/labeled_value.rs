//! Labeled value widget for displaying key-value pairs
//!
//! Renders a label on the left and a value right-aligned on the right.

use tiny_skia::Color;

use crate::frame::OverlayFrame;
use crate::widgets::colors;

/// Offset in pixels for text drop shadow
const SHADOW_OFFSET: f32 = 1.0;

/// A row displaying a label and right-aligned value
#[derive(Debug, Clone)]
pub struct LabeledValue {
    pub label: String,
    pub value: String,
    pub label_color: Color,
    pub value_color: Color,
}

impl LabeledValue {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            label_color: colors::label_dim(),
            value_color: colors::white(),
        }
    }

    pub fn with_label_color(mut self, color: Color) -> Self {
        self.label_color = color;
        self
    }

    pub fn with_value_color(mut self, color: Color) -> Self {
        self.value_color = color;
        self
    }

    /// Render the labeled value row
    ///
    /// # Arguments
    /// * `frame` - The overlay frame to render to
    /// * `x` - Left edge x position
    /// * `y` - Baseline y position for text
    /// * `width` - Total width available
    /// * `font_size` - Font size for both label and value
    pub fn render(&self, frame: &mut OverlayFrame, x: f32, y: f32, width: f32, font_size: f32) {
        let shadow = colors::text_shadow();

        // Draw label on left (shadow + bold for readability)
        frame.draw_text_styled(
            &self.label,
            x + SHADOW_OFFSET,
            y + SHADOW_OFFSET,
            font_size,
            shadow,
            true,
            false,
        );
        frame.draw_text_styled(&self.label, x, y, font_size, self.label_color, true, false);

        // Draw value on right (right-aligned, shadow + bold for readability)
        let (text_width, _) = frame.measure_text(&self.value, font_size);
        let value_x = x + width - text_width;
        frame.draw_text_styled(
            &self.value,
            value_x + SHADOW_OFFSET,
            y + SHADOW_OFFSET,
            font_size,
            shadow,
            true,
            false,
        );
        frame.draw_text_styled(
            &self.value,
            value_x,
            y,
            font_size,
            self.value_color,
            true,
            false,
        );
    }

    /// Calculate the height this widget needs (just the line height)
    pub fn height(&self, line_height: f32) -> f32 {
        line_height
    }
}
