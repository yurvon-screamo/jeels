use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::GetKanjiInfoUseCase,
    domain::{JeersError, dictionary::KanjiInfo},
};

use super::render_once;

pub async fn handle_kanji(_user_id: Ulid, kanji: String) -> Result<(), JeersError> {
    let use_case = GetKanjiInfoUseCase::new();
    let kanji_data = use_case.execute(&kanji)?;

    render_once(
        |frame| {
            let area = frame.area();
            render_kanji_card(&kanji_data, area, frame);
        },
        30,
    )
    .map_err(|e| JeersError::SettingsError {
        reason: e.to_string(),
    })?;

    Ok(())
}

fn render_kanji_card(kanji_data: &KanjiInfo, area: Rect, frame: &mut Frame) {
    let radicals_count = kanji_data.radicals().len();
    let radicals_height = if radicals_count > 0 {
        radicals_count * 3 + 2
    } else {
        3
    };

    let vertical = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(4),
        Constraint::Length(radicals_height as u16),
        Constraint::Min(0),
    ]);
    let [
        title_area,
        kanji_area,
        info_area,
        description_area,
        radicals_area,
        _,
    ] = vertical.areas(area);

    let info_horizontal =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
    let [jlpt_area, usage_area] = info_horizontal.areas(info_area);

    // Title
    let title = Line::from("Информация о кандзи:".bold().underlined());
    Paragraph::new(title)
        .alignment(Alignment::Left)
        .render(title_area, frame.buffer_mut());

    // Kanji character block
    let kanji_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Cyan))
        .title("Кандзи");

    let kanji_content = Text::from(vec![Line::from(kanji_data.kanji().to_string().bold())]);
    Paragraph::new(kanji_content)
        .block(kanji_block)
        .render(kanji_area, frame.buffer_mut());

    // JLPT level block
    Paragraph::new(Text::from(vec![Line::from(format!(
        "N{}",
        kanji_data.jlpt().as_number()
    ))]))
    .block(
        Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow))
            .title("JLPT уровень"),
    )
    .render(jlpt_area, frame.buffer_mut());

    // Usage count block
    Paragraph::new(Text::from(vec![Line::from(format!(
        "{}",
        kanji_data.used_in()
    ))]))
    .block(
        Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Используется в словах"),
    )
    .render(usage_area, frame.buffer_mut());

    // Description block
    let description_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Magenta))
        .title("Описание");

    let description_content = Text::from(vec![Line::from(kanji_data.description())]);
    Paragraph::new(description_content)
        .block(description_block)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(description_area, frame.buffer_mut());

    // Radicals block with detailed info
    let radicals_block = Block::bordered()
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(Color::Green))
        .title("Радикалы");

    let mut radicals_lines = Vec::new();
    if kanji_data.radicals().is_empty() {
        radicals_lines.push(Line::from("Информация о радикалах недоступна"));
    } else {
        for radical_info in kanji_data.radicals().iter() {
            radicals_lines.push(Line::from(format!(
                "{} - {}",
                radical_info.radical(),
                radical_info.name()
            )));
            radicals_lines.push(Line::from(format!("  {}", radical_info.description())));
            radicals_lines.push(Line::from(""));
        }
        radicals_lines.pop(); // Remove last empty line
    }

    let radicals_content = Text::from(radicals_lines);
    Paragraph::new(radicals_content)
        .block(radicals_block)
        .wrap(ratatui::widgets::Wrap { trim: true })
        .render(radicals_area, frame.buffer_mut());
}
