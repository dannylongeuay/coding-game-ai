#!/usr/bin/env bash
# Plays bot A against bot B on the local referee, each seed twice with swapped seats.
# Usage: scripts/match.sh <botA> <botB> [games=20] [jobs=8] [league=3]
# Run inside `nix develop`. Needs ./referee built (see README).
set -euo pipefail
cd "$(dirname "$0")/.."
A=$(realpath "$1"); B=$(realpath "$2"); GAMES=${3:-20}; JOBS=${4:-8}; LEAGUE=${5:-3}
CP="referee/target/classes:referee/target/test-classes:$(cat referee/cp.txt)"
OUT=$(mktemp -d)

run() {
  local seed=$1 swap=$2 p0=$A p1=$B
  if [ "$swap" = 1 ]; then p0=$B; p1=$A; fi
  java --add-opens java.base/java.lang=ALL-UNNAMED -cp "$CP" LocalRunner "$seed" "$LEAGUE" "$p0" "$p1" 2>/dev/null \
    | tail -1 | awk -v swap="$swap" '{ if (swap == 1) print $1, $3, $2; else print $1, $2, $3 }'
}
export -f run; export A B CP LEAGUE

for i in $(seq 1 $((GAMES / 2))); do
  seed=$((RANDOM * 32768 + RANDOM))
  echo "$seed 0"; echo "$seed 1"
done | xargs -P "$JOBS" -n 2 bash -c 'run "$0" "$1"' > "$OUT/results"

mkdir -p logs; cp "$OUT/results" "logs/match-$(date +%H%M%S).txt"
awk '{ if ($2 > $3) w++; else if ($2 < $3) l++; else d++; print }
     END { printf "A wins %d, losses %d, draws %d (%.0f%%)\n", w, l, d, 100 * (w + d / 2) / NR }' "$OUT/results"
rm -rf "$OUT"
