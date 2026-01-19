# v2026.1.1900

## New Features

- macOS support (experimental)
- X11 support for Linux
- Class icons on metrics overlays
- Split-value metrics overlays (e.g., DPS + HPS combined)
- Toast notifications for errors and feedback
- Single instance enforcement
- File-based logging with automatic rotation

## Improvements

- Session page polish with loading indicators and empty states
- Profile selector always visible with improved empty state
- Overlay settings live preview
- Overlay button tooltips
- Effect editor card-based UI with tooltips
- Hotkey limitation warning for Linux/Wayland
- PlayerStatsBar moved to session page
- Improved Windows font rendering for StarJedi

## Bugfixes

- Shelleigh is now counted to boss DPS in huntmaster
- XR-53 digestive enzyme and Revan force bond no longer contribute to player DPS
- Corruptor Zero timer for first gravity field added
- Vorgath boss encounter is now properly detected on story mode
- Timers will load correctly if the application is restarted/refreshed
- Revanite Commanders now appear on boss healthbar
- Raid frames re-render after profile switch
- Combat log scroll resets on encounter change
- Data explorer race conditions and formatting fixed
- Encounter context properly resets on restart
- Overlay startup data no longer gated behind tailing mode
- Move mode resets on profile switch
