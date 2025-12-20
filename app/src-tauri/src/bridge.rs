use crate::overlay::{create_all_entries, OverlayCommand, OverlayType, MetricType};
use crate::service::OverlayUpdate;
use crate::SharedOverlayState;
use baras_overlay::{OverlayData, PersonalStats};
use tokio::sync::mpsc;

/// Bridge between service overlay updates and the overlay threads
pub fn spawn_overlay_bridge(
    mut rx: mpsc::Receiver<OverlayUpdate>,
    overlay_state: SharedOverlayState,
) {
    tauri::async_runtime::spawn(async move {
        while let Some(update) = rx.recv().await {
            match update {
                OverlayUpdate::MetricsUpdated(metrics) => {
                    // Create entries for all overlay types
                    let all_entries = create_all_entries(&metrics);

                    // Get running metric overlays and their channels
                    let overlay_txs: Vec<_> = {
                        let state = match overlay_state.lock() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };

                        MetricType::all()
                            .iter()
                            .filter_map(|&overlay_type| {
                                let kind = OverlayType::Metric(overlay_type);
                                state.get_tx(kind).cloned().map(|tx| (overlay_type, tx))
                            })
                            .collect()
                    };

                    // Send entries to each running overlay
                    for (overlay_type, tx) in overlay_txs {
                        if let Some(entries) = all_entries.get(&overlay_type) {
                            let _ = tx.send(OverlayCommand::UpdateData(
                                OverlayData::Metrics(entries.clone())
                            )).await;
                        }
                    }
                }
                OverlayUpdate::PersonalStatsUpdated(stats) => {
                    // Get personal overlay channel
                    let personal_tx = {
                        let state = match overlay_state.lock() {
                            Ok(s) => s,
                            Err(_) => continue,
                        };
                        state.get_personal_tx().cloned()
                    };

                    // Convert to overlay crate's PersonalStats type
                    let overlay_stats = PersonalStats {
                        encounter_time_secs: stats.encounter_time_secs,
                        encounter_count: stats.encounter_count,
                        class_discipline: stats.class_discipline,
                        apm: stats.apm,
                        dps: stats.dps,
                        edps: stats.edps,
                        total_damage: stats.total_damage,
                        hps: stats.hps,
                        ehps: stats.ehps,
                        total_healing: stats.total_healing,
                        dtps: stats.dtps,
                        edtps: stats.edtps,
                        tps: stats.tps,
                        total_threat: stats.total_threat,
                        damage_crit_pct: stats.damage_crit_pct,
                        heal_crit_pct: stats.heal_crit_pct,
                        effective_heal_pct: stats.effective_heal_pct,
                    };

                    // Send stats to personal overlay
                    if let Some(tx) = personal_tx {
                        let _ = tx.send(OverlayCommand::UpdateData(
                            OverlayData::Personal(overlay_stats)
                        )).await;
                    }
                }
                OverlayUpdate::CombatStarted => {
                    // Could show overlay or clear entries
                }
                OverlayUpdate::CombatEnded => {
                    // Could hide overlay or freeze display
                }
            }
        }
    });
}
