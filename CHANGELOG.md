# v2026.2.15

Overlay visual improvements, raid frame icons, effects tracker upgrades, and bug fixes.

Features / Improvements:

- Raid frame role icons now render role glyphs (shield/cross) instead of procedural shapes
- Optional class icons on raid frames — shows player class inline with role icons (off by default)
- Class icons on metric overlays are now role-colored with dark shadow outline for readability
- APM column added to data explorer
- Restyled inline bars in data explorer
- Area-based file indexing and log file explorer filtering
- Sound preview button for alert audio
- Adjustable frame spacing option for raid frames
- Stale log file detection with visual indicator
- Death review now filters by source
- Effects editor shows when a default effect has been modified
- Discipline-scoping for effects tracker
- Separate alert text from audio configuration
- Raid frame grid validation loosened to allow more configurations

Overlay Visual Improvements:

- Text shadow on all overlay types (metrics, boss health, challenges, alerts) for better readability
- Bold text on alerts overlay
- Improved phase, challenges, counters, and effects formatting

Timers and Definitions:

- Added new timers and phase definitions
- Updated Dxun Holding Pen droid counters
- Updated AoE refresh dot ability IDs

Fixes:

- Fix custom timers failing to hot-reload within area
- Fix stale alerts firing on application start
- Fix combat time updating during grace window
- Fix Kolto Probe refresh timing and stack tracking
- Fix Virulence effect logging edge case
- Fix raid registry slot config not reloading on profile switch
- Fix effect refresh registration
- Entities logged at 0 HP are now correctly considered dead
- Stale session detection threshold changed to 15 minutes
