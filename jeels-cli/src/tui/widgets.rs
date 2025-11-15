use crate::domain::Card;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn create_card_widget(card: &Card, show_answer: bool) -> Paragraph<'_> {
    let question_text = card.question().text();
    let answer_text = card.answer().text();

    let content = if show_answer {
        format!("Question: {}\n\nAnswer: {}", question_text, answer_text)
    } else {
        format!("Question: {}\n\nPress SPACE to show answer", question_text)
    };

    Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).title("Card"))
        .wrap(Wrap { trim: true })
}

pub fn create_empty_card_widget() -> Paragraph<'static> {
    Paragraph::new("No cards available")
        .block(Block::default().borders(Borders::ALL).title("Card"))
        .wrap(Wrap { trim: true })
}

pub fn get_instructions_text(show_answer: bool) -> &'static str {
    if show_answer {
        "1: Again | 2: Hard | 3: Good | 4: Easy | N/→: Next | Q/ESC: Quit"
    } else {
        "SPACE: Show Answer | N/→: Next | Q/ESC: Quit"
    }
}

pub fn get_status_text(no_cards: bool) -> &'static str {
    if no_cards {
        "No cards to study. All done!"
    } else {
        "Ready to study"
    }
}
