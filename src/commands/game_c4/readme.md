# C4 Game

## TODO

- As players use the reactions to place tokens, keep track of everyone who played.
    At the end, list them all, maybe also with (all of) the colors each player played.
- Limit how many games can exist at once (per server?). Prune old games from memory, either
    on a cadence, or when new games are added.
- Improve game performance? Takes a while for actions to resolve.
- Reduce boilerplate in arbiter--event forwarding functions. Macros? Variable-parameter callbacks?
