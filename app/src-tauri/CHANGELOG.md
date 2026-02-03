# v2026.2.3

- Implemented fix for regression causing parser to freeze.
- Encounters will now time out 5 seconds after the local player receives the revive immunity buff
- Ravager/TfB encounters no longer show all bosses as wipes
- Fixed issue causing some bosses to be registered in trash fights prior to encounter
- Enemies will only appear on the HP overlays after they have taken damage.
- Removed Watchdog as a kill target in Lady Dom, causing wipes to be classified as success

- Added experimental Wayland Hotkey support
- Changes to overlay state via hotkeys will now be reflected in the UI

# v2026.2.2 Hotfix

- **Fixed issue causing timers not to appear for new overlay profiles and new users**
- Improved wipe detection logic
- Encounters ended by exiting to med center will no longer appear to be in the area exited to
- Parsely upload success toast notification now stays until closed by user

# v2026.2.1

## What's New

### General

- Individual combats can now be uploaded to parsely.io via the session page
- Users can now set visibility and add an optional note when uploading to Parsely
- Starting the application in the middle of combat will now detect and parse the in-progress encounter
- UI positions and open elements are now preserved across tab-navigation; including the combat log scroll position
- Tweaked combat log formatting
- Improved handling of SWTOR combat log rotation upon character login/logout

### Encounter Classification

- Fake combat encounters that occur shortly after fights (e.g. Dread Master Holocrons) are now automatically ignored
- Fixed several edge cases causing encounter to split if mechanics are pushed too fast or player was revived at a specific time
- Fixed issue causing encounter to be classified as wipe if the local player used area start revive
- Coratanni boss fight will no longer appear split across multiple encounters if the local player dies during the encounter

### Timers and Bosses

- Fixed typo causing Ravagers default definitions failing to appear
- Fixed several text alerts on ToS firing on non-local player
