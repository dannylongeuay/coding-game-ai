#!/usr/bin/env bash
# Writes an executable wrapper that runs a bot binary with fixed CG_* settings, for match.sh.
# Usage: scripts/variant.sh <out> <binary> [VAR=value ...]
set -euo pipefail
out=$1 bin=$(realpath "$2"); shift 2
{ echo '#!/usr/bin/env bash'; for kv in "$@"; do echo "export $kv"; done; echo "exec $bin"; } > "$out"
chmod +x "$out"
