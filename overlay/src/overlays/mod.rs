//! Complete overlay implementations
//!
//! Each overlay type is a self-contained window that displays specific
//! combat information. Overlays use widgets for rendering and the platform
//! layer for window management.

mod metric;

pub use metric::{MeterEntry, MetricOverlay};
