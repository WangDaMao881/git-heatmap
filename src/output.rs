//! Atomic write of the 4 SVG files + `index.json` into `<project>/.git/heatmap/`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use crate::model::Bundle;

#[derive(Debug, Serialize)]
struct IndexFile<'a> {
    range: &'a str,
    since: String,
    until: String,
    total_commits: u32,
    total_additions: u64,
    total_deletions: u64,
    personal_commits: u32,
    personal_additions: u64,
    personal_deletions: u64,
    author_query: &'a str,
    top_authors: &'a [crate::model::AuthorBucket],
    git_head: &'a str,
    generated_at: String,
    files: IndexFiles<'a>,
}

#[derive(Debug, Serialize)]
struct IndexFiles<'a> {
    heatmap: &'a str,
    trend: &'a str,
    time_of_day: &'a str,
    authors: &'a str,
}

/// Write all artifacts to `dir`. Creates `dir` if missing. Each write goes
/// through `<name>.tmp` and is then atomically renamed.
pub fn write_all(bundle: &Bundle, dir: &Path) -> Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("create {}", dir.display()))?;

    atomic_write(&dir.join("heatmap.svg"), bundle.heatmap_svg.as_bytes())?;
    atomic_write(&dir.join("trend.svg"), bundle.trend_svg.as_bytes())?;
    atomic_write(&dir.join("time-of-day.svg"), bundle.time_of_day_svg.as_bytes())?;
    atomic_write(&dir.join("authors.svg"), bundle.authors_svg.as_bytes())?;

    let idx = IndexFile {
        range: &bundle.range_label,
        since: bundle.since.to_string(),
        until: bundle.until.to_string(),
        total_commits: bundle.total_commits,
        total_additions: bundle.total_additions,
        total_deletions: bundle.total_deletions,
        personal_commits: bundle.personal_commits,
        personal_additions: bundle.personal_additions,
        personal_deletions: bundle.personal_deletions,
        author_query: &bundle.author_query,
        top_authors: &bundle.top_authors,
        git_head: &bundle.git_head,
        generated_at: bundle.generated_at.to_rfc3339(),
        files: IndexFiles {
            heatmap: "heatmap.svg",
            trend: "trend.svg",
            time_of_day: "time-of-day.svg",
            authors: "authors.svg",
        },
    };
    let json = serde_json::to_string_pretty(&idx).context("serialize index.json")?;
    atomic_write(&dir.join("index.json"), json.as_bytes())?;

    Ok(())
}

/// Ensures `dir/.gitignore` contains a `*` so the contents never get committed.
pub fn ensure_local_gitignore(dir: &Path) -> Result<()> {
    let gi = dir.join(".gitignore");
    if !gi.exists() {
        fs::write(&gi, "*\n!.gitignore\n")
            .with_context(|| format!("write {}", gi.display()))?;
    }
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let tmp = path.with_extension(format!(
        "{}.tmp",
        path.extension().and_then(|s| s.to_str()).unwrap_or("")
    ));
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("rename to {}", path.display()))?;
    Ok(())
}