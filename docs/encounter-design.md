# Encounter Design

The primary unit of analysis for the combat logs is a combat encounter. After parsing a combat event each event needs to be associated with a combat encounter.

A Encounter class should serve the following purposes:

- Determining the time basis for statistics calculations
- Determining the active encounter to display in real time
- Retrieving a description of the encounter based on the enemy NPCs displayed in the encounter
- Providing some method to efficiently query on the all the relative events
- Provide a method to determine if this is a boss encounter or a trash encounter (based on IDs for entities)

The following things to keep in mind:

- Fast queries. Any queries should be <<20ms to allow for real time display
- Minimize memory usage. Make use of caching and other tactics in order to keep the application's memory footprint <300mb at all time

## Considerations

### Timers

Eventually timers will be implemented. A timer may or may not be related to an encounter. For example a 5 minute timer for the battle revive cooldown would be encounter independent, however a timer informing the user of an upcoming boss attack would be removed upon an encounter end.

### Effect Overlays

Overlays for Heals Over Time (HOTs) or other effects simply record the current presence or absence of an effect applied to a character. These effects can be applied in between combat and the information is expected to always be present regardless of an encounter.

- Some effects persist through player death
- The majority of effects are removed on player death

The challenge here is to efficiently feed into this overlay without needing to query an unlimited amount of data.

### Post-Encounter Analytics

Eventually the future UI will also include the ability to view encounter details after for a more in-depth look at analytics. This would mean the user can select a arbitrary encounter in the log file and should have all the data for it visible in near-real time.

## Memory Management

Currently reading a 56mb log file into the program results in about a ~250mb increase in memory usage just storing the data without accounting for the future UI or performing any operations. This seems excessive and there are a few approaches to manage this:

- Lazy Loading encounters - figure out parsimonious boundaries for selecting a subset of events that will satisfy all use cases and then only load that data. Implement functionality to allow the user to select a prior encounter and only load the relevant data in for that.
  - Will need to figure out how to efficiently index the values
  - Could potentially read and write to a parquet file that is ad hoc queried from while keeping the last XX logs in memory

- Better storage format. Look through the current structure of CombatEvent and consider improvements or refactoring it using a crate like `polars` that can help handle events

## Performance
