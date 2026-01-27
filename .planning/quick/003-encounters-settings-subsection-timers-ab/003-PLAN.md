---
quick: 003
type: execute
files_modified:
  - app/src/components/settings_panel.rs
autonomous: true
---

<objective>
Reorganize the settings panel tabs to create an ENCOUNTERS section containing Boss Health, Timers A, Timers B, and Challenges.

Purpose: Better logical grouping - encounter-related overlays should be together, separate from general overlays.
Output: Settings panel with new ENCOUNTERS section and split Timers A/Timers B tabs.
</objective>

<context>
@app/src/components/settings_panel.rs

Current layout (lines 329-362):
- GENERAL: Personal Stats, Raid Frames, Boss Health, Timers, Challenges, Alerts
- EFFECTS: Effects A, Effects B, Cooldowns, DOT Tracker
- METRICS: (metric types)

Target layout:
- GENERAL: Personal Stats, Raid Frames, Alerts
- ENCOUNTERS: Boss Health, Timers A, Timers B, Challenges
- EFFECTS: (unchanged)
- METRICS: (unchanged)

Note: The single "Timers" tab currently shows both Timers A and Timers B settings sections. Need to split into separate tabs where "Timers A" shows only Timers A settings and "Timers B" shows only Timers B settings.
</context>

<tasks>

<task type="auto">
  <name>Task 1: Reorganize tab groups and split Timers tabs</name>
  <files>app/src/components/settings_panel.rs</files>
  <action>
1. Modify the tab groups section (around lines 329-340):

Replace the GENERAL tab-group with:
```rust
div { class: "tab-group",
    span { class: "tab-group-label", "General" }
    div { class: "tab-group-buttons",
        TabButton { label: "Personal Stats", tab_key: "personal", selected_tab: selected_tab }
        TabButton { label: "Raid Frames", tab_key: "raid", selected_tab: selected_tab }
        TabButton { label: "Alerts", tab_key: "alerts", selected_tab: selected_tab }
    }
}
div { class: "tab-group",
    span { class: "tab-group-label", "Encounters" }
    div { class: "tab-group-buttons",
        TabButton { label: "Boss Health", tab_key: "boss_health", selected_tab: selected_tab }
        TabButton { label: "Timers A", tab_key: "timers_a", selected_tab: selected_tab }
        TabButton { label: "Timers B", tab_key: "timers_b", selected_tab: selected_tab }
        TabButton { label: "Challenges", tab_key: "challenges", selected_tab: selected_tab }
    }
}
```

2. Update the tab content conditions (around line 508):

Change `else if tab == "timers"` to `else if tab == "timers_a"`

Keep only the "Timers A Settings" section (lines 509-551) for this condition.

3. Add new condition for timers_b tab after the timers_a block:

```rust
} else if tab == "timers_b" {
    // Timers B Settings
    div { class: "settings-section",
        h4 { "Timers B Appearance" }
        // ... existing Timers B settings content (lines 558-591)
    }
}
```

Move the existing "Timers B Settings" div (lines 554-594) into this new condition block.
  </action>
  <verify>
- `cargo check -p baras` compiles without errors
- Settings panel loads and shows 4 tab groups: General, Encounters, Effects, Metrics
- General section has: Personal Stats, Raid Frames, Alerts
- Encounters section has: Boss Health, Timers A, Timers B, Challenges
- Clicking "Timers A" shows only Timers A settings
- Clicking "Timers B" shows only Timers B settings
  </verify>
  <done>Settings panel reorganized with ENCOUNTERS section and split Timers A/B tabs</done>
</task>

</tasks>

<verification>
- cargo check passes
- Visual inspection of settings panel shows correct tab organization
</verification>

<success_criteria>
- GENERAL section contains only: Personal Stats, Raid Frames, Alerts
- New ENCOUNTERS section contains: Boss Health, Timers A, Timers B, Challenges
- Timers A and Timers B are separate tabs with their respective settings
- All existing functionality preserved
</success_criteria>

<output>
After completion, create `.planning/quick/003-encounters-settings-subsection-timers-ab/003-SUMMARY.md`
</output>
