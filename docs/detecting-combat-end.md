# Combat Encounters

## Start

1. The time stamp of the `EnterCombat` event.
2. The time stamp of a `Damage` event observed more than 15 seconds after the last `ExitCombat` event has been observed (or if no prior exit combat has been observed)

## Termination

A combat encounter ends if one of the following conditions are met:

1. An `ExitCombat` event has been recorded
2. An `AreaEntered` event has been recorded after a `EnterCombat` and before a `ExitCombat` event
3. A `EnterCombat` was recorded after another `EnterCombat` event without an `ExitCombat` event appearing between the two
4. All players in the combat encounter are dead
5. 120 seconds have passed without a `Heal` or `Damage` event being observed
