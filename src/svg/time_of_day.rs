//! 24-hour commit distribution histogram (hand-written SVG).

use crate::model::HourBuckets;

pub fn render(buckets: &HourBuckets) -> String {
    let w = 720usize;
    let h = 200usize;
    let pad_l = 30usize;
    let pad_b = 30usize;
    let pad_t = 20usize;
    let chart_h = h - pad_b - pad_t;
    let bar_count = 24;
    let total_bar_w = w - pad_l;
    let bar_w = total_bar_w / bar_count - 2;

    let max = buckets.0.iter().copied().max().unwrap_or(0).max(1);

    let mut s = String::new();
    s.push_str(&format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" font-family="-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif" font-size="10" fill="#586069">"##
    ));

    s.push_str(&format!(
        r##"<text x="{pad_l}" y="14" font-size="12" fill="#24292e">Commits by hour of day (UTC)</text>"##
    ));

    s.push_str(&format!(
        r##"<text x="0" y="{pad_t}" fill="#586069">{max}</text>"##
    ));
    s.push_str(&format!(
        r##"<text x="0" y="{}" fill="#586069">0</text>"##,
        pad_t + chart_h
    ));

    for hour in 0..24 {
        let count = buckets.0[hour];
        let bar_h = (count as f64 / max as f64 * chart_h as f64) as usize;
        let x = pad_l + hour * (bar_w + 2);
        let y = pad_t + (chart_h - bar_h);
        let color = if hour >= 9 && hour <= 18 { "#40c463" } else { "#9be9a8" };
        s.push_str(&format!(
            r##"<rect x="{x}" y="{y}" width="{bar_w}" height="{bar_h}" fill="{color}"><title>{hour:02}:00 &#8212; {count} commits</title></rect>"##
        ));
        if hour % 3 == 0 {
            let label_y = pad_t + chart_h + 12;
            s.push_str(&format!(
                r##"<text x="{}" y="{label_y}" text-anchor="middle">{hour:02}</text>"##,
                x + bar_w / 2
            ));
        }
    }

    s.push_str(&format!(
        r##"<line x1="{pad_l}" y1="{}" x2="{}" y2="{}" stroke="#d0d7de" />"##,
        pad_t + chart_h,
        w - 5,
        pad_t + chart_h
    ));

    s.push_str("</svg>");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_24_bars() {
        let b = HourBuckets([1; 24]);
        let svg = render(&b);
        let count = svg.matches("<rect").count();
        assert_eq!(count, 24);
    }
}
