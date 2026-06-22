//! Lines added / deleted over time, via plotters svg-only backend.

use crate::model::TrendSeries;

pub fn render(series: &TrendSeries) -> String {
    use plotters::prelude::*;
    use plotters::series::LineSeries;

    if series.dates.is_empty() {
        return empty_svg();
    }

    let mut buf = String::new();
    let w = (series.dates.len().max(30) * 8).min(1200) as u32;
    let h = 280u32;
    let max_y = series
        .additions
        .iter()
        .chain(series.deletions.iter())
        .copied()
        .max()
        .unwrap_or(0)
        .max(1);

    let mut buf = String::new();
    let w = (series.dates.len().max(30) * 8).min(1200) as u32;
    let h = 280u32;
    let max_y = series
        .additions
        .iter()
        .chain(series.deletions.iter())
        .copied()
        .max()
        .unwrap_or(0)
        .max(1);

    {
        let backend = SVGBackend::with_string(&mut buf, (w, h));
        let root = backend.into_drawing_area();
        root.fill(&WHITE).unwrap();

        let mut chart = ChartBuilder::on(&root)
            .margin(10)
            .caption("Lines added / deleted per day", ("sans-serif", 14))
            .x_label_area_size(20)
            .y_label_area_size(40)
            .build_cartesian_2d(
                0..series.dates.len(),
                0u64..(max_y as u64 + 10),
            )
            .unwrap();

        chart
            .configure_mesh()
            .disable_x_mesh()
            .x_labels(8)
            .x_label_formatter(&|i| {
                let i = *i;
                if i < series.dates.len() {
                    series.dates[i].format("%m-%d").to_string()
                } else {
                    String::new()
                }
            })
            .draw()
            .unwrap();

        chart
            .draw_series(LineSeries::new(
                series
                    .additions
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, *v as u64)),
                &GREEN,
            ))
            .unwrap()
            .label("additions")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 12, y)], &GREEN));

        chart
            .draw_series(LineSeries::new(
                series
                    .deletions
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, *v as u64)),
                &RED,
            ))
            .unwrap()
            .label("deletions")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 12, y)], &RED));

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .unwrap();

        root.present().expect("present svg");
    }
    buf
}

fn empty_svg() -> String {
    r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="80" font-family="sans-serif" font-size="12"><text x="10" y="40" fill="#586069">No commits in range.</text></svg>"##.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn empty_series_returns_svg() {
        let s = TrendSeries {
            dates: vec![],
            additions: vec![],
            deletions: vec![],
        };
        let svg = render(&s);
        assert!(svg.starts_with("<svg"));
    }

    #[test]
    fn single_day_renders() {
        let d = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let s = TrendSeries {
            dates: vec![d],
            additions: vec![10],
            deletions: vec![2],
        };
        let svg = render(&s);
        assert!(svg.contains("<svg"));
        assert!(svg.contains("additions"));
    }
}