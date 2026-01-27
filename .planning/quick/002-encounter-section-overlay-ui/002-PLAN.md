---
phase: quick
plan: 002
type: execute
wave: 1
depends_on: []
files_modified:
  - app/src/app.rs
  - app/src/types.rs
  - app/src-tauri/src/commands/overlay.rs
autonomous: true

must_haves:
  truths:
    - "Overlay window shows 'Encounter' section between General and Effects"
    - "Boss Health and Challenges buttons appear under Encounter section"
    - "Timers A and Timers B buttons appear under Encounter section"
    - "General section only contains Personal Stats, Raid Frames, Alerts"
    - "Timers B button toggles TimersB overlay independently"
  artifacts:
    - path: "app/src/app.rs"
      provides: "Encounter section UI with reorganized buttons"
      contains: "Encounter"
    - path: "app/src/types.rs"
      provides: "OverlayStatus with timers_b fields"
      contains: "timers_b_enabled"
    - path: "app/src-tauri/src/commands/overlay.rs"
      provides: "Backend status response with timers_b fields"
      contains: "timers_b_enabled"
  key_links:
    - from: "app/src/app.rs"
      to: "OverlayType::TimersB"
      via: "toggle_overlay call"
      pattern: "toggle_overlay.*TimersB"
---

<objective>
Add "Encounter" section to overlay window UI, reorganizing encounter-related overlays into a dedicated section.

Purpose: Group encounter-specific overlays (Boss Health, Challenges, Timers A/B) separately from general overlays for better UI organization.

Output: Reorganized overlay window with new Encounter section containing Boss Health, Challenges, Timers A, and Timers B buttons.
</objective>

<context>
@.planning/STATE.md
@app/src/app.rs (lines 980-1090 - overlay sections)
@app/src/types.rs (OverlayStatus struct)
@app/src-tauri/src/commands/overlay.rs (get_overlay_status)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add Timers B state tracking to backend and frontend types</name>
  <files>
    app/src-tauri/src/commands/overlay.rs
    app/src/types.rs
  </files>
  <action>
  In `app/src-tauri/src/commands/overlay.rs`:
  1. Add `timers_b_running: bool` and `timers_b_enabled: bool` fields to `OverlayStatusResponse` struct (after timers_enabled, around line 28)
  2. In `get_overlay_status` function, add `timers_b_running` to the tuple extraction using `s.is_running(OverlayType::TimersB)` (around line 133)
  3. Add `let timers_b_enabled = config.overlay_settings.is_enabled("timers_b");` (after timers_enabled, around line 157)
  4. Add both fields to the `OverlayStatusResponse` return struct (after timers_enabled, around line 175)

  In `app/src/types.rs`:
  1. Add `timers_b_running: bool` and `timers_b_enabled: bool` fields to `OverlayStatus` struct (after timers_enabled, around line 79)
  </action>
  <verify>cargo check -p baras-app (no errors)</verify>
  <done>Both backend response and frontend types include timers_b_running and timers_b_enabled fields</done>
</task>

<task type="auto">
  <name>Task 2: Add Timers B signal and wire status sync in app.rs</name>
  <files>app/src/app.rs</files>
  <action>
  1. Add `let mut timers_b_enabled = use_signal(|| false);` after `timers_enabled` signal (around line 41)

  2. Add `let timers_b_on = timers_b_enabled();` after `timers_on` (around line 292)

  3. Update `any_enabled` calculation to include `|| timers_b_on` (around line 302)

  4. Pass `&mut timers_b_enabled` to all `apply_status` calls (there are multiple - search for `apply_status`)

  5. In `apply_status` function signature (around line 1920), add `timers_b_enabled: &mut Signal<bool>` parameter after `timers_enabled`

  6. In `apply_status` function body, add `timers_b_enabled.set(status.timers_b_enabled);` after the timers_enabled.set line
  </action>
  <verify>cargo check -p baras-app (no errors related to timers_b)</verify>
  <done>Timers B signal exists and syncs with backend status</done>
</task>

<task type="auto">
  <name>Task 3: Reorganize overlay sections with new Encounter section</name>
  <files>app/src/app.rs</files>
  <action>
  In the overlay controls section (around lines 979-1090), restructure as follows:

  1. GENERAL section (around line 980) - Keep only these buttons:
     - Personal Stats (existing)
     - Raid Frames (existing)
     - Alerts (move from further down in current General section)

  2. Add NEW ENCOUNTER section after General (before Effects):
     ```rust
     // Encounter overlays
     h4 { class: "subsection-title", "Encounter" }
     div { class: "overlay-grid",
         // Boss Health button (move from General)
         button {
             class: if boss_health_on { "btn btn-overlay btn-active" } else { "btn btn-overlay" },
             title: "Shows boss health bars and cast timers",
             onclick: move |_| { spawn(async move {
                 if api::toggle_overlay(OverlayType::BossHealth, boss_health_on).await {
                     boss_health_enabled.set(!boss_health_on);
                 }
             }); },
             "Boss Health"
         }
         // Challenges button (move from General)
         button {
             class: if challenges_on { "btn btn-overlay btn-active" } else { "btn btn-overlay" },
             title: "Tracks raid challenge objectives and progress",
             onclick: move |_| { spawn(async move {
                 if api::toggle_overlay(OverlayType::Challenges, challenges_on).await {
                     challenges_enabled.set(!challenges_on);
                 }
             }); },
             "Challenges"
         }
         // Timers A button (renamed from "Encounter Timers")
         button {
             class: if timers_on { "btn btn-overlay btn-active" } else { "btn btn-overlay" },
             title: "Displays encounter-specific timers and phase markers (Group A)",
             onclick: move |_| { spawn(async move {
                 if api::toggle_overlay(OverlayType::TimersA, timers_on).await {
                     timers_enabled.set(!timers_on);
                 }
             }); },
             "Timers A"
         }
         // Timers B button (NEW)
         button {
             class: if timers_b_on { "btn btn-overlay btn-active" } else { "btn btn-overlay" },
             title: "Displays encounter-specific timers and phase markers (Group B)",
             onclick: move |_| { spawn(async move {
                 if api::toggle_overlay(OverlayType::TimersB, timers_b_on).await {
                     timers_b_enabled.set(!timers_b_on);
                 }
             }); },
             "Timers B"
         }
     }
     ```

  3. EFFECTS section (existing, unchanged) - Should follow Encounter section

  4. METRICS section (existing, unchanged)

  5. BEHAVIOR section (existing, unchanged)

  Remove Boss Health, Challenges, and Encounter Timers buttons from the General section.
  </action>
  <verify>cargo check -p baras-app && cargo fmt --check -p baras-app</verify>
  <done>Overlay window shows General (Personal Stats, Raid Frames, Alerts), Encounter (Boss Health, Challenges, Timers A, Timers B), Effects, Metrics, Behavior sections in order</done>
</task>

</tasks>

<verification>
- cargo check -p baras-app passes
- cargo check -p baras-app --features tauri passes (if applicable)
- Visual inspection: Run app, open overlay window, confirm:
  - General section has only Personal Stats, Raid Frames, Alerts
  - Encounter section exists with Boss Health, Challenges, Timers A, Timers B
  - Effects section follows Encounter
  - Timers A and Timers B buttons toggle independently
</verification>

<success_criteria>
- Overlay window UI reorganized with new Encounter section
- Boss Health, Challenges moved from General to Encounter
- "Encounter Timers" renamed to "Timers A"
- New "Timers B" button added and functional
- General section contains only Personal Stats, Raid Frames, Alerts
- Both Timers A and Timers B can be toggled independently
- No compilation errors
</success_criteria>

<output>
After completion, create `.planning/quick/002-encounter-section-overlay-ui/002-SUMMARY.md`
</output>
