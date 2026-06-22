//! GitHub-style contribution heatmap: 53 weeks x 7 days.

use crate::model::HeatmapGrid;

/// Render heatmap as SVG. `palette` is the GitHub green scale (low -> high).
pub fn render(grid: &HeatmapGrid) -> String {
    let cell = 11usize;
    let gap = 2usize;
    let label_w = 22usize;
    let label_h = 16usize;
    let w = label_w + grid.weeks * (cell + gap);
    let h = label_h + grid.days * (cell + gap);
    let max = grid.counts.iter().copied().max().unwrap_or(0);

    let mut s = String::with_capacity(w * h / 2);
    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" font-size="9" fill="#586069">"##
    ));

    // Weekday labels
    let weekdays = ["Mon", "Wed", "Fri"];
    for (i, label) in weekdays.iter().enumerate() {
        let row = i * 2 + 1; // Mon(1), Wed(3), Fri(5)
        let y = label_h + row * (cell + gap) + cell - 2;
        s.push_str(&format!(
            r##"<text x="0" y="{y}">{label}</text>"##
        ));
    }

    // Cells
    for week in 0..grid.weeks {
        for day in 0..grid.days {
            let idx = week * grid.days + day;
            let count = grid.counts[idx];
            let color = bucket_color(count, max);
            let x = label_w + week * (cell + gap);
            let y = label_h + day * (cell + gap);
            s.push_str(&format!(
                r##"<rect x="{x}" y="{y}" width="{cell}" height="{cell}" fill="{color}" rx="2"><title>{count}</title></rect>"##
            ));
        }
    }

    s.push_str("</svg>");
    s
}

/// Map count -> GitHub green palette index 0..4.
fn bucket_color(count: u32, max: u32) -> &'static str {
    if count == 0 {
        return "#ebedf0";
    }
    if max == 0 {
        return "#9be9a8";
    }
    let ratio = count as f64 / max as f64;
    if ratio < 0.25 {
        "#9be9a8"
    } else if ratio < 0.5 {
        "#40c463"
    } else if ratio < 0.75 {
        "#30a14e"
    } else {
        "#216e39"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_53x7() {
        let g = HeatmapGrid {
            weeks: 53,
            days: 7,
            counts: vec![0; 53 * 7],
        };
        let svg = render(&g);
        assert!(svg.starts_with("<svg"));
        assert!(svg.ends_with("</svg>"));
        assert!(svg.contains("<rect"));
    }
}
