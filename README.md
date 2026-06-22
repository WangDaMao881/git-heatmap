# git-heatmap

A Claude Code plugin that renders 4 Git activity charts and tracks your personal commit count. Auto-runs after every `git commit` and at session end; `/heatmap` for manual refresh.

## Charts

1. **Heatmap** — GitHub-style 53 × 7 contribution grid (Sunday-anchored).
2. **Trend** — Lines added / deleted per day.
3. **Time of day** — 24-bucket commit distribution (UTC).
4. **Author share** — Top-N authors by commit count, with +/− lines.

Plus `index.json` exposing `total_commits`, `personal_commits`, `range`, `git_head`, etc.

## Install (binary from GitHub release)

```bash
# Download your platform asset from https://github.com/<you>/git-heatmap/releases
tar -xzf git-heatmap-<your-target>.tar.gz      # or unzip on Windows
mkdir -p ~/.claude/plugins/git-heatmap
cp -r bin ~/.claude/plugins/git-heatmap/

# Register as a local marketplace
cat > ~/.claude/plugins/marketplaces/git-heatmap.json <<'EOF'
{ "name": "git-heatmap", "source": "$HOME/.claude/plugins/git-heatmap" }
EOF

claude plugin marketplace add ~/.claude/plugins/marketplaces/git-heatmap.json
claude plugin install git-heatmap@git-heatmap
```

## Install (build from source)

Requires Rust stable (≥ 1.78). On Windows install **MSVC Build Tools** + `rustup target add x86_64-pc-windows-msvc`.

```bash
git clone https://github.com/<you>/git-heatmap.git
cd git-heatmap
cargo build --release
mkdir -p bin
cp target/release/git-heatmap        bin/      # macOS / Linux
cp target/release/git-heatmap.exe    bin/      # Windows
```

Then register as above.

## Usage

- **Auto** — `git commit` → PostToolUse hook → cache + last commit; session `Stop` → full refresh.
- **Manual** — `/heatmap` in Claude Code, or run `bin/git-heatmap --mode manual --range 30d`.
- **Force refresh** — `/heatmap --mode once` (no cache).
- **Custom range** — `--range d7|d30|d90|y1|all`.
- **Personal author override** — `--author alice@corp.com`.

## Output location

`<project>/.git/heatmap/` — auto-created on first run. Contains:

```
heatmap.svg
trend.svg
time-of-day.svg
authors.svg
index.json
.gitignore   # matches *, so the dir is never accidentally committed
```

For git submodules / worktrees the resolved real `.git/` is used (no clash with the `.git` gitfile).

## Plugin manifest

| File | Purpose |
|---|---|
| `.claude-plugin/plugin.json` | Plugin name + commands + hooks |
| `hooks/hooks.json` | `PostToolUse(Bash)` on `git commit:*`, and `Stop` |
| `commands/heatmap.md` | `/heatmap` slash command |
| `skills/git-heatmap/SKILL.md` | Auto-loaded when user asks about git activity |
| `scripts/run-heatmap.sh` / `.cmd` | Cross-platform wrapper: detects OS, picks `.exe`, always exits 0, swallows stdin JSON, emits `{"systemMessage": "..."}` on stdout |

## Failure modes (all silent → systemMessage)

| Symptom | Cause |
|---|---|
| `git-heatmap binary not found` | Run `cargo build --release` and copy to `bin/`. |
| `git is required` | Install git, ensure `git` is on `PATH`. |
| `not a git repository` | Run inside a git repo. Submodules are auto-resolved. |
| Wrapper exits 0 with no output | Probably `<3 commits` in the last 7 days — try `--range d90`. |

## Limitations

- `--mode post-commit` and `--mode stop` are wired in `hooks/hooks.json`; the current Rust build is a single pass that always recomputes the full range. The `--mode` flag exists for a future incremental layer (cache + diff apply).
- Author share shows top-N (default 8) for the current range, not all-time.
- Hour-of-day is UTC (no local-timezone conversion).

## License

MIT.