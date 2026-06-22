---
description: Generate / refresh the 4 Git activity charts (heatmap, code trend, time-of-day, author share) and personal commit count
argument-hint: "[--range 7d|30d|90d|1y|all] [--author <name-or-email>] [--top-authors <N>] [--mode post-commit|stop|manual|once]"
allowed-tools: "Bash(${CLAUDE_PLUGIN_ROOT}/scripts/run-heatmap.sh:*) Read"
---

# /heatmap — Git activity visualization

## What this does

Runs the `git-heatmap` Rust binary against the current repository. Produces 4 SVG charts in `.git/heatmap/`:

- `heatmap.svg` — 7×52 commit heatmap (GitHub contribution style)
- `trend.svg` — daily lines added / removed over the selected range
- `time-of-day.svg` — 24-bucket commit distribution
- `authors.svg` — top N author share (horizontal bars)
- `index.json` — machine-readable summary (includes `personal_commits` count)

## Steps

1. Default `--range 90d` if `$ARGUMENTS` is empty.
2. Run the binary via the wrapper script:

   ```bash
   "${CLAUDE_PLUGIN_ROOT}/scripts/run-heatmap.sh" --mode manual $ARGUMENTS
   ```

3. Read `.git/heatmap/index.json` and report:
   - Personal commit count for the current range
   - Total commits
   - Top 3 authors
4. List the 4 SVG file paths so the user can open them.

## Notes

- The binary shells out to `git log`; requires `git` on `PATH`.
- Binary is platform-specific: download the matching artifact from GitHub Releases or build from source (see README).
- Hooks auto-run on `git commit` (PostToolUse) and on session Stop. Use this command for on-demand refresh, to change range/author, or after merges/rebases.