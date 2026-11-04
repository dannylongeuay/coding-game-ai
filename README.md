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
2. **Predict the opponent.** Run our own move planner from the opponent's side (the board with
   the players swapped) for about a third of the turn's time budget. Plan against that predicted
   placement; the weight on the opponent's current placement is set by `CG_PRED_W` and defaults
   to 0.
3. **Plan our moves.** Each unit stays or moves to a neighbour. The planner starts from a
   coordinate descent over units, then does random pair moves (hill climbing). Each candidate
   placement is scored by `resolve()`, a copy of the referee's end-of-turn rules (surround
   kills, house owners, captures), followed by `evaluate()`:
   - 10 × (our triangles − theirs)
   - 4 × (our units − theirs)
   - 1 × (owned houses difference)
   - −4 for each owned triangle the opponent could take next turn
   - an optional attack-exposure term (`CG_EXPOSE_W`, off by default)
4. **Coordinate captures.** `assign_targets()` sends the nearest free units to the corners of
   the closest capturable triangles. A shaping penalty pulls each unit toward its assigned
   corner. Without it, units oscillate and never complete a triangle.
5. **Spawn.** Spawn while our army is smaller than the opponent's plus 2 units (between 8 and
   40), until turn 160. The bot spawns greedily on the triangle and corner that improve the
   evaluation most, where a unit counts 12 points.

Every `CG_*` environment variable is a tuning parameter for local experiments only. On CodinGame
the defaults always apply.

## Development

Everything runs inside the Nix dev shell: `nix develop`.

- Build: `cargo build --release`. Set `CG_ITERS=<n>` for a deterministic, iteration-budgeted search.
- Local referee: clone the game's referee into `./referee` (gitignored; it has no license, so it
  must never be committed), copy `tools/runner/LocalRunner.java` into
  `referee/src/test/java/`, then build it:
  `mvn -q -DskipTests test-compile dependency:build-classpath -Dmdep.outputFile=cp.txt`.
- `scripts/match.sh <botA> <botB> [games] [jobs] [league=2]` plays local games with seats swapped.
- `scripts/variant.sh <out> <binary> CG_X=1 ...` wraps a binary with fixed settings for match.sh.
- `scripts/server_eval.sh <source.rs> <games>` plays real CodinGame test games against the
  opponents in `logs/opponents.txt` and reports the win rate (needs an API token in
  `~/.config/codingame/token`).

See `HANDOFF.md` for current results and next steps.
