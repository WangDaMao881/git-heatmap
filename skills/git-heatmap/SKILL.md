---
name: git-heatmap
description: Use when the user asks about git activity, commit history visualization, personal commit counts, or wants to "see the heatmap / chart". Auto-runs after `git commit` and on session Stop. Also invokable as `/heatmap`.
---

# git-heatmap skill

## Trigger phrases

Trigger this skill when the user says things like:
- "show my git activity", "render the heatmap", "commit heatmap"
- "how many commits did I make this week / month"
- "代码提交热力图", "我的提交统计", "画一下 git 趋势图"
- "open the heatmap", "刷新一下图表"

## What to do

1. If the user wants an on-demand render or wants to change range / author / top-N, run the `/heatmap` slash command — that handles argument parsing and reports back from `index.json`.
2. If the user just wants a quick number, read `.git/heatmap/index.json` and report `personal_commits` and `total_commits`.
3. If charts look stale (user says "数据不对" / "outdated"), invoke `/heatmap` with `--range all` to force a full refresh.
4. Never try to re-implement the charts in TypeScript / shell — the Rust binary is the source of truth.

## Do NOT trigger

- When the user is asking about a single commit, blame, or diff — that's normal git, not visualization.
- When the user wants CI / pipeline metrics — this skill is local-only.

## Output contract

Always report:
- Personal commit count for the current range
- Total commit count
- Paths to the 4 SVG files (so the user can open them)
- Whether the run was auto (post-commit / stop) or manual