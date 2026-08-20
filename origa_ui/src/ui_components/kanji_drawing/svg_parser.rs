use tracing::debug;

use super::StrokeData;

pub fn parse_stroke_paths(svg: &str) -> Vec<StrokeData> {
    let mut strokes = Vec::new();
    let mut pos = 0;
    while let Some(rel_start) = svg[pos..].find("<path") {
        let abs_start = pos + rel_start;
        let rest = &svg[abs_start..];
        let tag_end = rest.find('>').unwrap_or(rest.len());
        let path_tag = &rest[..=tag_end.min(rest.len() - 1)];
        if !path_tag.contains("class=\"bg\"")
            && !path_tag.contains("class='bg'")
            && let Some(d) = extract_attribute(path_tag, "d")
        {
            debug!(stroke_index = strokes.len(), d = %d, "Parsed stroke path");
            strokes.push(StrokeData { d });
        }
        pos = abs_start + tag_end + 1;
    }
    debug!(
        total_strokes = strokes.len(),
        "Finished parsing SVG strokes"
    );
    strokes
}

fn extract_attribute(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{}=\"", attr), format!("{}='", attr)];
    for pattern in patterns {
        if let Some(start) = tag.find(&pattern) {
            let value_start = start + pattern.len();
            let quote = &pattern[pattern.len() - 1..pattern.len()];
            if let Some(end) = tag[value_start..].find(quote) {
                return Some(tag[value_start..value_start + end].to_string());
            }
        }
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCommand {
    MoveTo(f64, f64),
    LineTo(f64, f64),
    CurveTo(f64, f64, f64, f64, f64, f64),
    ClosePath,
}

pub fn parse_svg_path_commands(d: &str) -> Vec<PathCommand> {
    let mut commands = Vec::new();
    let chars: Vec<char> = d.chars().collect();
    let mut pos = 0;
    let mut current_cmd = 'M';
    let mut current_pos = (0.0_f64, 0.0_f64);

    while pos < chars.len() {
        let c = chars[pos];
        if c.is_ascii_alphabetic() {
            current_cmd = c;
            pos += 1;
        }
        match current_cmd {
            'M' | 'm' => {
                if let Some((cmd, new_pos, new_current, next_cmd)) =
                    parse_move(current_cmd, current_pos, &chars, pos)
                {
                    current_pos = new_current;
                    commands.push(cmd);
                    pos = new_pos;
                    current_cmd = next_cmd;
                } else {
                    break;
                }
            },
            'L' | 'l' => {
                if let Some((cmd, new_pos, new_current)) =
                    parse_line(current_cmd, current_pos, &chars, pos)
                {
                    current_pos = new_current;
                    commands.push(cmd);
                    pos = new_pos;
                } else {
                    break;
                }
            },
            'C' | 'c' => {
                if let Some((cmd, new_pos, new_current)) =
                    parse_curve(current_cmd, current_pos, &chars, pos)
                {
                    current_pos = new_current;
                    commands.push(cmd);
                    pos = new_pos;
                } else {
                    break;
                }
            },
            'Z' | 'z' => {
                commands.push(PathCommand::ClosePath);
                pos += 1;
            },
            _ => {
                pos += 1;
            },
        }
    }
    commands
}

fn parse_move(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<(PathCommand, usize, (f64, f64), char)> {
    let (x, y, new_pos) = parse_coords(chars, pos)?;
    let (abs_x, abs_y) = if cmd == 'm' {
        (current_pos.0 + x, current_pos.1 + y)
    } else {
        (x, y)
    };
    let next_cmd = if cmd == 'm' { 'l' } else { 'L' };
    Some((
        PathCommand::MoveTo(abs_x, abs_y),
        new_pos,
        (abs_x, abs_y),
        next_cmd,
    ))
}

fn parse_line(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<(PathCommand, usize, (f64, f64))> {
    let (x, y, new_pos) = parse_coords(chars, pos)?;
    let (abs_x, abs_y) = if cmd == 'l' {
        (current_pos.0 + x, current_pos.1 + y)
    } else {
        (x, y)
    };
    Some((PathCommand::LineTo(abs_x, abs_y), new_pos, (abs_x, abs_y)))
}

fn parse_curve(
    cmd: char,
    current_pos: (f64, f64),
    chars: &[char],
    pos: usize,
) -> Option<(PathCommand, usize, (f64, f64))> {
    let (x1, y1, x2, y2, x, y, new_pos) = parse_curve_coords(chars, pos)?;
    let (abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y) = if cmd == 'c' {
        (
            current_pos.0 + x1,
            current_pos.1 + y1,
            current_pos.0 + x2,
            current_pos.1 + y2,
            current_pos.0 + x,
            current_pos.1 + y,
        )
    } else {
        (x1, y1, x2, y2, x, y)
    };
    Some((
        PathCommand::CurveTo(abs_x1, abs_y1, abs_x2, abs_y2, abs_x, abs_y),
        new_pos,
        (abs_x, abs_y),
    ))
}

pub(crate) fn parse_coords(chars: &[char], start: usize) -> Option<(f64, f64, usize)> {
    let mut pos = skip_whitespace(chars, start);
    let (x, new_pos) = parse_number(chars, pos)?;
    pos = skip_whitespace(chars, new_pos);
    let (y, new_pos) = parse_number(chars, pos)?;
    Some((x, y, new_pos))
}

pub(crate) fn parse_curve_coords(
    chars: &[char],
    start: usize,
) -> Option<(f64, f64, f64, f64, f64, f64, usize)> {
    let (x1, pos) = parse_number(chars, skip_whitespace(chars, start))?;
    let (y1, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (x2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y2, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (x, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    let (y, pos) = parse_number(chars, skip_whitespace(chars, pos))?;
    Some((x1, y1, x2, y2, x, y, pos))
}

pub(crate) fn skip_whitespace(chars: &[char], start: usize) -> usize {
    let mut pos = start;
    while pos < chars.len() && (chars[pos].is_whitespace() || chars[pos] == ',') {
        pos += 1;
    }
    pos
}

pub(crate) fn parse_number(chars: &[char], start: usize) -> Option<(f64, usize)> {
    let mut pos = start;
    let mut num_str = String::new();
    if pos < chars.len() && (chars[pos] == '-' || chars[pos] == '+') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.') {
        num_str.push(chars[pos]);
        pos += 1;
    }
    let num: f64 = num_str.parse().ok()?;
    Some((num, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stroke_paths_extracts_d_attributes() {
        let svg = r#"<svg><path d="M 10 10 L 20 20"/><path d="M 1 2 C 3 4 5 6 7 8"/></svg>"#;
        let strokes = parse_stroke_paths(svg);
        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].d, "M 10 10 L 20 20");
        assert_eq!(strokes[1].d, "M 1 2 C 3 4 5 6 7 8");
    }

    #[test]
    fn parse_stroke_paths_skips_background_strokes() {
        let svg = r#"<path class="bg" d="M 0 0 L 1 1"/><path d="M 5 5 L 6 6"/>"#;
        let strokes = parse_stroke_paths(svg);
        assert_eq!(strokes.len(), 1, "bg stroke must be filtered out");
        assert_eq!(strokes[0].d, "M 5 5 L 6 6");
    }

    #[test]
    fn parse_stroke_paths_empty_input_yields_nothing() {
        assert!(parse_stroke_paths("").is_empty());
        assert!(parse_stroke_paths("<circle cx=\"1\"/>").is_empty());
    }

    #[test]
    fn parse_path_commands_absolute_move_line_close() {
        // Compact comma-separated format used by CDN kanji SVG files.
        let cmds = parse_svg_path_commands("M10,20L30,40Z");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 20.0),
                PathCommand::LineTo(30.0, 40.0),
                PathCommand::ClosePath,
            ]
        );
    }

    #[test]
    fn parse_path_commands_relative_coordinates() {
        // m 10 10 l 5 0 → MoveTo(10,10) then relative LineTo(15,10)
        let cmds = parse_svg_path_commands("m10,10l5,0");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 10.0),
                PathCommand::LineTo(15.0, 10.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_implicit_line_after_move() {
        // SVG allows implicit repetition: "M1,2 3,4" = M 1 2 L 3 4
        let cmds = parse_svg_path_commands("M1,2 3,4");
        assert_eq!(
            cmds,
            vec![PathCommand::MoveTo(1.0, 2.0), PathCommand::LineTo(3.0, 4.0),]
        );
    }

    #[test]
    fn parse_path_commands_absolute_cubic_bezier() {
        let cmds = parse_svg_path_commands("M0,0C1,2 3,4 5,6");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(0.0, 0.0),
                PathCommand::CurveTo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_relative_cubic_bezier_offsets_from_current() {
        let cmds = parse_svg_path_commands("M10,10c1,1 2,2 3,3");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(10.0, 10.0),
                PathCommand::CurveTo(11.0, 11.0, 12.0, 12.0, 13.0, 13.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_negative_and_comma_separated_numbers() {
        let cmds = parse_svg_path_commands("M-5.5,-2.5L+3,4");
        assert_eq!(
            cmds,
            vec![
                PathCommand::MoveTo(-5.5, -2.5),
                PathCommand::LineTo(3.0, 4.0),
            ]
        );
    }

    #[test]
    fn parse_path_commands_real_cdn_stroke_path() {
        // Verbatim fragment of cdn/kanji_animations/丁.svg — the actual
        // production input format.
        let d = "M14,24.17c2.44,0.56,6.92,0.82,9.35,0.56c18.9-1.99,39.53-5.36,60.62-6.48";
        let cmds = parse_svg_path_commands(d);
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], PathCommand::MoveTo(14.0, 24.17));

        // Relative curve: offsets added to the current point (14, 24.17).
        // f64 addition carries float error (24.17+0.82), so compare with a
        // tolerance instead of exact equality.
        match cmds[1] {
            PathCommand::CurveTo(x1, y1, x2, y2, x, y) => {
                assert!((x1 - 16.44).abs() < 1e-9);
                assert!((y1 - 24.73).abs() < 1e-9);
                assert!((x2 - 20.92).abs() < 1e-9);
                assert!((y2 - 24.99).abs() < 1e-9);
                assert!((x - 23.35).abs() < 1e-9);
                assert!((y - 24.73).abs() < 1e-9);
            },
            other => panic!("expected CurveTo, got {other:?}"),
        }
    }

    #[test]
    fn parse_path_commands_empty_and_noise_input() {
        assert!(parse_svg_path_commands("").is_empty());
        // Unknown command letters are skipped without breaking the parser.
        assert!(parse_svg_path_commands("Q 1 2 3 4").is_empty());
    }
}
