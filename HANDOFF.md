# Handoff: Fall Challenge 2026 bot

State as of 2026-11-05. Read `README.md` first for the rules summary, the bot's strategy and the
tooling. The contest ends 2026-11-09 09:00 UTC; Legend opens 2026-11-07 09:00 UTC.

## Where things stand

- **League:** Gold (promoted 2026-11-05). Gold is the final rule set: it adds `ADD_PATH` and
  `REMOVE_PATH`, and the input lists the linkable house pairs. The bot reads that list and ignores
  it.
- **Submitted:** v9 (submission 41643038), `src/main.rs` at this commit. Rank 224 of 396 in Gold
  at 94% of its ranking games (score 15.04). The previous submission (v3b, 41635790) had sunk to
  rank 342 of 386. No Gold boss is listed yet; Legend opens 2026-11-07.
- **Paths are unused by everyone we checked**, including rank 1 (DomiKo). Gold games are won on
  tempo: strong bots capture about 3 triangles a turn from turn 3 and spawn from every one, often
  placing the spawn on the corner that completes another capture that same turn. Games against
  them end by turn 15–35 (we lose all units, or "can't score any more points").

## What changed today (v6–v9)

| Version | Change | Local vs previous HEAD | Server (Gold, ranks 212–371) |
|---|---|---|---|
| v6 | moves and spawns planned in one search | 38–48% (noise) | 11/20 |
| v7 | + prep term: capturable triangles with 2 of 3 corners owned (3.0) | 64–70% | 18/30 |
| v8 | + spawning uncapped (lead 50, max 100), no opponent-spawn prediction | 95% | 20/29 |
| v9 | + vacate term: −1.0 per used triangle we still stand on | 100% (72% vs v8) | not run; submitted |

Local tuning that did not help (60–100 games each against the version shown): prep 2.0 or
5.0 (both lost to 3.0 on v8 settings) and 8.0 (collapses); spawn value 16 with the cap kept;
node weight 0.5/2.0; attack threshold 2; assign weight 1.0; spawning until turn 200; vacate 0.5,
1.5 and 2.0.

**Noise:** two identical bots score 48–52% over 40 local games; treat ±10% at 60 games as noise.
Local self-play is now in Gold rules by default (`match.sh` league 3).

## Suggested next steps

1. **Follow v9's ranking** (`/progress`, `/battles`). Download lost battles (`GET /games/<id>`,
   plain text with our stderr) and look at where units are lost.
2. **Opening speed.** Compare our captures per turn with a top bot's in turns 0–15
   (`logs/tg-domiko1.txt` has a full game against DomiKo with v5 at turns 0–16).
3. **Paths.** Possibly worth trying late: `ADD_PATH` can create triangles inside our territory, and
   `REMOVE_PATH` can break an opponent triangle. The referee rules: `from` is a corner of a
   triangle we owned since the start of the turn, costs 1 unit per corner (units that were there
   the whole turn), several per turn are allowed but resolve one per sub-step with both players'
   adds ordered shortest first, max length 400, no crossings.
   New triangles start capturable by both players.
4. **Late game.** Spawning still stops at turn 160.

## Practical notes

- **API:** base URL `https://www.codingame.com/api/v1`. The installed CodinGame skill lives in
  `~/.claude/skills/codingame/`. The token is in `~/.config/codingame/token`. Never print,
  paste or commit it; read it inline:
  `curl -H "Authorization: Bearer $(cat ~/.config/codingame/token)" ...`.
- **Test games:** POST `/challenges/fall-challenge-2026/test-games` with
  `{language: "Rust", code, opponents: [agent_id]}`. Then GET `/test-games/<id>?wait=30`
  until the plain-text log says `status: done`. Limits: 10 per minute, 30 per 15 minutes,
  80 per hour, 600 per day. The 15-minute window is the one you hit; `server_eval.sh` waits it out.
- **Submissions:** submitting resets the ladder ranking, so only submit a version that clearly
  beats the current one.
- **`logs/` is gitignored** and exists only on this machine. It holds variant sources
  (`v3b.rs` … `v9.rs`, `v5-head.rs` = the logic of v3b), binaries (`bot-head` = v3b logic,
  `bot-v7` … `bot-v9`), `server-*.txt` results and downloaded game logs.
- **`referee/` is gitignored** and must stay out of git (no license). On JDK 17 the local
  runner needs `--add-opens java.base/java.lang=ALL-UNNAMED`. `match.sh` already passes it.
- **No `python3` outside `nix develop`.** Use `jq`, `sed` or the Nix shell.
- **Benchmark opponents** for `server_eval.sh` are in `logs/opponents-gold.txt`
  (`agent_id name rank`, Gold ranks 212–371 on 2026-11-05):

  ```
  6850662 jBigman 212
  6850137 antoine.bonamy 260
  6849516 ehajnik 268
  6850476 colegu 277
  6850357 SparkleX 300
  6849822 OldJohn 317
  6849553 Julien_E 328
  6849579 zangaflex 346
  6849883 Ghostjin 359
  6850468 Kathrin 371
  ```

  Agents get replaced when their owners resubmit. The script now skips an agent the server
  rejects; refresh ids from `/challenges/fall-challenge-2026/opponents` by nickname.
- **Commits:** use the `feat(bot): :sparkles: ...` style and commit only when the user asks.
