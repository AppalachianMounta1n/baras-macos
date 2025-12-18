//! Overlay manager for handling multiple overlay windows
//!
//! This module provides a high-level interface for creating and managing
//! multiple overlay windows, each with its own content.

use crate::platform::{NativeOverlay, OverlayConfig, OverlayPlatform, PlatformError};
use crate::renderer::{Renderer, colors};
use tiny_skia::Color;

/// A managed overlay window with its own renderer
pub struct OverlayWindow {
    platform: NativeOverlay,
    renderer: Renderer,
}

impl OverlayWindow {
    /// Create a new overlay window
    pub fn new(config: OverlayConfig) -> Result<Self, PlatformError> {
        let platform = NativeOverlay::new(config)?;
        let renderer = Renderer::new();

        Ok(Self { platform, renderer })
    }

    /// Get the window width
    pub fn width(&self) -> u32 {
        self.platform.width()
    }

    /// Get the window height
    pub fn height(&self) -> u32 {
        self.platform.height()
    }

    /// Set the window position
    pub fn set_position(&mut self, x: i32, y: i32) {
        self.platform.set_position(x, y);
    }

    /// Set the window size
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.platform.set_size(width, height);
    }

    /// Enable or disable click-through mode
    pub fn set_click_through(&mut self, enabled: bool) {
        self.platform.set_click_through(enabled);
    }

    /// Clear the overlay with a color
    pub fn clear(&mut self, color: Color) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer.clear(buffer, width, height, color);
        }
    }

    /// Draw a filled rectangle
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: Color) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer
                .fill_rect(buffer, width, height, x, y, w, h, color);
        }
    }

    /// Draw a filled rounded rectangle
    pub fn fill_rounded_rect(&mut self, x: f32, y: f32, w: f32, h: f32, radius: f32, color: Color) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer
                .fill_rounded_rect(buffer, width, height, x, y, w, h, radius, color);
        }
    }

    /// Draw a rounded rectangle outline
    pub fn stroke_rounded_rect(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        stroke_width: f32,
        color: Color,
    ) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer.stroke_rounded_rect(
                buffer,
                width,
                height,
                x,
                y,
                w,
                h,
                radius,
                stroke_width,
                color,
            );
        }
    }

    /// Draw a progress bar
    pub fn draw_progress_bar(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        progress: f32,
        bg_color: Color,
        fill_color: Color,
        radius: f32,
    ) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer.draw_progress_bar(
                buffer, width, height, x, y, w, h, progress, bg_color, fill_color, radius,
            );
        }
    }

    /// Draw text at the specified position
    pub fn draw_text(&mut self, text: &str, x: f32, y: f32, font_size: f32, color: Color) {
        let width = self.platform.width();
        let height = self.platform.height();
        if let Some(buffer) = self.platform.pixel_buffer() {
            self.renderer
                .draw_text(buffer, width, height, text, x, y, font_size, color);
        }
    }

    /// Measure text dimensions
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        self.renderer.measure_text(text, font_size)
    }

    /// Commit the current frame to the screen
    pub fn commit(&mut self) {
        self.platform.commit();
    }

    /// Poll for events (non-blocking)
    /// Returns false if the window should close
    pub fn poll_events(&mut self) -> bool {
        self.platform.poll_events()
    }

    /// Check if pointer is in the resize corner
    pub fn in_resize_corner(&self) -> bool {
        self.platform.in_resize_corner()
    }

    /// Check if currently resizing
    pub fn is_resizing(&self) -> bool {
        self.platform.is_resizing()
    }

    /// Get pending resize dimensions (if resizing)
    pub fn pending_size(&self) -> Option<(u32, u32)> {
        self.platform.pending_size()
    }

    /// Run the window event loop with a render callback
    pub fn run<F>(&mut self, mut render_callback: F)
    where
        F: FnMut(&mut Self),
    {
        // We need to implement our own loop since we can't pass self through the platform
        while self.poll_events() {
            render_callback(self);
        }
    }
}

/// Entry in a DPS/HPS meter
#[derive(Debug, Clone)]
pub struct MeterEntry {
    pub name: String,
    pub value: f64,
    pub max_value: f64,
    pub color: Color,
}

/// A specialized DPS/HPS meter overlay
pub struct MeterOverlay {
    window: OverlayWindow,
    entries: Vec<MeterEntry>,
    title: String,
    bar_height: f32,
    bar_spacing: f32,
    padding: f32,
    font_size: f32,
}

impl MeterOverlay {
    /// Create a new meter overlay
    pub fn new(config: OverlayConfig, title: &str) -> Result<Self, PlatformError> {
        let window = OverlayWindow::new(config)?;

        Ok(Self {
            window,
            entries: Vec::new(),
            title: title.to_string(),
            bar_height: 20.0,
            bar_spacing: 4.0,
            padding: 8.0,
            font_size: 14.0,
        })
    }

    /// Update the meter entries
    pub fn set_entries(&mut self, entries: Vec<MeterEntry>) {
        self.entries = entries;
    }

    /// Set the title
    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    /// Render the meter
    pub fn render(&mut self) {
        let width = self.window.width() as f32;
        let height = self.window.height() as f32;

        // Clear with transparent background
        self.window.clear(colors::transparent());

        // Draw background
        self.window
            .fill_rounded_rect(0.0, 0.0, width, height, 8.0, colors::overlay_bg());

        // Draw title
        let title_y = self.padding + self.font_size;
        self.window.draw_text(
            &self.title,
            self.padding,
            title_y,
            self.font_size,
            colors::white(),
        );

        // Draw separator line
        let sep_y = title_y + self.bar_spacing + 2.0;
        self.window.fill_rect(
            self.padding,
            sep_y,
            width - self.padding * 2.0,
            1.0,
            colors::white(),
        );

        // Find max value for scaling
        let max_val = self.entries.iter().map(|e| e.max_value).fold(1.0, f64::max);

        // Draw entries
        let bar_width = width - self.padding * 2.0;
        let mut y = sep_y + self.bar_spacing + 4.0;

        for entry in &self.entries {
            let progress = (entry.value / max_val) as f32;

            // Draw bar
            self.window.draw_progress_bar(
                self.padding,
                y,
                bar_width,
                self.bar_height,
                progress,
                colors::dps_bar_bg(),
                entry.color,
                4.0,
            );

            // Draw name on the left
            let text_y = y + self.bar_height / 2.0 + self.font_size / 3.0;
            self.window.draw_text(
                &entry.name,
                self.padding + 4.0,
                text_y,
                self.font_size - 2.0,
                colors::white(),
            );

            // Draw value on the right
            let value_text = format!("{:.1}", entry.value);
            let (text_width, _) = self.window.measure_text(&value_text, self.font_size - 2.0);
            self.window.draw_text(
                &value_text,
                width - self.padding - text_width - 4.0,
                text_y,
                self.font_size - 2.0,
                colors::white(),
            );

            y += self.bar_height + self.bar_spacing;
        }

        // Draw resize indicator in bottom-right corner when pointer is there
        if self.window.in_resize_corner() || self.window.is_resizing() {
            let indicator_size = 16.0;
            let corner_x = width - indicator_size - 4.0;
            let corner_y = height - indicator_size - 4.0;

            // Draw a small triangle/grip indicator
            let highlight = if self.window.is_resizing() {
                colors::white()
            } else {
                Color::from_rgba8(255, 255, 255, 180)
            };

            // Draw diagonal lines as resize grip
            for i in 0..3 {
                let offset = i as f32 * 5.0;
                self.window.fill_rect(
                    corner_x + offset,
                    corner_y + indicator_size - 2.0,
                    indicator_size - offset,
                    2.0,
                    highlight,
                );
                self.window.fill_rect(
                    corner_x + indicator_size - 2.0,
                    corner_y + offset,
                    2.0,
                    indicator_size - offset,
                    highlight,
                );
            }
        }

        self.window.commit();
    }

    /// Poll for events
    pub fn poll_events(&mut self) -> bool {
        self.window.poll_events()
    }

    /// Get mutable access to the underlying window
    pub fn window_mut(&mut self) -> &mut OverlayWindow {
        &mut self.window
    }
}
