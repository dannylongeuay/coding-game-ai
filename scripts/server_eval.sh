#!/usr/bin/env bash
# Plays a source file against a fixed opponent list on the CodinGame server and reports the win rate.
# Usage: scripts/server_eval.sh <source.rs> <games> [opponents-file=logs/opponents.txt]
# The opponents file has one "agent_id name rank" per line; games cycle through it. Waits out test-game limits.
set -uo pipefail
cd "$(dirname "$0")/.."
SRC=$1 GAMES=$2 OPP=${3:-logs/opponents.txt}
API=https://www.codingame.com/api/v1/challenges/fall-challenge-2026
cg() { curl -s -H "Authorization: Bearer $(cat ~/.config/codingame/token)" "$@"; }
mapfile -t AGENTS < <(awk '{print $1}' "$OPP")
OUT=logs/server-$(basename "$SRC" .rs)-$(date +%H%M%S).txt
wins=0 played=0
for i in $(seq 0 $((GAMES - 1))); do
  agent=${AGENTS[$((i % ${#AGENTS[@]}))]}
  while true; do
    R=$(jq -n --rawfile code "$SRC" --argjson a "$agent" '{language: "Rust", code: $code, opponents: [$a]}' \
      | cg -X POST "$API/test-games" -H 'Content-Type: application/json' --data-binary @-)
    TID=$(jq -r '.test_game_id // empty' <<< "$R" 2>/dev/null)
    [ -n "$TID" ] && break
    if [ "$(jq -r '.code // empty' <<< "$R" 2>/dev/null)" = INVALID_OPPONENT ]; then
      echo "$agent is no longer a valid opponent, skipping" >&2; continue 2
    fi
    wait_s=$(jq -r '.retry_after_seconds // .limits.test_games.resets_in_seconds // 30' <<< "$R" 2>/dev/null || echo 30)
    echo "waiting ${wait_s}s ($(jq -r '.title // .type // "limit"' <<< "$R" 2>/dev/null))" >&2
    sleep "$wait_s"
  done
  LOG=$(cg "https://www.codingame.com/api/v1/test-games/$TID?wait=30")
  until grep -qE '^status: (done|failed)' <<< "$LOG"; do LOG=$(cg "https://www.codingame.com/api/v1/test-games/$TID?wait=30"); done
  me=$(sed -n '/^agents/,/^frame 0/p' <<< "$LOG" | grep -E '\(me' | grep -oE '^\s+[01]' | tr -d ' ')
  if [ -z "$me" ]; then echo "$TID: could not parse the log, skipping" >&2; continue; fi
  first=$(grep -A2 '^result' <<< "$LOG" | grep -oE 'first\): [01]' | grep -oE '[01]$')
  played=$((played + 1))
  res=L; [ "$first" = "$me" ] && { res=W; wins=$((wins + 1)); }
  echo "$TID $agent $res $(grep -A1 '^result' <<< "$LOG" | tail -1 | xargs)" | tee -a "$OUT"
done
echo "$(basename "$SRC"): won $wins/$played" | tee -a "$OUT"
