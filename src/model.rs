//! Domain types shared between git_log, stats, svg, and output.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Commit {
    pub hash: String,
    pub timestamp: DateTime<Utc>,
    pub author_name: String,
    pub author_email: String,
    pub additions: u64,
    pub deletions: u64,
}

/// 53 × 7 grid, Sunday-anchored weeks.
#[derive(Debug, Clone)]
pub struct HeatmapGrid {
    pub weeks: usize,
    pub days: usize,
    pub counts: Vec<u32>,
}

/// Per-day additions/deletions series for the trend chart.
#[derive(Debug, Clone)]
pub struct TrendSeries {
    pub dates: Vec<NaiveDate>,
    pub additions: Vec<u64>,
    pub deletions: Vec<u64>,
}

/// 24-bucket histogram (hour-of-day UTC).
#[derive(Debug, Clone)]
pub struct HourBuckets(pub [u32; 24]);

/// One author bucket for the author-share chart.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorBucket {
    pub name: String,
    pub email: String,
    pub commits: u32,
    pub additions: u64,
    pub deletions: u64,
}

/// Aggregate output passed to `output::write_all`.
#[derive(Debug, Clone)]
pub struct Bundle {
    pub range_label: String,
    pub since: NaiveDate,
    pub until: NaiveDate,
    pub total_commits: u32,
    pub total_additions: u64,
    pub total_deletions: u64,
    pub personal_commits: u32,
    pub personal_additions: u64,
    pub personal_deletions: u64,
    pub author_query: String,
    pub top_authors: Vec<AuthorBucket>,
    pub heatmap_svg: String,
    pub trend_svg: String,
    pub time_of_day_svg: String,
    pub authors_svg: String,
    pub git_head: String,
    pub generated_at: DateTime<Utc>,
}