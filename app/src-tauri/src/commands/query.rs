//! Data query commands for the encounter explorer.
//!
//! Provides SQL-based queries over encounter data using DataFusion.

use baras_core::query::{AbilityBreakdown, BreakdownMode, DataTab, EncounterTimeline, EntityBreakdown, RaidOverviewRow, TimeRange, TimeSeriesPoint};
use tauri::State;

use crate::service::ServiceHandle;

/// Query ability breakdown for an encounter and data tab.
/// Pass encounter_idx for historical, or None for live encounter.
#[tauri::command]
pub async fn query_breakdown(
    handle: State<'_, ServiceHandle>,
    tab: DataTab,
    encounter_idx: Option<u32>,
    entity_name: Option<String>,
    time_range: Option<TimeRange>,
    entity_types: Option<Vec<String>>,
    breakdown_mode: Option<BreakdownMode>,
    duration_secs: Option<f32>,
) -> Result<Vec<AbilityBreakdown>, String> {
    handle.query_breakdown(tab, encounter_idx, entity_name, time_range, entity_types, breakdown_mode, duration_secs).await
}

/// Query damage/healing breakdown by entity for a data tab.
#[tauri::command]
pub async fn query_entity_breakdown(
    handle: State<'_, ServiceHandle>,
    tab: DataTab,
    encounter_idx: Option<u32>,
    time_range: Option<TimeRange>,
) -> Result<Vec<EntityBreakdown>, String> {
    handle.query_entity_breakdown(tab, encounter_idx, time_range).await
}

/// Query raid overview - aggregated stats per player.
#[tauri::command]
pub async fn query_raid_overview(
    handle: State<'_, ServiceHandle>,
    encounter_idx: Option<u32>,
    time_range: Option<TimeRange>,
    duration_secs: Option<f32>,
) -> Result<Vec<RaidOverviewRow>, String> {
    handle.query_raid_overview(encounter_idx, time_range, duration_secs).await
}

/// Query DPS over time with specified bucket size.
#[tauri::command]
pub async fn query_dps_over_time(
    handle: State<'_, ServiceHandle>,
    encounter_idx: Option<u32>,
    bucket_ms: i64,
    source_name: Option<String>,
    time_range: Option<TimeRange>,
) -> Result<Vec<TimeSeriesPoint>, String> {
    handle.query_dps_over_time(encounter_idx, bucket_ms, source_name, time_range).await
}

/// List available encounter parquet files.
#[tauri::command]
pub async fn list_encounter_files(
    handle: State<'_, ServiceHandle>,
) -> Result<Vec<u32>, String> {
    handle.list_encounter_files().await
}

/// Get encounter timeline with phase segments.
#[tauri::command]
pub async fn query_encounter_timeline(
    handle: State<'_, ServiceHandle>,
    encounter_idx: Option<u32>,
) -> Result<EncounterTimeline, String> {
    handle.query_encounter_timeline(encounter_idx).await
}
