use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
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
    application::{CompleteLessonUseCase, RateCardUseCase, StartStudySessionUseCase},
    cli::{furigana_renderer, render_once},
    domain::{JeersError, Rating, value_objects::StudySessionItem},
    settings::ApplicationEnvironment,
};

enum CardState {
    Question,
    Answer,
    Completed,
}

struct LearnCardApp {
    card: StudySessionItem,
    state: CardState,
    exit: bool,
    exit_session: bool,
    furigana_shown: bool,
    similarity_shown: bool,
    current_index: usize,
    total_count: usize,
}

impl LearnCardApp {
    fn new(
        card: StudySessionItem,
        current_index: usize,
        total_count: usize,
        furigana_force: bool,
        similarity_force: bool,
    ) -> Self {
        Self {
            card,
            state: CardState::Question,
            exit: false,
            exit_session: false,
            furigana_shown: furigana_force,
            similarity_shown: similarity_force,
            current_index,
            total_count,
        }
    }

    async fn run(&mut self) -> io::Result<(Option<Rating>, bool)> {
        let mut terminal = ratatui::init();
        let mut rating = None;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut rating).await?;
        }

        ratatui::restore();
        Ok((rating, self.exit_session))
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let (main_area, footer_area) = self.create_vertical_layout(area);
        let (card_area, similarity_area) = self.create_horizontal_layout(main_area);

        self.draw_card(frame, card_area);
        if let Some(similarity_area) = similarity_area {
            self.draw_similarity(frame, similarity_area);
        }
        self.draw_footer(frame, footer_area);
    }

    fn create_vertical_layout(&self, area: Rect) -> (Rect, Rect) {
        const FOOTER_HEIGHT: u16 = 1;
        let layout =
            Layout::vertical([Constraint::Min(0), Constraint::Length(FOOTER_HEIGHT)]).split(area);
        (layout[0], layout[1])
    }

    fn create_horizontal_layout(&self, main_area: Rect) -> (Rect, Option<Rect>) {
        let layout = if self.similarity_shown {
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_area)
        } else {
            Layout::horizontal([Constraint::Percentage(100)]).split(main_area)
        };

        let card_area = layout[0];
        let similarity_area = if layout.len() > 1 {
            Some(layout[1])
        } else {
            None
        };
        (card_area, similarity_area)
    }

    fn draw_card(&self, frame: &mut Frame, area: Rect) {
        let card_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Green));

        let card_content = self.build_card_content();
        Paragraph::new(Text::from(card_content))
            .block(card_block)
            .alignment(Alignment::Left)
            .render(area, frame.buffer_mut());
    }

    fn build_card_content(&self) -> Vec<Line<'_>> {
        match &self.state {
            CardState::Question => self.build_question_content(),
            CardState::Answer => self.build_answer_content(),
            CardState::Completed => self.build_completed_content(),
        }
    }

    fn build_question_content(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];
        lines.push(self.render_question_line(Color::Magenta));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Нажмите пробел чтобы показать ответ.".fg(Color::Gray),
        ));
        lines.push(self.build_similarity_hint());
        if self.card.furigana().is_some() {
            lines.push(self.build_furigana_hint());
        }
        lines.push(Line::from(
            "Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray),
        ));
        lines.push(Line::from("Нажмите \"q\" чтобы выйти.".fg(Color::Gray)));
        lines
    }

    fn build_answer_content(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];
        lines.push(self.render_question_line(Color::Blue));
        lines.push(Line::from(self.card.answer().bold().fg(Color::Magenta)));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Используйте цифры от 1 до 4 для оценки карточки.".fg(Color::Gray),
        ));
        lines.push(Line::from("1 - Легко".fg(Color::Gray)));
        lines.push(Line::from("2 - Нормально".fg(Color::Gray)));
        lines.push(Line::from("3 - Трудно".fg(Color::Gray)));
        lines.push(Line::from("4 - Очень трудно".fg(Color::Gray)));
        lines.push(self.build_similarity_hint());
        lines.push(Line::from(
            "Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray),
        ));
        lines.push(Line::from("Нажмите \"q\" чтобы выйти.".fg(Color::Gray)));
        lines
    }

    fn build_completed_content(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];
        lines.push(self.render_question_line(Color::Blue));
        lines.push(Line::from(self.card.answer().bold().fg(Color::Magenta)));
        lines
    }

    fn render_question_line(&self, color: Color) -> Line<'_> {
        if self.furigana_shown {
            if let Some(question_furigana) = self.card.furigana() {
                return furigana_renderer::render_furigana(question_furigana);
            }
        }
        Line::from(self.card.question().bold().fg(color))
    }

    fn build_similarity_hint(&self) -> Line<'_> {
        if self.similarity_shown {
            Line::from("Нажмите \"h\" чтобы скрыть связанные карточки.".fg(Color::Gray))
        } else {
            Line::from("Нажмите \"h\" чтобы показать связанные карточки.".fg(Color::Gray))
        }
    }

    fn build_furigana_hint(&self) -> Line<'_> {
        if self.furigana_shown {
            Line::from("Нажмите \"f\" чтобы скрыть фуригану.".fg(Color::Gray))
        } else {
            Line::from("Нажмите \"f\" чтобы показать фуригану.".fg(Color::Gray))
        }
    }

    fn draw_similarity(&self, frame: &mut Frame, area: Rect) {
        if !self.similarity_shown {
            return;
        }

        let similarity_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Yellow))
            .title("Связанные карточки");

        let similarity_lines = self.build_similarity_lines();
        Paragraph::new(Text::from(similarity_lines))
            .block(similarity_block)
            .alignment(Alignment::Left)
            .render(area, frame.buffer_mut());
    }

    fn build_similarity_lines(&self) -> Vec<Line<'_>> {
        let mut lines = vec![Line::from("")];

        if self.card.similarity().is_empty() {
            lines.push(Line::from("Связанные карточки не найдены.".fg(Color::Gray)));
        } else {
            for similar_card in self.card.similarity() {
                lines.push(self.render_similar_card_question(similar_card));
                if self.should_show_answer() {
                    lines.push(Line::from(
                        format!("  {}", similar_card.answer()).fg(Color::Magenta),
                    ));
                }
                lines.push(Line::from(""));
            }
        }
        lines
    }

    fn render_similar_card_question(&self, similar_card: &StudySessionItem) -> Line<'_> {
        if self.furigana_shown
            && let Some(furigana) = similar_card.furigana()
        {
            furigana_renderer::render_furigana(furigana)
        } else {
            Line::from(format!("• {}", similar_card.question()).fg(Color::Cyan))
        }
    }

    fn should_show_answer(&self) -> bool {
        matches!(self.state, CardState::Answer | CardState::Completed)
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let remaining = self.total_count - self.current_index;
        let progress_text = format!(
            "Карточка {} из {} (осталось: {})",
            self.current_index + 1,
            self.total_count,
            remaining
        );
        let progress_line = Line::from(progress_text.fg(Color::Cyan));
        Paragraph::new(Text::from(vec![progress_line]))
            .alignment(Alignment::Center)
            .render(area, frame.buffer_mut());
    }

    async fn handle_events(&mut self, rating: &mut Option<Rating>) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key(key_event.code, rating);
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key_code: KeyCode, rating: &mut Option<Rating>) {
        match key_code {
            KeyCode::Char(' ') => self.handle_space_key(),
            KeyCode::Char('h') => self.handle_h_key(),
            KeyCode::Char('f') => self.handle_f_key(),
            KeyCode::Char('s') => self.handle_s_key(),
            KeyCode::Char('q') => self.handle_q_key(),
            KeyCode::Char('1') => self.handle_rating_key(rating, Rating::Easy),
            KeyCode::Char('2') => self.handle_rating_key(rating, Rating::Good),
            KeyCode::Char('3') => self.handle_rating_key(rating, Rating::Hard),
            KeyCode::Char('4') => self.handle_rating_key(rating, Rating::Again),
            _ => {}
        }
    }

    fn handle_space_key(&mut self) {
        if matches!(self.state, CardState::Question) {
            self.state = CardState::Answer;
        }
    }

    fn handle_h_key(&mut self) {
        if matches!(self.state, CardState::Question | CardState::Answer) {
            self.similarity_shown = !self.similarity_shown;
        }
    }

    fn handle_f_key(&mut self) {
        if matches!(self.state, CardState::Question | CardState::Answer)
            && self.card.furigana().is_some()
        {
            self.furigana_shown = !self.furigana_shown;
        }
    }

    fn handle_s_key(&mut self) {
        self.exit = true;
    }

    fn handle_q_key(&mut self) {
        self.exit = true;
        self.exit_session = true;
    }

    fn handle_rating_key(&mut self, rating: &mut Option<Rating>, new_rating: Rating) {
        if matches!(self.state, CardState::Answer) {
            *rating = Some(new_rating);
            self.state = CardState::Completed;
            self.exit = true;
        }
    }
}

fn render_empty_cards_message() -> Result<(), JeersError> {
    render_once(
        |frame| {
            let area = frame.area();
            let block = Block::bordered()
                .border_set(border::ROUNDED)
                .border_style(Style::default().fg(Color::Red));
            let text = Text::from(vec![Line::from("Вы всё выучили!".bold().fg(Color::Red))]);
            Paragraph::new(text)
                .block(block)
                .alignment(Alignment::Center)
                .render(area, frame.buffer_mut());
        },
        10,
    )
    .map_err(|e| JeersError::RepositoryError {
        reason: e.to_string(),
    })
}

pub async fn handle_learn(
    user_id: Ulid,
    new_cards_force: bool,
    furigana_force: bool,
    similarity_force: bool,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    let start_study_usecase = StartStudySessionUseCase::new(
        settings.get_repository().await?,
        settings.get_furigana_service().await?,
    );
    let cards = start_study_usecase
        .execute(user_id, new_cards_force)
        .await?;

    if cards.is_empty() {
        render_empty_cards_message()?;
        return Ok(());
    }

    let srs_service = settings.get_srs_service().await?;
    let rate_usecase = RateCardUseCase::new(settings.get_repository().await?, srs_service);

    let total_count = cards.len();
    for (index, card) in cards.iter().enumerate() {
        let mut app = LearnCardApp::new(
            card.clone(),
            index,
            total_count,
            furigana_force,
            similarity_force,
        );

        let (rating, exit_session) = app.run().await.map_err(|e| JeersError::RepositoryError {
            reason: e.to_string(),
        })?;

        if let Some(rating) = rating {
            if let Err(e) = rate_usecase.execute(user_id, card.card_id(), rating).await {
                eprintln!("Error rating card: {:?}", e);
            }
        }

        if exit_session {
            break;
        }
    }

    let complete_lesson_usecase = CompleteLessonUseCase::new(settings.get_repository().await?);
    if let Err(e) = complete_lesson_usecase.execute(user_id).await {
        eprintln!("Error completing lesson: {:?}", e);
    }

    Ok(())
}
