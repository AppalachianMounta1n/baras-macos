//! Header widget for section titles with separator lines
//!
//! Renders a title with an optional separator line below.

use tiny_skia::Color;

use crate::frame::OverlayFrame;
use crate::renderer::colors;

/// A section header with title and optional separator
#[derive(Debug, Clone)]
pub struct Header {
    pub title: String,
    pub color: Color,
    pub show_separator: bool,
}

impl Header {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            color: colors::white(),
            show_separator: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_separator(mut self, show: bool) -> Self {
        self.show_separator = show;
        self
    }

    /// Render the header and return the y position after rendering
    ///
    /// # Arguments
    /// * `frame` - The overlay frame to render to
    /// * `x` - Left edge x position
    /// * `y` - Top y position
    /// * `width` - Total width available
    /// * `font_size` - Font size for title
    /// * `spacing` - Spacing after separator
    ///
    /// # Returns
    /// The y position after the header (ready for next content)
    pub fn render(
        &self,
        frame: &mut OverlayFrame,
        x: f32,
        y: f32,
        width: f32,
        font_size: f32,
        spacing: f32,
    ) -> f32 {
        let title_y = y + font_size;
        frame.draw_text(&self.title, x, title_y, font_size, self.color);

        if self.show_separator {
            let sep_y = title_y + spacing + 2.0;
            let line_height = 0.2 * frame.scale_factor();
            frame.fill_rect(x, sep_y, width, line_height, self.color);

            sep_y + spacing + 4.0 * frame.scale_factor()
        } else {
            title_y + spacing
        }
    }

    /// Calculate the total height this header will use
    pub fn height(&self, font_size: f32, spacing: f32, scale: f32) -> f32 {
        if self.show_separator {
            font_size + spacing + 2.0 + spacing + 4.0 * scale
        } else {
            font_size + spacing
        }
    }
}

/// A footer with separator and right-aligned value (e.g., total)
#[derive(Debug, Clone)]
pub struct Footer {
    pub value: String,
    pub color: Color,
    pub show_separator: bool,
}

impl Footer {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            color: colors::white(),
            show_separator: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_separator(mut self, show: bool) -> Self {
        self.show_separator = show;
        self
    }

    /// Render the footer
    ///
    /// # Arguments
    /// * `frame` - The overlay frame to render to
    /// * `x` - Left edge x position
    /// * `y` - Top y position (where separator starts)
    /// * `width` - Total width available
    /// * `font_size` - Font size for value
    /// * `spacing` - Spacing between separator and value
    pub fn render(
        &self,
        frame: &mut OverlayFrame,
        x: f32,
        y: f32,
        width: f32,
        font_size: f32,
        spacing: f32,
    ) {
        let mut current_y = y;

        if self.show_separator {
            let line_height = 0.2 * frame.scale_factor();
            frame.fill_rect(x, current_y + 2.0, width, line_height, self.color);
            current_y += spacing;
        }

        // Draw value right-aligned
        let (text_width, _) = frame.measure_text(&self.value, font_size);
        let text_padding = 4.0 * frame.scale_factor();
        frame.draw_text(
            &self.value,
            x + width - text_width - text_padding,
            current_y + font_size,
            font_size,
            self.color,
        );
    }
}
