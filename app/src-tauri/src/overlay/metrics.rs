//! Metric entry creation helpers
//!
//! Functions for converting player metrics into overlay entries.

use std::collections::HashMap;

use baras_overlay::MeterEntry;

use super::types::MetricType;
use crate::service::PlayerMetrics;

/// Create meter entries for a specific overlay type from player metrics
pub fn create_entries_for_type(overlay_type: MetricType, metrics: &[PlayerMetrics]) -> Vec<MeterEntry> {
    let color = overlay_type.bar_color();
    let (mut values, max_value): (Vec<_>, i64) = match overlay_type {
        MetricType::Dps => {
            let max = metrics.iter().map(|m| m.dps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.dps)).collect(), max)
        }
        MetricType::EDps => {
            let max = metrics.iter().map(|m| m.edps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.edps)).collect(), max)
        }
        MetricType::Hps => {
            let max = metrics.iter().map(|m| m.hps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.hps)).collect(), max)
        }
        MetricType::EHps => {
            let max = metrics.iter().map(|m| m.ehps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.ehps)).collect(), max)
        }
        MetricType::Tps => {
            let max = metrics.iter().map(|m| m.tps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.tps)).collect(), max)
        }
        MetricType::Dtps => {
            let max = metrics.iter().map(|m| m.dtps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.dtps)).collect(), max)
        }
        MetricType::EDtps => {
            let max = metrics.iter().map(|m| m.edtps).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.edtps)).collect(), max)
        }
        MetricType::Abs => {
            let max = metrics.iter().map(|m| m.abs).max().unwrap_or(0);
            (metrics.iter().map(|m| (m.name.clone(), m.abs)).collect(), max)
        }
    };

    // Sort by metric value descending (highest first)
    values.sort_by(|a, b| b.1.cmp(&a.1));

    values
        .into_iter()
        .map(|(name, value)| MeterEntry::new(&name, value, max_value).with_color(color))
        .collect()
}

/// Create entries for all overlay types from metrics
pub fn create_all_entries(metrics: &[PlayerMetrics]) -> HashMap<MetricType, Vec<MeterEntry>> {
    let mut result = HashMap::new();
    for overlay_type in MetricType::all() {
        result.insert(*overlay_type, create_entries_for_type(*overlay_type, metrics));
    }
    result
}
