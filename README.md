# coding-game-ai

Online bot programming with AI assistance. The current game is CodinGame's
[Fall Challenge 2026](https://www.codingame.com/contests/fall-challenge-2026), played by a
single-file Rust bot (`src/main.rs`, std only).

## The game in short

Two players move units around a graph of houses. Three mutually connected houses with no house
inside form a triangle. A player owns a house by having strictly more units on it, or by holding
the unit majority on every neighbouring house. Owning all 3 corners of a triangle captures it,
and each captured triangle scores 1 point per turn. Using a triangle makes it neutral again:
`SPAWN` adds a unit, `ATTACK` (Silver and up) wipes every enemy unit on an adjacent house for
1 unit per corner, and `ADD_PATH`/`REMOVE_PATH` change the graph (probably Gold). Units on a
house whose neighbours all have an enemy majority are killed. A player wins with 80% of the
triangles, by eliminating the opponent, or on points after 200 turns.

## Strategy

Each turn the bot runs a short search that optimises the next move only, in this order:

1. **Attacks (fixed rule).** From every triangle we own with a unit on each corner, attack the
   largest adjacent enemy stack of 3 or more units. Letting the search choose attacks was tried
   (v4b) and lost more games, because it spent units on small attacks.
2. **Predict the opponent.** Run our own planner from the opponent's side (the board with
   the players swapped) for about a third of the turn's time budget. Plan against that predicted
   placement; the weight on the opponent's current placement is set by `CG_PRED_W` and defaults
   to 0. The prediction leaves out the opponent's spawns (`CG_PRED_SPAWNS`, default 0); adding
   them lost locally.
3. **Plan moves and spawns together.** Each unit stays or moves to a neighbour, and each triangle
   we own either spawns on one of its corners or not. Planning both together lets a spawn land
   on the corner that completes another capture the same turn, which is how strong Gold bots
   grow. The planner starts from a coordinate descent over these choices, then does random pair
   changes (hill climbing). Each candidate is scored by `resolve()`, a copy of the referee's
   end-of-turn rules (surround kills, house owners, captures), followed by `evaluate()`:
   - 10 × (our triangles − theirs)
   - 4 × (our units − theirs), plus 8 for each unit we spawn
   - 1 × (owned houses difference)
   - −4 for each owned triangle the opponent could take next turn
   - 3 × (capturable triangles with 2 of 3 corners owned, ours − theirs), so units set up the
     next turn's captures (`CG_PREP_W`)
   - an optional attack-exposure term (`CG_EXPOSE_W`, off by default)
4. **Coordinate captures.** `assign_targets()` sends the nearest free units to the corners of
   the closest capturable triangles. A shaping penalty pulls each unit toward its assigned
   corner. Without it, units oscillate and never complete a triangle.
5. **Spawn limits.** Spawning is effectively uncapped (the army may reach the opponent's plus 50,
   at most 100 units) until turn 160.

The bot does not use `ADD_PATH` or `REMOVE_PATH` (Gold). Through Gold, even the top-ranked
players did not use them in the games we looked at.

Every `CG_*` environment variable is a tuning parameter for local experiments only. On CodinGame
the defaults always apply.

## Development

Everything runs inside the Nix dev shell: `nix develop`.

- Build: `cargo build --release`. Set `CG_ITERS=<n>` for a deterministic, iteration-budgeted search.
- Local referee: clone the game's referee into `./referee` (gitignored; it has no license, so it
  must never be committed), copy `tools/runner/LocalRunner.java` into
  `referee/src/test/java/`, then build it:
  `mvn -q -DskipTests test-compile dependency:build-classpath -Dmdep.outputFile=cp.txt`.
- `scripts/match.sh <botA> <botB> [games] [jobs] [league=3]` plays local games with seats swapped.
- `scripts/variant.sh <out> <binary> CG_X=1 ...` wraps a binary with fixed settings for match.sh.
- `scripts/server_eval.sh <source.rs> <games>` plays real CodinGame test games against the
  opponents in `logs/opponents.txt` and reports the win rate (needs an API token in
  `~/.config/codingame/token`).

See `HANDOFF.md` for current results and next steps.
