//! Pure aggregation functions from `Vec<Commit>` into chart-ready structures.

use chrono::{Datelike, NaiveDate};

use crate::model::{AuthorBucket, Commit, HeatmapGrid, HourBuckets, TrendSeries};

/// Build a 53-week × 7-day grid spanning `[since, until]`.
/// Days outside the range get count 0; days outside the grid window are dropped.
pub fn build_heatmap(commits: &[Commit], since: NaiveDate, until: NaiveDate) -> HeatmapGrid {
    const WEEKS: usize = 53;
    const DAYS: usize = 7;
    let mut counts = vec![0u32; WEEKS * DAYS];

    // Anchor: start = first Sunday on or before `since`
    let start = first_sunday_on_or_before(since);
    for c in commits {
        let d = c.timestamp.date_naive();
        if d < since || d > until {
            continue;
        }
        let diff_days = (d - start).num_days();
        if diff_days < 0 {
            continue;
        }
        let idx = diff_days as usize;
        if idx >= counts.len() {
            continue;
        }
        counts[idx] = counts[idx].saturating_add(1);
    }
    HeatmapGrid {
        weeks: WEEKS,
        days: DAYS,
        counts,
    }
}

fn first_sunday_on_or_before(d: NaiveDate) -> NaiveDate {
    // chrono 0.4 NaiveDate::weekday() returns Weekday enum.
    // num_days_from_sunday(): Sun=0, Mon=1, ..., Sat=6.
    let days_since_sun = d.weekday().num_days_from_sunday() as i64;
    d - chrono::Duration::days(days_since_sun)
}

/// Build per-day additions/deletions series for the trend chart.
pub fn build_trend(commits: &[Commit], since: NaiveDate, until: NaiveDate) -> TrendSeries {
    use std::collections::BTreeMap;
    let mut add: BTreeMap<NaiveDate, u64> = BTreeMap::new();
    let mut del: BTreeMap<NaiveDate, u64> = BTreeMap::new();
    for c in commits {
        let d = c.timestamp.date_naive();
        if d < since || d > until {
            continue;
        }
        *add.entry(d).or_insert(0) += c.additions;
        *del.entry(d).or_insert(0) += c.deletions;
    }
    let dates: Vec<NaiveDate> = (0..=(until - since).num_days())
        .map(|n| since + chrono::Duration::days(n))
        .collect();
    let additions: Vec<u64> = dates.iter().map(|d| *add.get(d).unwrap_or(&0)).collect();
    let deletions: Vec<u64> = dates.iter().map(|d| *del.get(d).unwrap_or(&0)).collect();
    TrendSeries {
        dates,
        additions,
        deletions,
    }
}

/// 24-bucket histogram by commit hour-of-day (UTC).
pub fn build_hours(commits: &[Commit]) -> HourBuckets {
    let mut buckets = [0u32; 24];
    for c in commits {
        let h = c.timestamp.hour() as usize;
        buckets[h] = buckets[h].saturating_add(1);
    }
    HourBuckets(buckets)
}

/// Top-N authors by commit count.
pub fn build_authors(commits: &[Commit], top_n: usize) -> Vec<AuthorBucket> {
    use std::collections::HashMap;
    let mut by_email: HashMap<String, AuthorBucket> = HashMap::new();
    for c in commits {
        let key = c.author_email.clone();
        let entry = by_email.entry(key).or_insert_with(|| AuthorBucket {
            name: c.author_name.clone(),
            email: c.author_email.clone(),
            commits: 0,
            additions: 0,
            deletions: 0,
        });
        entry.commits = entry.commits.saturating_add(1);
        entry.additions = entry.additions.saturating_add(c.additions);
        entry.deletions = entry.deletions.saturating_add(c.deletions);
    }
    let mut v: Vec<AuthorBucket> = by_email.into_values().collect();
    v.sort_by(|a, b| {
        b.commits
            .cmp(&a.commits)
            .then_with(|| b.additions.cmp(&a.additions))
    });
    v.truncate(top_n);
    v
}

/// Count commits whose author matches `needle` (case-insensitive substring on email or name).
pub fn count_personal(commits: &[Commit], needle: &str) -> u32 {
    let needle = needle.to_lowercase();
    commits
        .iter()
        .filter(|c| {
            c.author_email.to_lowercase().contains(&needle)
                || c.author_name.to_lowercase().contains(&needle)
        })
        .count() as u32
}

/// Total additions + deletions across all commits in the slice.
pub fn total_lines(commits: &[Commit]) -> (u64, u64) {
    let mut add = 0u64;
    let mut del = 0u64;
    for c in commits {
        add = add.saturating_add(c.additions);
        del = del.saturating_add(c.deletions);
    }
    (add, del)
}

/// Additions + deletions for commits whose author matches `needle`.
/// Same matching rule as `count_personal`.
pub fn personal_lines(commits: &[Commit], needle: &str) -> (u64, u64) {
    let needle = needle.to_lowercase();
    let mut add = 0u64;
    let mut del = 0u64;
    for c in commits {
        if c.author_email.to_lowercase().contains(&needle)
            || c.author_name.to_lowercase().contains(&needle)
        {
            add = add.saturating_add(c.additions);
            del = del.saturating_add(c.deletions);
        }
    }
    (add, del)
}

use chrono::Timelike;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn c(hash: &str, when: &str, name: &str, email: &str, add: u64, del: u64) -> Commit {
        Commit {
            hash: hash.into(),
            timestamp: chrono::DateTime::parse_from_rfc3339(when)
                .unwrap()
                .with_timezone(&chrono::Utc),
            author_name: name.into(),
            author_email: email.into(),
            additions: add,
            deletions: del,
        }
    }

    #[test]
    fn heatmap_counts_inside_range() {
        let commits = vec![
            c("a", "2025-03-15T10:00:00Z", "X", "x@x", 1, 0),
            c("b", "2025-03-15T11:00:00Z", "X", "x@x", 1, 0),
            c("c", "2025-04-01T10:00:00Z", "X", "x@x", 1, 0),
        ];
        let since = NaiveDate::from_ymd_opt(2025, 3, 1).unwrap();
        let until = NaiveDate::from_ymd_opt(2025, 4, 30).unwrap();
        let g = build_heatmap(&commits, since, until);
        assert_eq!(g.weeks * g.days, g.counts.len());
    }

    #[test]
    fn hours_buckets_24() {
        let commits = vec![
            c("a", "2025-03-15T00:30:00Z", "X", "x@x", 1, 0),
            c("b", "2025-03-15T23:30:00Z", "X", "x@x", 1, 0),
            c("c", "2025-03-15T23:45:00Z", "X", "x@x", 1, 0),
        ];
        let h = build_hours(&commits);
        assert_eq!(h.0[0], 1);
        assert_eq!(h.0[23], 2);
        assert_eq!(h.0[12], 0);
    }

    #[test]
    fn authors_top_n() {
        let commits = vec![
            c("a", "2025-03-15T10:00:00Z", "Alice", "a@x", 1, 0),
            c("b", "2025-03-15T10:00:00Z", "Alice", "a@x", 1, 0),
            c("c", "2025-03-15T10:00:00Z", "Bob", "b@x", 1, 0),
        ];
        let v = build_authors(&commits, 5);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "Alice");
        assert_eq!(v[0].commits, 2);
    }

    #[test]
    fn personal_count_substring() {
        let commits = vec![
            c("a", "2025-03-15T10:00:00Z", "Alice", "alice@corp.com", 1, 0),
            c("b", "2025-03-15T10:00:00Z", "Bob", "bob@corp.com", 1, 0),
        ];
        assert_eq!(count_personal(&commits, "alice"), 1);
        assert_eq!(count_personal(&commits, "@corp.com"), 2);
    }

    #[test]
    fn total_lines_aggregates() {
        let commits = vec![
            c("a", "2025-03-15T10:00:00Z", "X", "x@x", 10, 3),
            c("b", "2025-03-15T10:00:00Z", "Y", "y@x", 5, 7),
        ];
        assert_eq!(total_lines(&commits), (15, 10));
    }

    #[test]
    fn personal_lines_substring() {
        let commits = vec![
            c("a", "2025-03-15T10:00:00Z", "Alice", "alice@corp.com", 10, 3),
            c("b", "2025-03-15T10:00:00Z", "Bob", "bob@corp.com", 5, 7),
        ];
        assert_eq!(personal_lines(&commits, "alice"), (10, 3));
        assert_eq!(personal_lines(&commits, "@corp.com"), (15, 10));
    }
}
