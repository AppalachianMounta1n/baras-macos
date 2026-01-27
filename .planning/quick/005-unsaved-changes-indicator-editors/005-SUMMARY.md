---
phase: quick-005
plan: 01
subsystem: ui/editors
tags: [dioxus, css, ux, feedback]
tech-stack:
  patterns: [on_dirty callback, use_effect sync]
files:
  created: []
  modified:
    - app/assets/styles.css
    - app/src/components/effect_editor.rs
    - app/src/components/encounter_editor/timers.rs
    - app/src/components/encounter_editor/phases.rs
    - app/src/components/encounter_editor/counters.rs
    - app/src/components/encounter_editor/challenges.rs
    - app/src/components/encounter_editor/entities.rs
decisions: []
metrics:
  duration: ~5 min
  completed: 2026-01-27
---

# Quick Task 005: Unsaved Changes Indicator for Editors

**One-liner:** Orange pulsing dot indicator for unsaved changes in effect and encounter editor rows.

## Objective

Users needed visual feedback when an expanded item has been modified but not saved. Previously the only indication was that the Save button became enabled, which was easy to miss, leading to accidental data loss when collapsing without saving.

## What Was Built

### CSS Styling
- Added `.unsaved-indicator` class with 8px orange pulsing dot
- Uses `--swtor-orange` CSS variable for consistent theming
- Subtle 2s ease-in-out opacity animation between 1 and 0.6

### Effect Editor
- Added `is_dirty` signal to `EffectRow` component
- Added `on_dirty: EventHandler<bool>` prop to `EffectEditForm`
- `EffectEditForm` now notifies parent via `use_effect` when `has_changes` memo changes
- Indicator displays next to effect name when expanded AND dirty

### Encounter Editor (5 tabs)
Applied identical pattern to all encounter editor tabs:

| File | Row Component | EditForm Component |
|------|---------------|-------------------|
| timers.rs | TimerRow | TimerEditForm |
| phases.rs | PhaseRow | PhaseEditForm |
| counters.rs | CounterRow | CounterEditForm |
| challenges.rs | ChallengeRow | ChallengeEditForm |
| entities.rs | EntityRow | EntityEditForm |

Each follows the same pattern:
1. `is_dirty` signal in Row component
2. `on_dirty` callback passed to EditForm
3. `use_effect` in EditForm syncs `has_changes` to parent
4. Indicator rendered conditionally: `expanded && is_dirty()`

## Implementation Pattern

```rust
// In Row component
let mut is_dirty = use_signal(|| false);

// In row header
if expanded && is_dirty() {
    span { class: "unsaved-indicator", title: "Unsaved changes" }
}

// Pass to EditForm
EditForm {
    on_dirty: move |dirty: bool| is_dirty.set(dirty),
    // ... other props
}

// In EditForm component
#[props(default)] on_dirty: EventHandler<bool>,

// After has_changes memo
use_effect(move || {
    on_dirty.call(has_changes());
});
```

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 2960bc6 | style | Add unsaved indicator CSS class |
| d904809 | feat | Add unsaved indicator to effect editor rows |
| 1e37b68 | feat | Add unsaved indicator to encounter editor rows |

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- [x] `cargo check -p app` passes
- [x] CSS file contains `.unsaved-indicator` class
- [x] Effect editor rows have indicator logic
- [x] All 5 encounter editor tabs have indicator logic
