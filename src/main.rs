//! Entry point: parse CLI → fetch git log → aggregate → render SVG → write output.

mod cli;
mod git_log;
mod model;
mod output;
mod signal;
mod stats;
mod svg;

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use clap::Parser;

use crate::cli::{git_user_email, parse_date, Args, Mode};
use crate::model::Bundle;
use crate::svg::{authors, heatmap, time_of_day, trend};

fn main() {
    signal::install();
    if let Err(e) = run() {
        eprintln!("git-heatmap: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    // 1. Project directory
    let project_dir: PathBuf = match &args.project_dir {
        Some(p) => p.clone(),
        None => std::env::current_dir().context("cwd")?,
    };
    if !git_log::is_git_repo(&project_dir) {
        return Err(anyhow!(
            "no .git found at or above {}",
            project_dir.display()
        ));
    }

    // 2. Resolve date range
    let (since, until) = if args.since.is_some() || args.until.is_some() {
        let s = args
            .since
            .as_deref()
            .map(parse_date)
            .transpose()?
            .unwrap_or_else(|| args.range.resolve().unwrap().0);
        let u = args
            .until
            .as_deref()
            .map(parse_date)
            .transpose()?
            .unwrap_or_else(|| chrono::Local::now().date_naive());
        (s, u)
    } else {
        args.range.resolve()?
    };

    // 3. Resolve author identity
    let author_query = args
        .author
        .clone()
        .or_else(git_user_email)
        .unwrap_or_default();

    // 4. Fetch commits
    let commits = git_log::fetch(&project_dir, since, until)?;

    // 5. Aggregate
    let heat_grid = stats::build_heatmap(&commits, since, until);
    let trend_series = stats::build_trend(&commits, since, until);
    let hour_buckets = stats::build_hours(&commits);
    let authors_top = stats::build_authors(&commits, args.top_authors);
    let personal = if author_query.is_empty() {
        0
    } else {
        stats::count_personal(&commits, &author_query)
    };
    let (total_add, total_del) = stats::total_lines(&commits);
    let (personal_add, personal_del) = if author_query.is_empty() {
        (0, 0)
    } else {
        stats::personal_lines(&commits, &author_query)
    };
    let total = commits.len() as u32;

    // 6. Render SVGs
    let heatmap_svg = heatmap::render(&heat_grid);
    let trend_svg = trend::render(&trend_series);
    let tod_svg = time_of_day::render(&hour_buckets);
    let authors_svg = authors::render(&authors_top);

    // 7. Compose bundle
    let bundle = Bundle {
        range_label: args.range.label().to_string(),
        since,
        until,
        total_commits: total,
        total_additions: total_add,
        total_deletions: total_del,
        personal_commits: personal,
        personal_additions: personal_add,
        personal_deletions: personal_del,
        author_query: author_query.clone(),
        top_authors: authors_top,
        heatmap_svg,
        trend_svg,
        time_of_day_svg: tod_svg,
        authors_svg,
        git_head: git_log::head_short(&project_dir).unwrap_or_default(),
        generated_at: Utc::now(),
    };

    // 8. Write output
    let out_dir = match &args.out {
        Some(p) => p.clone(),
        None => {
            // Use the resolved git dir so submodule/worktree worktrees land inside the
            // real .git/, not on top of a `.git` gitfile.
            git_log::git_dir(&project_dir)
                .unwrap_or_else(|| project_dir.join(".git"))
                .join("heatmap")
        }
    };
    output::write_all(&bundle, &out_dir)?;
    output::ensure_local_gitignore(&out_dir)?;

    // 9. Manual mode → summary on stdout
    if args.mode == Mode::Manual {
        println!(
            "git-heatmap [{since} .. {until}] (range={})\n  total_commits = {total} (+{total_add} -{total_del})\n  personal_commits ({author_query}) = {personal} (+{personal_add} -{personal_del})\n  files in {}/",
            args.range.label(),
            out_dir.display()
        );
    }

    Ok(())
}