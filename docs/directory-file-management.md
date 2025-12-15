# SWTOR Log directory

SWTOR logs combat files to a specific directory wit the following defaults:

Linux (Steam):

- $HOME/.local/share/Steam/steamapps/compatdata/1286830/pfx/drive_c/users/steamuser/Documents/Star Wars - The Old Republic/CombatLogs

Windows:

- C:/users/$USER/Documents/Star Wars - The Old Republic/CombatLogs

Mac:

- Unknown

Linux (Non-Steam):

- Unknown

## Directory structure

The files are written in a flat directory with CRLF endline delimiters with the following naming convention:

`combat_YYYY-MM-DD_HH_MM_SS_mmmmmm.txt`

## Log Rotation

The datetime corresponds to the time the player logged in on a specific character.
Whenever a player logs out the game stops writing to the file.
If a player then logs into a character again, a new file is created even if its the same session.
The player can also toggle the "Enable CombatLogs" option in game off then on again and that will
start a new log file without the player logging in or out

### Unknowns

- I don't know if there is a maximum log file size
- If the player disconnects (network or game crash) then logs back in, it interferes with the logging
  in programs like Starparse and users usually toggle their in game combat logging setting to restore functionality. I don't know if there's a way to handle this
  or if it's a game issue not properly logging after restarting from a crash

## Design

A process should be spawned on application start that begins watching the directory and performs the following actions:

- Identify the most recent log file
- Update the AppState to use the most recent log file when the logs are rotated
- Index all available log files along with the name of the primary character and a simplified date
  - the primary character is always between @ and # on the first line of discipline changed
  - e.g. 2025-01-01 Jerran Zeva - Sesssion 1
    2025-01-01 Joe Smith - Session 1
    2025-01-01 Jerran Zeva - Session 2

### Application features

- Allow deletion of log files older than XX days
- Automatically delete empty log files (excluding the latest log file) (this option should be a user choice)
- (future do not implement yet) allow preservation of specific log files
