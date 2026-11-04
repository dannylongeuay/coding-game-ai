# Handoff: Fall Challenge 2026 bot

State as of 2026-11-04. Read `README.md` first for the rules summary, the bot's strategy and the
tooling.

## Where things stand

- **Submitted:** v3b (submission 41635790), which is the logic in `src/main.rs` at commit
  `3aa071a`. Silver league, rank 308 of 653. No Silver boss is listed, so promotion to Gold
  needs a higher rank.
- **`src/main.rs` at HEAD** plays the same as v3b with defaults. It adds an attack-exposure term
  (`CG_EXPOSE_W`, default 0) and a per-turn stderr line:
  `turn T score A-B tris N units A-B owned A-B time Xms`.
- **League mapping:** Bronze = referee level 1 (MOVE and SPAWN only). Silver = level 2 (adds
  ATTACK). Gold is probably level 3 (ADD_PATH and REMOVE_PATH). The bot does not use paths
  yet.

## Results that matter

Server benchmark (`scripts/server_eval.sh`), 20 games per run against the fixed opponents below:

| Version | Wins | Notes |
|---|---|---|
| v3b (submitted) | 11/20, 10/20 | baseline |
| v4b: search-chosen attacks | 7/20 | spends 3 units and a triangle on small attacks; reverted |
| v4b + heavy spawning | 11/20 | wins fast against lower-ranked opponents, loses to others |
| v5: v3b + exposure weight 1.0 | 8/20 | no gain |
| v5s: v5 + heavy spawning | 13/20, 8/20 | 21/40 in total, a tie with v3b |

Heavy spawning means `CG_SPAWN_LEAD=50 MIN=60 MAX=100 VALUE=30`.

**Noise is the main obstacle.** Server games run on a time budget, so even replaying the same
game can flip the result. The same version scored 13/20 on one run and 8/20 on the next. To
detect a 10% improvement you need about 100 games per version. With 80 test games per hour,
that is about 75 minutes per version, and the daily limit is 600 games. Local self-play
(`match.sh`) is just as noisy and misses failures that only other bots expose, so treat it as
a smoke test only.

## Why we lose

Against the strongest of these opponents (Palmipedus, kosynus, Drangoht) we are behind on
units by turn 10, for example 22 to 34. After that we get surrounded and attacked (one kosynus
game: 22 units lost to surround kills and 13 to attacks) and end up dominated around turn 40.

Their openings spread single units over many houses (`MOVE x y 1` everywhere), capture many
triangles and spawn on them every turn. Ours concentrates units on a few target triangles.
Changing weights does not fix this.

## Suggested next steps

1. **Opening and expansion.** Compare our turns 0–15 with a strong opponent's in a test-game
   log. Try evaluation terms that reward spread and capture rate, such as reachable capturable
   triangles or houses held, rather than only the triangles held now.
2. **Benchmark properly.** Use 60–100 server games per version, and keep the same opponent
   list so results stay comparable.
3. **Late game.** Spawning stops at turn 160 and is limited by the unit cap; holding triangles
   for points may matter more near turn 200.
4. **Gold (paths).** Once promoted, add ADD_PATH and REMOVE_PATH. The referee in
   `referee/src/main/java/engine/task/` has the exact rules.

## Practical notes

- **API:** base URL `https://www.codingame.com/api/v1`. The installed CodinGame skill lives in
  `~/.claude/skills/codingame/`. The token is in `~/.config/codingame/token`. Never print,
  paste or commit it; read it inline:
  `curl -H "Authorization: Bearer $(cat ~/.config/codingame/token)" ...`.
- **Test games:** POST `/challenges/fall-challenge-2026/test-games` with
  `{language: "Rust", code, opponents: [agent_id]}`. Then GET `/test-games/<id>?wait=30`
  until the plain-text log says `status: done`. Limits: 10 games per minute, 30 per 15
  minutes, 80 per hour, 600 per day. Check them at
  `/limits?challenge=fall-challenge-2026`.
- **Submissions:** submitting resets the ladder ranking, so only submit a version that clearly
  beats the current one.
- **`logs/` is gitignored** and exists only on this machine. It holds the variant sources
  (`v3b.rs`, `v4b.rs`, `v5.rs`, `v5s.rs`, `variant_spawn.rs`), bot binaries, `server-*.txt`
  results and downloaded `tg-*.txt` game logs.
- **`referee/` is gitignored** and must stay out of git (no license). On JDK 17 the local
  runner needs `--add-opens java.base/java.lang=ALL-UNNAMED`. `match.sh` already passes it.
- **Benchmark opponents** for `server_eval.sh`, ranked 200–336 in Silver. Recreate them in
  `logs/opponents.txt` as `agent_id name rank`:

  ```
  6847271 Drangoht 200
  6847101 perfect_blue 215
  6846425 Palmipedus 230
  6847204 kosynus 245
  6845688 metwesse 260
  6845098 TetraktysPhi 275
  6846304 ashiknur 291
  6844400 Ztrk 306
  6838773 Zylo 321
  6846086 MichaelBerndt 336
  ```

  Ranks drift and agents get replaced when their owners resubmit; refresh the list from
  `/challenges/fall-challenge-2026/opponents` if games fail.
- **Commits:** use the `feat(bot): :sparkles: ...` style and commit only when the user asks.
