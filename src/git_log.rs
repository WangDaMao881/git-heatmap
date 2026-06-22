//! `git log --numstat` → `Vec<Commit>`.
//!
//! Format: `\t`-separated commit headers, then `\t`-separated numstat lines.
//!   header: `%H\t%aI\t%an\t%ae`
//!   numstat: `<add>\t<del>\t<file>`

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use chrono::{DateTime, Datelike, NaiveDate};

use crate::model::Commit;

/// Returns the resolved absolute path of the real git directory for `cwd`.
/// Handles both `.git` directories (regular repos) and `.git` gitfiles (submodules/worktrees).
pub fn git_dir(cwd: &Path) -> Option<std::path::PathBuf> {
    let mut p = cwd.to_path_buf();
    loop {
        let candidate = p.join(".git");
        if candidate.is_dir() {
            return Some(candidate);
        }
        if candidate.is_file() {
            if let Ok(text) = std::fs::read_to_string(&candidate) {
                if let Some(rest) = text.trim_start().strip_prefix("gitdir:") {
                    let raw = rest.trim();
                    if let Ok(rel) = std::fs::canonicalize(raw) {
                        return Some(rel);
                    }
                }
            }
        }
        match p.parent() {
            Some(parent) if parent != p => p = parent.to_path_buf(),
            _ => return None,
        }
    }
}

pub fn is_git_repo(cwd: &Path) -> bool {
    git_dir(cwd).is_some()
}

pub fn head_short(cwd: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Fetch commits between `[since, until]` (inclusive). `since` before 1970 = no lower bound.
pub fn fetch(cwd: &Path, since: NaiveDate, until: NaiveDate) -> Result<Vec<Commit>> {
    let mut args: Vec<String> = vec![
        "log".into(),
        "--pretty=format:%H%x09%aI%x09%an%x09%ae".into(),
        "--numstat".into(),
    ];
    if since.year() >= 1970 {
        args.push(format!("--since={}", since));
    }
    args.push(format!("--until={}T23:59:59", until));

    let out = Command::new("git")
        .args(&args)
        .env("GIT_PAGER", "cat")
        .current_dir(cwd)
        .output()
        .with_context(|| format!("spawn git log in {}", cwd.display()))?;

    if !out.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }

    let stdout = String::from_utf8_lossy(&out.stdout);
    Ok(parse(&stdout))
}

pub fn parse(raw: &str) -> Vec<Commit> {
    let mut commits: Vec<Commit> = Vec::new();
    let mut current: Option<Commit> = None;

    for line in raw.lines() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() == 4 && looks_like_hash(cols[0]) {
            if let Some(c) = current.take() {
                commits.push(c);
            }
            let ts = DateTime::parse_from_rfc3339(cols[1])
                .map(|d| d.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            current = Some(Commit {
                hash: cols[0].to_string(),
                timestamp: ts,
                author_name: cols[2].to_string(),
                author_email: cols[3].to_string(),
                additions: 0,
                deletions: 0,
            });
        } else if cols.len() >= 3 {
            let add = parse_count(cols[0]);
            let del = parse_count(cols[1]);
            if let Some(ref mut c) = current {
                c.additions = c.additions.saturating_add(add);
                c.deletions = c.deletions.saturating_add(del);
            }
        }
    }
    if let Some(c) = current.take() {
        commits.push(c);
    }
    commits
}

fn looks_like_hash(s: &str) -> bool {
    s.len() >= 7 && s.chars().all(|c| c.is_ascii_hexdigit())
}

fn parse_count(s: &str) -> u64 {
    if s == "-" {
        0
    } else {
        s.parse::<u64>().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_two_commits() {
        let raw = "\
aaaa1111\t2025-03-15T10:00:00+00:00\tAlice\talice@x.com
10\t2\tsrc/a.rs
5\t0\tsrc/b.rs

bbbb2222\t2025-03-16T11:30:00+00:00\tBob\tbob@x.com
0\t3\tREADME.md
";
        let v = parse(raw);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].additions, 15);
        assert_eq!(v[0].deletions, 2);
        assert_eq!(v[1].deletions, 3);
        assert_eq!(v[1].additions, 0);
    }

    #[test]
    fn binary_file_count_dash() {
        let raw = "\
aaaa1111\t2025-03-15T10:00:00+00:00\tAlice\talice@x.com
-\t-\timg.png
";
        let v = parse(raw);
        assert_eq!(v[0].additions, 0);
        assert_eq!(v[0].deletions, 0);
    }
}