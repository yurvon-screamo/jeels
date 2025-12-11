use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

pub fn render_furigana(furigana_text: &str) -> Line<'static> {
    let mut spans = Vec::new();
    let mut current_pos = 0;
    let chars: Vec<char> = furigana_text.chars().collect();

    while current_pos < chars.len() {
        if chars[current_pos] == '[' {
            // Find matching ']'
            if let Some(end_bracket) = find_matching_bracket(&chars, current_pos, '[', ']') {
                let base_start = current_pos + 1;
                let base_text: String = chars[base_start..end_bracket].iter().collect();

                // Check if next is '{'
                if end_bracket + 1 < chars.len() && chars[end_bracket + 1] == '{' {
                    // Find matching '}'
                    if let Some(end_brace) =
                        find_matching_bracket(&chars, end_bracket + 1, '{', '}')
                    {
                        let furigana_start = end_bracket + 2;
                        let furigana_reading: String =
                            chars[furigana_start..end_brace].iter().collect();

                        spans.push(Span::styled(base_text, Style::default().fg(Color::Cyan)));
                        spans.push(Span::styled(
                            format!("[ {}]", furigana_reading),
                            Style::default().fg(Color::DarkGray),
                        ));

                        current_pos = end_brace + 1;
                        continue;
                    }
                }
                current_pos = end_bracket + 1;
                continue;
            }
        }

        // Regular character
        let mut regular_text = String::new();
        while current_pos < chars.len() && chars[current_pos] != '[' {
            regular_text.push(chars[current_pos]);
            current_pos += 1;
        }

        if !regular_text.is_empty() {
            spans.push(Span::styled(regular_text, Style::default().fg(Color::Cyan)));
        }
    }

    Line::from(spans)
}

fn find_matching_bracket(chars: &[char], start: usize, open: char, close: char) -> Option<usize> {
    if start >= chars.len() || chars[start] != open {
        return None;
    }

    let mut depth = 1;
    let mut pos = start + 1;

    while pos < chars.len() && depth > 0 {
        if chars[pos] == open {
            depth += 1;
        } else if chars[pos] == close {
            depth -= 1;
        }
        pos += 1;
    }

    if depth == 0 { Some(pos - 1) } else { None }
}
