//! Top-N author share as horizontal bars (hand-written SVG).

use crate::model::AuthorBucket;

pub fn render(authors: &[AuthorBucket]) -> String {
    if authors.is_empty() {
        return r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="60" font-family="sans-serif" font-size="12"><text x="10" y="32" fill="#586069">No authors in range.</text></svg>"##.to_string();
    }

    let row_h = 24usize;
    let pad_l = 160usize;
    let pad_r = 20usize;
    let pad_t = 30usize;
    let pad_b = 10usize;
    let w = 720usize;
    let h = pad_t + authors.len() * row_h + pad_b;
    let chart_w = w - pad_l - pad_r;
    let max = authors.iter().map(|a| a.commits).max().unwrap_or(0).max(1);

    let mut s = String::new();
    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" font-size="11" fill="#24292e">"##
    ));

    s.push_str(&format!(
        r##"<text x="{pad_l}" y="18" font-size="13" fill="#24292e">Top authors by commits</text>"##
    ));

    for (i, a) in authors.iter().enumerate() {
        let y = pad_t + i * row_h;
        let name = truncate(&a.name, 22);
        s.push_str(&format!(
            r##"<text x="0" y="{}" fill="#24292e">{name}</text>"##,
            y + 14
        ));
        let bar_w = (a.commits as f64 / max as f64 * chart_w as f64) as usize;
        let bx = pad_l;
        let by = y + 4;
        let bh = row_h - 8;
        s.push_str(&format!(
            r##"<rect x="{bx}" y="{by}" width="{bar_w}" height="{bh}" fill="#40c463" rx="2"><title>{} &#8212; {} commits (+{} -{})</title></rect>"##,
            a.name, a.commits, a.additions, a.deletions
        ));
        s.push_str(&format!(
            r##"<text x="{}" y="{}" fill="#586069" font-size="10">{} (+{} -{})</text>"##,
            bx + bar_w + 4,
            y + 14,
            a.commits,
            a.additions,
            a.deletions
        ));
    }

    s.push_str("</svg>");
    s
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('\u{2026}');
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_svg() {
        let svg = render(&[]);
        assert!(svg.contains("No authors"));
    }

    #[test]
    fn one_author() {
        let v = vec![AuthorBucket {
            name: "Alice".into(),
            email: "a@x".into(),
            commits: 5,
            additions: 100,
            deletions: 30,
        }];
        let svg = render(&v);
        assert!(svg.contains("Alice"));
        assert!(svg.contains("<rect"));
    }
}
