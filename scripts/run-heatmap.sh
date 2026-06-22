#!/usr/bin/env bash
# git-heatmap hook wrapper
# - Always exit 0 (harness requirement)
# - Emit {"systemMessage": "..."} on stdout for non-error paths
# - Resolve binary by platform; tolerate spaces in paths
# - Swallow stdin JSON (harness sends it; we don't need it for these events)

set -u

PLUGIN_ROOT="${CLAUDE_PLUGIN_ROOT:-$(cd "$(dirname "$0")/.." && pwd)}"
PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$PWD}"

# 1) Swallow stdin so the pipe never blocks (harness sends JSON on stdin)
cat >/dev/null 2>&1 || true

# 2) Pick the binary for this platform
UNAME_S="$(uname -s 2>/dev/null || echo Unknown)"
case "$UNAME_S" in
  MINGW*|MSYS*|CYGWIN*) BIN_NAME="git-heatmap.exe" ;;
  *)                    BIN_NAME="git-heatmap" ;;
esac

BIN="${PLUGIN_ROOT}/bin/${BIN_NAME}"

# 3) If the binary is missing, fail silently and tell the user in the message
if [ ! -f "$BIN" ]; then
  printf '{"systemMessage":"git-heatmap: binary not found at %s. Run /heatmap once or build per README."}\n' "$BIN"
  exit 0
fi
if [ ! -x "$BIN" ] && [ ! -x "/usr/bin/env" ]; then
  # binary exists but not executable — still try, /usr/bin/env will exec
  :
fi

# 4) git missing check (graceful)
if ! command -v git >/dev/null 2>&1; then
  printf '{"systemMessage":"git-heatmap: git not found on PATH. Install git and retry."}\n'
  exit 0
fi

# 5) Run the binary. Never let its exit code fail the hook.
#    We cd into PROJECT_DIR so `git log` walks the right repo.
OUTPUT="$(cd "$PROJECT_DIR" && "$BIN" "$@" 2>&1)"
RC=$?

if [ "$RC" -ne 0 ]; then
  # truncate long stderr
  SHORT="$(printf '%s' "$OUTPUT" | head -c 200)"
  printf '{"systemMessage":"git-heatmap: binary exited %d. %s"}\n' "$RC" "$SHORT"
  exit 0
fi

# 6) Success path — surface a short summary; index.json is the source of truth.
SHORT="$(printf '%s' "$OUTPUT" | head -c 200)"
printf '{"systemMessage":"git-heatmap: charts refreshed at .git/heatmap/. %s"}\n' "$SHORT"
exit 0