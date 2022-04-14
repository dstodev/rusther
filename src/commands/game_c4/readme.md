# C4 Game

## TODO

- As players use the reactions to place tokens, keep track of everyone who played.
    At the end, list them all, maybe also with (all of) the colors each player played.
- Limit how many games can exist at once (per server?). Prune old games from memory, either
    on a cadence, or when new games are added.
- When games are pruned, set the game to a draw, re-render, and strip control reactions.
- Improve game performance? Takes a while for actions to resolve.
  - Right now, it seems that every single action is resolved in a deterministic order, which
      is not necessary. Re-architect sub-handlers? They could take a "pool" to insert
      futures into.
