use tracing::debug;

use super::svg_parser::{PathCommand, parse_svg_path_commands};
use super::{CANVAS_TOLERANCE, MIN_STROKE_POINTS, SAMPLE_COUNT, SUCCESS_THRESHOLD, SVG_SCALE};

fn bounding_box(points: &[(f64, f64)]) -> (f64, f64, f64, f64) {
    points.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), &(x, y)| {
            (min_x.min(x), max_x.max(x), min_y.min(y), max_y.max(y))
        },
    )
}

pub fn is_stroke_similar(user_points: &[(f64, f64)], stroke_d: &str) -> bool {
    debug!("[Stroke Check] user_points count: {}", user_points.len());

    if user_points.len() < MIN_STROKE_POINTS {
        debug!(
            "[Stroke Check] FAIL: too few points ({})",
            user_points.len()
        );
        return false;
    }

    let stroke_samples = sample_stroke_path(stroke_d);
    debug!(
        "[Stroke Check] stroke_samples count: {}",
        stroke_samples.len()
    );

    if stroke_samples.len() < SAMPLE_COUNT {
        debug!("[Stroke Check] FAIL: too few stroke samples");
        return false;
    }

    if !user_points.is_empty() {
        let (min_x, max_x, min_y, max_y) = bounding_box(user_points);
        debug!(
            "[Stroke Check] user raw range: x=[{:.1}, {:.1}], y=[{:.1}, {:.1}]",
            min_x, max_x, min_y, max_y
        );
    }

    if !stroke_samples.is_empty() {
        let (min_x, max_x, min_y, max_y) = bounding_box(&stroke_samples);
        debug!(
            "[Stroke Check] stroke raw range: x=[{:.1}, {:.1}], y=[{:.1}, {:.1}]",
            min_x, max_x, min_y, max_y
        );
    }

    let stroke_resampled = resample_points(&stroke_samples, SAMPLE_COUNT);

    let covered_count = stroke_resampled
        .iter()
        .filter(|stroke_point| {
            user_points.iter().any(|user_point| {
                let dx = user_point.0 - stroke_point.0;
                let dy = user_point.1 - stroke_point.1;
                (dx * dx + dy * dy).sqrt() <= CANVAS_TOLERANCE
            })
        })
        .count();

    let coverage_ratio = covered_count as f64 / stroke_resampled.len() as f64;
    debug!(
        "[Stroke Check] coverage: {}/{} = {:.2}, threshold: {:.2}",
        covered_count,
        stroke_resampled.len(),
        coverage_ratio,
        SUCCESS_THRESHOLD
    );

    let result = coverage_ratio >= SUCCESS_THRESHOLD;
    debug!("[Stroke Check] RESULT: {}", result);

    result
}

fn sample_stroke_path(d: &str) -> Vec<(f64, f64)> {
    let mut points = Vec::new();
    let mut current_pos = (0.0_f64, 0.0_f64);
    let mut start_pos = (0.0_f64, 0.0_f64);
    let commands = parse_svg_path_commands(d);

    for cmd in &commands {
        match cmd {
            PathCommand::MoveTo(x, y) => {
                current_pos = (*x, *y);
                start_pos = current_pos;
                points.push((x * SVG_SCALE, y * SVG_SCALE));
            },
            PathCommand::LineTo(x, y) => {
                let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                let end = (x * SVG_SCALE, y * SVG_SCALE);
                interpolate_line(&mut points, start, end);
                current_pos = (*x, *y);
            },
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                let ctrl1 = (x1 * SVG_SCALE, y1 * SVG_SCALE);
                let ctrl2 = (x2 * SVG_SCALE, y2 * SVG_SCALE);
                let end = (x * SVG_SCALE, y * SVG_SCALE);
                interpolate_bezier(&mut points, start, ctrl1, ctrl2, end);
                current_pos = (*x, *y);
            },
            PathCommand::ClosePath => {
                let start = (current_pos.0 * SVG_SCALE, current_pos.1 * SVG_SCALE);
                let end = (start_pos.0 * SVG_SCALE, start_pos.1 * SVG_SCALE);
                interpolate_line(&mut points, start, end);
                current_pos = start_pos;
            },
        }
    }
    resample_points(&points, SAMPLE_COUNT)
}

fn interpolate_line(points: &mut Vec<(f64, f64)>, start: (f64, f64), end: (f64, f64)) {
    let steps = 10;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let x = start.0 + (end.0 - start.0) * t;
        let y = start.1 + (end.1 - start.1) * t;
        points.push((x, y));
    }
}

fn interpolate_bezier(
    points: &mut Vec<(f64, f64)>,
    p0: (f64, f64),
    p1: (f64, f64),
    p2: (f64, f64),
    p3: (f64, f64),
) {
    let steps = 20;
    for i in 1..=steps {
        let t = i as f64 / steps as f64;
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        let x = mt3 * p0.0 + 3.0 * mt2 * t * p1.0 + 3.0 * mt * t2 * p2.0 + t3 * p3.0;
        let y = mt3 * p0.1 + 3.0 * mt2 * t * p1.1 + 3.0 * mt * t2 * p2.1 + t3 * p3.1;
        points.push((x, y));
    }
}

fn resample_points(points: &[(f64, f64)], count: usize) -> Vec<(f64, f64)> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::with_capacity(count);
    let total_points = points.len();
    for i in 0..count {
        let idx = (i * (total_points - 1)) / (count - 1).max(1);
        result.push(points[idx.min(total_points - 1)]);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A dense point cloud along a horizontal line in canvas space.
    fn horizontal_points(y: f64, x0: f64, x1: f64, steps: usize) -> Vec<(f64, f64)> {
        (0..=steps)
            .map(|i| {
                let t = i as f64 / steps as f64;
                (x0 + (x1 - x0) * t, y)
            })
            .collect()
    }

    #[test]
    fn stroke_similar_exact_line_traced_matches() {
        // d="M10,10L99,10" in viewBox 109 → canvas coords ×(320/109)
        let scale = 320.0 / 109.0;
        let y = 10.0 * scale;
        let user = horizontal_points(y, 10.0 * scale, 99.0 * scale, 30);
        assert!(
            is_stroke_similar(&user, "M10,10L99,10"),
            "a dense trace of the target line must match"
        );
    }

    #[test]
    fn stroke_similar_far_away_line_does_not_match() {
        let user = horizontal_points(250.0, 30.0, 280.0, 30);
        assert!(
            !is_stroke_similar(&user, "M10,10L99,10"),
            "a line on the opposite side of the canvas must not match"
        );
    }

    #[test]
    fn stroke_similar_too_few_points_rejected() {
        let user = horizontal_points(30.0, 30.0, 280.0, 3);
        assert!(
            !is_stroke_similar(&user, "M10,10L99,10"),
            "fewer than MIN_STROKE_POINTS must be rejected outright"
        );
    }

    #[test]
    fn stroke_similar_partial_coverage_below_threshold_rejected() {
        // Cover only the left ~40% of the stroke: coverage ≈ 0.4 < 0.8.
        let scale = 320.0 / 109.0;
        let y = 10.0 * scale;
        let user = horizontal_points(y, 10.0 * scale, 45.0 * scale, 30);
        assert!(
            !is_stroke_similar(&user, "M10,10L99,10"),
            "partial trace below the success threshold must fail"
        );
    }

    #[test]
    fn stroke_similar_bezier_traced_within_tolerance_matches() {
        // A shallow curve approximated by its chord stays within the
        // 25px canvas tolerance of the true bezier samples.
        let scale = 320.0 / 109.0;
        let y0 = 30.0 * scale;
        let y1 = 50.0 * scale;
        let user = (0..=30)
            .map(|i| {
                let t = i as f64 / 30.0;
                (10.0 * scale + (99.0 - 10.0) * scale * t, y0 + (y1 - y0) * t)
            })
            .collect::<Vec<_>>();
        assert!(
            is_stroke_similar(&user, "M10,30C40,30 70,50 99,50"),
            "chord trace of a shallow bezier must match within tolerance"
        );
    }
}
