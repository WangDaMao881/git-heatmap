//! CLI argument parsing and range resolution.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "git-heatmap",
    version,
    about = "Render Git activity charts and personal commit count."
)]
pub struct Args {
    /// Operating mode.
    #[arg(long, value_enum, default_value_t = Mode::Once)]
    pub mode: Mode,

    /// Time range window.
    #[arg(long, value_enum, default_value_t = Range::D90)]
    pub range: Range,

    /// Author identity used for personal commit counting. Defaults to `git config user.email`.
    #[arg(long)]
    pub author: Option<String>,

    /// Top-N authors shown on the author share chart.
    #[arg(long, default_value_t = 8)]
    pub top_authors: usize,

    /// Explicit start date (YYYY-MM-DD), overrides `--range`.
    #[arg(long)]
    pub since: Option<String>,

    /// Explicit end date (YYYY-MM-DD). Defaults to today.
    #[arg(long)]
    pub until: Option<String>,

    /// Output directory. Defaults to `<cwd>/.git/heatmap/`.
    #[arg(long)]
    pub out: Option<PathBuf>,

    /// Current working directory (project root). Defaults to the current dir.
    #[arg(long)]
    pub project_dir: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Incrementally update with the latest commit (cache + 1).
    PostCommit,
    /// Full refresh at session end (cache + latest commit).
    Stop,
    /// Full render and print summary to stdout.
    Manual,
    /// Full render, no cache, no stdout.
    Once,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum)]
pub enum Range {
    D7,
    D30,
    D90,
    Y1,
    All,
}

impl Range {
    pub fn label(&self) -> &'static str {
        match self {
            Range::D7 => "7d",
            Range::D30 => "30d",
            Range::D90 => "90d",
            Range::Y1 => "1y",
            Range::All => "all",
        }
    }

    /// Returns `(since, until)` where `until` is inclusive.
    pub fn resolve(&self) -> Result<(NaiveDate, NaiveDate)> {
        let until = Local::now().date_naive();
        let since = match self {
            Range::D7 => until - chrono::Duration::days(7),
            Range::D30 => until - chrono::Duration::days(30),
            Range::D90 => until - chrono::Duration::days(90),
            Range::Y1 => NaiveDate::from_ymd_opt(until.year() - 1, until.month(), until.day())
                .ok_or_else(|| anyhow!("invalid date arithmetic"))?,
            Range::All => NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
        };
        Ok((since, until))
    }
}

pub fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .with_context(|| format!("invalid date '{s}', expected YYYY-MM-DD"))
}

pub fn git_user_email() -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["config", "user.email"])
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