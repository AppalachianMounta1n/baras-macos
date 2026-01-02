//! Data Explorer Panel Component
//!
//! Displays detailed ability breakdown and DPS analysis for encounters.
//! Uses DataFusion SQL queries over parquet files for historical data.

use dioxus::prelude::*;
use std::collections::HashSet;
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local as spawn;

use crate::api::{self, AbilityBreakdown, BreakdownMode, DataTab, EncounterTimeline, EntityBreakdown, RaidOverviewRow, TimeRange};
use crate::components::history_panel::EncounterSummary;
use crate::components::phase_timeline::PhaseTimelineFilter;

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

fn format_number(n: f64) -> String {
    let n = n as i64;
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

fn format_pct(n: f64) -> String {
    format!("{:.1}%", n)
}

fn format_duration(secs: i64) -> String {
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{}:{:02}", mins, secs)
}

/// Group encounters into sections by area (based on is_phase_start flag)
fn group_by_area(encounters: &[EncounterSummary]) -> Vec<(String, Option<String>, Vec<&EncounterSummary>)> {
    let mut sections: Vec<(String, Option<String>, Vec<&EncounterSummary>)> = Vec::new();

    for enc in encounters.iter() {
        if enc.is_phase_start || sections.is_empty() {
            sections.push((enc.area_name.clone(), enc.difficulty.clone(), vec![enc]));
        } else if let Some(section) = sections.last_mut() {
            section.2.push(enc);
        }
    }

    sections
}

// ─────────────────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct DataExplorerProps {
    /// Initial encounter index (None = show selector)
    #[props(default)]
    pub encounter_idx: Option<u32>,
}

#[component]
pub fn DataExplorerPanel(props: DataExplorerProps) -> Element {
    // Encounter selection state
    let mut encounters = use_signal(Vec::<EncounterSummary>::new);
    let mut selected_encounter = use_signal(|| props.encounter_idx);

    // Sidebar state
    let mut show_only_bosses = use_signal(|| false);
    let mut collapsed_sections = use_signal(HashSet::<String>::new);

    // Query result state
    let mut abilities = use_signal(Vec::<AbilityBreakdown>::new);
    let mut entities = use_signal(Vec::<EntityBreakdown>::new);
    let mut selected_source = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| None::<String>);

    // Entity filter: true = players/companions only, false = show all (including NPCs)
    let mut show_players_only = use_signal(|| true);

    // Timeline state
    let mut timeline = use_signal(|| None::<EncounterTimeline>);
    let mut time_range = use_signal(|| TimeRange::default());

    // Breakdown mode state (toggles for grouping)
    let mut breakdown_mode = use_signal(|| BreakdownMode::ability_only());

    // Data tab state (Damage, Healing, DamageTaken, HealingTaken)
    let mut selected_tab = use_signal(|| DataTab::Damage);

    // Overview mode - true shows raid overview (default), false shows detailed tabs
    let mut show_overview = use_signal(|| true);
    let mut overview_data = use_signal(Vec::<RaidOverviewRow>::new);

    // Load encounter list on mount
    use_effect(move || {
        spawn(async move {
            if let Some(list) = api::get_encounter_history().await {
                encounters.set(list);
            }
        });
    });

    // Listen for session updates (refresh on combat end, file load)
    use_future(move || async move {
        let closure = Closure::new(move |event: JsValue| {
            // Extract payload from event object (Tauri events have { payload: "..." } structure)
            if let Ok(payload) = js_sys::Reflect::get(&event, &JsValue::from_str("payload"))
                && let Some(event_type) = payload.as_string()
                && (event_type.contains("CombatEnded") || event_type.contains("FileLoaded"))
            {
                // Reset selection only on file load (new file invalidates old encounter indices)
                if event_type.contains("FileLoaded") {
                    selected_encounter.set(None);
                }
                spawn(async move {
                    // Refresh encounter list
                    if let Some(list) = api::get_encounter_history().await {
                        encounters.set(list);
                    }
                });
            }
        });
        api::tauri_listen("session-updated", &closure).await;
        closure.forget();
    });

    // Load data when encounter selection or tab changes
    use_effect(move || {
        let idx = *selected_encounter.read();
        let tab = *selected_tab.read();
        spawn(async move {
            // Clear previous data
            abilities.set(Vec::new());
            entities.set(Vec::new());
            overview_data.set(Vec::new());
            selected_source.set(None);
            timeline.set(None);
            time_range.set(TimeRange::default());
            error_msg.set(None);

            if idx.is_none() {
                return; // No encounter selected
            }

            loading.set(true);

            // Load timeline first (needed for time range filter and DPS calc)
            let duration = if let Some(tl) = api::query_encounter_timeline(idx).await {
                let dur = tl.duration_secs;
                time_range.set(TimeRange::full(dur));
                timeline.set(Some(tl));
                Some(dur)
            } else {
                None
            };

            // Load raid overview data
            if let Some(data) = api::query_raid_overview(idx, None, duration).await {
                overview_data.set(data);
            }

            // Load entity breakdown for current tab (no time filter on initial load)
            match api::query_entity_breakdown(tab, idx, None).await {
                Some(data) => entities.set(data),
                None => {
                    error_msg.set(Some("No data available for this encounter".to_string()));
                    loading.set(false);
                    return;
                }
            }

            // Load ability breakdown (filtered by entity type if players-only)
            let entity_filter: Option<&[&str]> = if *show_players_only.read() {
                Some(&["Player", "Companion"])
            } else {
                None
            };
            let mode = *breakdown_mode.read();
            match api::query_breakdown(tab, idx, None, None, entity_filter, Some(&mode), duration).await {
                Some(data) => abilities.set(data),
                None => error_msg.set(Some("Failed to load ability breakdown".to_string())),
            }

            loading.set(false);
        });
    });

    // Reload data when time range changes
    use_effect(move || {
        let idx = *selected_encounter.read();
        let tab = *selected_tab.read();
        let tr = time_range();
        let src = selected_source.read().clone();

        // Skip if no encounter selected or time_range is default (initial load)
        if idx.is_none() || (tr.start == 0.0 && tr.end == 0.0) {
            return;
        }

        spawn(async move {
            loading.set(true);

            let duration = timeline.read().as_ref().map(|t| t.duration_secs);

            // Reload raid overview with time filter
            if let Some(data) = api::query_raid_overview(idx, Some(&tr), duration).await {
                overview_data.set(data);
            }

            // Reload entity breakdown with time filter
            if let Some(data) = api::query_entity_breakdown(tab, idx, Some(&tr)).await {
                entities.set(data);
            }

            // Reload ability breakdown with time filter (apply entity filter if no source selected)
            let entity_filter: Option<&[&str]> = if src.is_none() && *show_players_only.read() {
                Some(&["Player", "Companion"])
            } else {
                None
            };
            let mode = *breakdown_mode.read();
            if let Some(data) = api::query_breakdown(tab, idx, src.as_deref(), Some(&tr), entity_filter, Some(&mode), duration).await {
                abilities.set(data);
            }

            loading.set(false);
        });
    });

    // Reload abilities when entity filter or breakdown mode changes (only if no source selected)
    use_effect(move || {
        let players_only = *show_players_only.read();
        let mode = *breakdown_mode.read();
        let idx = *selected_encounter.read();
        let tab = *selected_tab.read();
        let src = selected_source.read().clone();
        let tr = time_range();

        // Skip if no encounter or a specific source is selected
        if idx.is_none() || src.is_some() {
            return;
        }

        spawn(async move {
            loading.set(true);
            let entity_filter: Option<&[&str]> = if players_only {
                Some(&["Player", "Companion"])
            } else {
                None
            };
            let tr_opt = if tr.start == 0.0 && tr.end == 0.0 { None } else { Some(tr) };
            let duration = timeline.read().as_ref().map(|t| t.duration_secs);
            if let Some(data) = api::query_breakdown(tab, idx, None, tr_opt.as_ref(), entity_filter, Some(&mode), duration).await {
                abilities.set(data);
            }
            loading.set(false);
        });
    });

    // Filter by source when selected
    let mut on_source_click = move |name: String| {
        let idx = *selected_encounter.read();
        let tab = *selected_tab.read();
        let current = selected_source.read().clone();
        let tr = time_range();

        // Toggle selection
        let new_source = if current.as_ref() == Some(&name) {
            None
        } else {
            Some(name.clone())
        };

        selected_source.set(new_source.clone());

        // Use time_range if not default
        let tr_opt = if tr.start == 0.0 && tr.end == 0.0 { None } else { Some(tr) };

        spawn(async move {
            loading.set(true);
            // Apply entity filter only when no specific source is selected
            let entity_filter: Option<&[&str]> = if new_source.is_none() && *show_players_only.read() {
                Some(&["Player", "Companion"])
            } else {
                None
            };
            let mode = *breakdown_mode.read();
            let duration = timeline.read().as_ref().map(|t| t.duration_secs);
            if let Some(data) = api::query_breakdown(tab, idx, new_source.as_deref(), tr_opt.as_ref(), entity_filter, Some(&mode), duration).await {
                abilities.set(data);
            }
            loading.set(false);
        });
    };

    // Prepare data for rendering
    let history = encounters();
    let bosses_only = show_only_bosses();
    let collapsed = collapsed_sections();

    // Filter encounters based on boss-only toggle
    let filtered_history: Vec<_> = if bosses_only {
        history.iter().filter(|e| e.boss_name.is_some()).cloned().collect()
    } else {
        history.clone()
    };

    // Group encounters by area
    let sections = group_by_area(&filtered_history);

    rsx! {
        div { class: "data-explorer",
            // Sidebar with encounter list
            aside { class: "explorer-sidebar",
                div { class: "sidebar-header",
                    h3 {
                        i { class: "fa-solid fa-list" }
                        " Encounters"
                    }
                    div { class: "history-controls",
                        label { class: "boss-filter-toggle",
                            input {
                                r#type: "checkbox",
                                checked: bosses_only,
                                onchange: move |e| show_only_bosses.set(e.checked())
                            }
                            span { "Trash" }
                        }
                        span { class: "encounter-count",
                            "{filtered_history.len()}"
                            if bosses_only { " / {history.len()}" }
                        }
                    }
                }

                div { class: "sidebar-encounter-list",
                    if history.is_empty() {
                        div { class: "sidebar-empty",
                            i { class: "fa-solid fa-inbox" }
                            p { "No encounters" }
                            p { class: "hint", "Load a log file to see encounters" }
                        }
                    } else {
                        for (idx, (area_name, difficulty, area_encounters)) in sections.iter().enumerate() {
                            {
                                let section_key = format!("{}_{}", idx, area_name);
                                let is_collapsed = collapsed.contains(&section_key);
                                let section_key_toggle = section_key.clone();
                                let chevron_class = if is_collapsed { "fa-chevron-right" } else { "fa-chevron-down" };

                                rsx! {
                                    // Area header (collapsible)
                                    div {
                                        class: "sidebar-section-header",
                                        onclick: move |_| {
                                            let mut set = collapsed_sections();
                                            if set.contains(&section_key_toggle) {
                                                set.remove(&section_key_toggle);
                                            } else {
                                                set.insert(section_key_toggle.clone());
                                            }
                                            collapsed_sections.set(set);
                                        },
                                        i { class: "fa-solid {chevron_class} collapse-icon" }
                                        span { class: "section-area", "{area_name}" }
                                        if let Some(diff) = difficulty {
                                            span { class: "section-difficulty", " • {diff}" }
                                        }
                                        span { class: "section-count", " ({area_encounters.len()})" }
                                    }

                                    // Encounter items (hidden if collapsed)
                                    if !is_collapsed {
                                        for (enc_offset, enc) in area_encounters.iter().enumerate() {
                                            {
                                                // Calculate global index for this encounter
                                                let global_idx = filtered_history.iter()
                                                    .position(|e| e.encounter_id == enc.encounter_id)
                                                    .map(|i| i as u32);
                                                let enc_idx = global_idx.unwrap_or(enc_offset as u32);
                                                let is_selected = *selected_encounter.read() == Some(enc_idx);
                                                let success_class = if enc.success { "success" } else { "wipe" };

                                                rsx! {
                                                    div {
                                                        class: if is_selected { "sidebar-encounter-item selected" } else { "sidebar-encounter-item" },
                                                        onclick: move |_| selected_encounter.set(Some(enc_idx)),
                                                        div { class: "encounter-main",
                                                            span { class: "encounter-name", "{enc.display_name}" }
                                                            span { class: "result-indicator {success_class}",
                                                                if enc.success {
                                                                    i { class: "fa-solid fa-check" }
                                                                } else {
                                                                    i { class: "fa-solid fa-skull" }
                                                                }
                                                            }
                                                        }
                                                        div { class: "encounter-meta",
                                                            if let Some(time) = &enc.start_time {
                                                                span { class: "encounter-time", "{time}" }
                                                            }
                                                            span { class: "encounter-duration", "({format_duration(enc.duration_seconds)})" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Data Panel (main content area)
            div { class: "data-panel",
                if selected_encounter.read().is_none() {
                    div { class: "panel-placeholder",
                        i { class: "fa-solid fa-chart-bar" }
                        p { "Select an encounter" }
                        p { class: "hint", "Choose an encounter from the sidebar to view detailed breakdown" }
                    }
                } else {
                    // Phase timeline filter (when timeline is loaded)
                    if let Some(tl) = timeline.read().as_ref() {
                        PhaseTimelineFilter {
                            timeline: tl.clone(),
                            range: time_range(),
                            on_range_change: move |new_range: TimeRange| {
                                time_range.set(new_range);
                            }
                        }
                    }

                    // Data tab selector (Overview, Damage, Healing, Damage Taken, Healing Taken)
                    div { class: "data-tab-selector",
                        button {
                            class: if *show_overview.read() { "data-tab active" } else { "data-tab" },
                            onclick: move |_| show_overview.set(true),
                            "Overview"
                        }
                        button {
                            class: if !*show_overview.read() && *selected_tab.read() == DataTab::Damage { "data-tab active" } else { "data-tab" },
                            onclick: move |_| { show_overview.set(false); selected_tab.set(DataTab::Damage); },
                            "Damage"
                        }
                        button {
                            class: if !*show_overview.read() && *selected_tab.read() == DataTab::Healing { "data-tab active" } else { "data-tab" },
                            onclick: move |_| { show_overview.set(false); selected_tab.set(DataTab::Healing); },
                            "Healing"
                        }
                        button {
                            class: if !*show_overview.read() && *selected_tab.read() == DataTab::DamageTaken { "data-tab active" } else { "data-tab" },
                            onclick: move |_| { show_overview.set(false); selected_tab.set(DataTab::DamageTaken); },
                            "Damage Taken"
                        }
                        button {
                            class: if !*show_overview.read() && *selected_tab.read() == DataTab::HealingTaken { "data-tab active" } else { "data-tab" },
                            onclick: move |_| { show_overview.set(false); selected_tab.set(DataTab::HealingTaken); },
                            "Healing Taken"
                        }
                    }

                    // Error display
                    if let Some(err) = error_msg.read().as_ref() {
                        div { class: "error-message", "{err}" }
                    }

                    // Content area - Overview or Detailed view
                    if *show_overview.read() {
                        // Raid Overview Table
                        div { class: "overview-section",
                            {
                                // Filter to only show Players/Companions
                                let rows: Vec<_> = overview_data.read().iter()
                                    .filter(|r| r.entity_type == "Player" || r.entity_type == "Companion")
                                    .cloned()
                                    .collect();
                                rsx! {
                                    table { class: "overview-table",
                                        thead {
                                            tr {
                                                th { class: "name-col", "Name" }
                                                th { class: "section-header", colspan: "2", "Damage Dealt" }
                                                th { class: "section-header", colspan: "2", "Threat" }
                                                th { class: "section-header", colspan: "3", "Damage Taken" }
                                                th { class: "section-header", colspan: "4", "Healing" }
                                            }
                                            tr { class: "sub-header",
                                                th {}
                                                th { class: "num", "Total" }
                                                th { class: "num", "DPS" }
                                                th { class: "num", "Total" }
                                                th { class: "num", "TPS" }
                                                th { class: "num", "Total" }
                                                th { class: "num", "DTPS" }
                                                th { class: "num", "APS" }
                                                th { class: "num", "Total" }
                                                th { class: "num", "HPS" }
                                                th { class: "num", "%" }
                                                th { class: "num", "EHPS" }
                                            }
                                        }
                                        tbody {
                                            for row in rows.iter() {
                                                tr {
                                                    td { class: "name-col", "{row.name}" }
                                                    td { class: "num dmg", "{format_number(row.damage_total)}" }
                                                    td { class: "num dmg", "{format_number(row.dps)}" }
                                                    td { class: "num threat", "{format_number(row.threat_total)}" }
                                                    td { class: "num threat", "{format_number(row.tps)}" }
                                                    td { class: "num taken", "{format_number(row.damage_taken_total)}" }
                                                    td { class: "num taken", "{format_number(row.dtps)}" }
                                                    td { class: "num taken", "{format_number(row.aps)}" }
                                                    td { class: "num heal", "{format_number(row.healing_total)}" }
                                                    td { class: "num heal", "{format_number(row.hps)}" }
                                                    td { class: "num heal", "{format_pct(row.healing_pct)}" }
                                                    td { class: "num heal", "{format_number(row.ehps)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        // Two-column layout (Detailed breakdown)
                        div { class: "explorer-content",
                            // Entity breakdown (source filter for outgoing, target filter for incoming)
                            div { class: "entity-section",
                                div { class: "entity-header",
                                    h4 {
                                        if selected_tab.read().is_outgoing() { "Sources" } else { "Targets" }
                                    }
                                    div { class: "entity-filter-tabs",
                                        button {
                                            class: if *show_players_only.read() { "filter-tab active" } else { "filter-tab" },
                                            onclick: move |_| show_players_only.set(true),
                                            "Players"
                                        }
                                        button {
                                            class: if !*show_players_only.read() { "filter-tab active" } else { "filter-tab" },
                                            onclick: move |_| show_players_only.set(false),
                                            "All"
                                        }
                                    }
                                }
                                div { class: "entity-list",
                                    {
                                        let players_only = *show_players_only.read();
                                        let entity_list: Vec<_> = entities.read().iter()
                                            .filter(|e| !players_only || e.entity_type == "Player" || e.entity_type == "Companion")
                                            .cloned()
                                            .collect();
                                        rsx! {
                                            for entity in entity_list.iter() {
                                                {
                                                    let name = entity.source_name.clone();
                                                    let is_selected = selected_source.read().as_ref() == Some(&name);
                                                    let is_npc = entity.entity_type == "Npc";
                                                    rsx! {
                                                        div {
                                                            class: if is_selected { "entity-row selected" } else if is_npc { "entity-row npc" } else { "entity-row" },
                                                            onclick: {
                                                                let name = name.clone();
                                                                move |_| on_source_click(name.clone())
                                                            },
                                                            span { class: "entity-name", "{entity.source_name}" }
                                                            span { class: "entity-value", "{format_number(entity.total_value)}" }
                                                            span { class: "entity-abilities", "{entity.abilities_used} abilities" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Ability breakdown table
                            div { class: "ability-section",
                                // Header with title and breakdown controls
                                div { class: "ability-header",
                                    h4 {
                                        if let Some(src) = selected_source.read().as_ref() {
                                            "Abilities - {src}"
                                        } else {
                                            "All Abilities"
                                        }
                                    }
                                // Breakdown mode toggles (nested hierarchy)
                                // Labels change based on tab: outgoing uses "Target", incoming uses "Source"
                                {
                                    let is_outgoing = selected_tab.read().is_outgoing();
                                    let type_label = if is_outgoing { "Target type" } else { "Source type" };
                                    let instance_label = if is_outgoing { "Target instance" } else { "Source instance" };
                                    rsx! {
                                        div { class: "breakdown-controls",
                                            span { class: "breakdown-label", "Breakdown by" }
                                            div { class: "breakdown-options",
                                                label { class: "breakdown-option primary",
                                                    input {
                                                        r#type: "checkbox",
                                                        checked: breakdown_mode.read().by_ability,
                                                        disabled: true, // Always on
                                                    }
                                                    "Ability"
                                                }
                                                div { class: "breakdown-nested",
                                                    label { class: "breakdown-option",
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: breakdown_mode.read().by_target_type,
                                                            onchange: move |e| {
                                                                let mut mode = *breakdown_mode.read();
                                                                mode.by_target_type = e.checked();
                                                                // If disabling target type, also disable target instance
                                                                if !e.checked() {
                                                                    mode.by_target_instance = false;
                                                                }
                                                                breakdown_mode.set(mode);
                                                            }
                                                        }
                                                        "{type_label}"
                                                    }
                                                    label { class: "breakdown-option nested",
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: breakdown_mode.read().by_target_instance,
                                                            disabled: !breakdown_mode.read().by_target_type,
                                                            onchange: move |e| {
                                                                let mut mode = *breakdown_mode.read();
                                                                mode.by_target_instance = e.checked();
                                                                breakdown_mode.set(mode);
                                                            }
                                                        }
                                                        "{instance_label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                }
                                // Table with dynamic columns
                                {
                                let mode = *breakdown_mode.read();
                                let tab = *selected_tab.read();
                                let show_breakdown_col = mode.by_target_type || mode.by_target_instance;
                                let breakdown_col_label = if tab.is_outgoing() { "Target" } else { "Source" };
                                let rate_label = tab.rate_label();
                                rsx! {
                                    table { class: "ability-table",
                                        thead {
                                            tr {
                                                if show_breakdown_col {
                                                    th { "{breakdown_col_label}" }
                                                }
                                                th { "Ability" }
                                                th { class: "num", "Total" }
                                                th { class: "num", "%" }
                                                th { class: "num", "{rate_label}" }
                                                th { class: "num", "Hits" }
                                                th { class: "num", "Avg" }
                                                th { class: "num", "Crit%" }
                                            }
                                        }
                                        tbody {
                                            for ability in abilities.read().iter() {
                                                tr {
                                                    if show_breakdown_col {
                                                        td { class: "target-cell",
                                                            {ability.target_name.as_deref().unwrap_or("-")}
                                                            // Show @M:SS when instance mode is on
                                                            if let Some(first_hit) = ability.target_first_hit_secs {
                                                                span { class: "target-time",
                                                                    " @{(first_hit as i32) / 60}:{(first_hit as i32) % 60:02}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    td { "{ability.ability_name}" }
                                                    td { class: "num", "{format_number(ability.total_value)}" }
                                                    td { class: "num pct-cell",
                                                        span { class: "pct-bar", style: "width: {ability.percent_of_total.min(100.0)}%;" }
                                                        span { class: "pct-text", "{format_pct(ability.percent_of_total)}" }
                                                    }
                                                    td { class: "num", "{format_number(ability.dps)}" }
                                                    td { class: "num", "{ability.hit_count}" }
                                                    td { class: "num", "{format_number(ability.avg_hit)}" }
                                                    td { class: "num", "{format_pct(ability.crit_rate)}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
