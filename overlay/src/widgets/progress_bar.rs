//! Progress bar widget for displaying metrics

use tiny_skia::Color;

use crate::manager::OverlayWindow;
use crate::renderer::colors;

/// A horizontal progress bar with label and value
#[derive(Debug, Clone)]
pub struct ProgressBar {
    pub label: String,
    pub value: f64,
    pub max_value: f64,
    pub fill_color: Color,
    pub bg_color: Color,
    pub show_value: bool,
}

impl ProgressBar {
    pub fn new(label: impl Into<String>, value: f64, max_value: f64) -> Self {
        Self {
            label: label.into(),
            value,
            max_value,
            fill_color: colors::dps_bar_fill(),
            bg_color: colors::dps_bar_bg(),
            show_value: true,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    /// Calculate progress as 0.0-1.0
    pub fn progress(&self) -> f32 {
        if self.max_value <= 0.0 {
            0.0
        } else {
            (self.value / self.max_value).clamp(0.0, 1.0) as f32
        }
    }

    /// Render the progress bar
    pub fn render(
        &self,
        window: &mut OverlayWindow,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        font_size: f32,
        radius: f32,
    ) {
        // Draw background
        window.fill_rounded_rect(x, y, width, height, radius, self.bg_color);

        // Draw fill
        let fill_width = width * self.progress();
        if fill_width > 0.0 {
            window.fill_rounded_rect(x, y, fill_width, height, radius, self.fill_color);
        }

        // Draw label on the left
        let text_y = y + height / 2.0 + font_size / 3.0;
        let text_padding = 4.0;
        window.draw_text(&self.label, x + text_padding, text_y, font_size, colors::white());

        // Draw value on the right
        if self.show_value {
            let value_text = format!("{:.1}", self.value);
            let (text_width, _) = window.measure_text(&value_text, font_size);
            window.draw_text(
                &value_text,
                x + width - text_width - text_padding,
                text_y,
                font_size,
                colors::white(),
            );
        }
    }
}
