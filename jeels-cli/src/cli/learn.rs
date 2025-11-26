use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::{
        FindSynonymsUseCase, FuriganaService, GetFuriganaUseCase, RateCardUseCase,
        StartStudySessionUseCase, UserRepository,
    },
    cli::{furigana_renderer, render_once},
    domain::{Card, JeersError, Rating},
    settings::ApplicationEnvironment,
};

enum CardState {
    Question,
    Answer,
    Completed,
}

struct LearnCardApp<'a, R: UserRepository, F: FuriganaService> {
    card: Card,
    state: CardState,
    exit: bool,
    user_id: Ulid,
    repository: &'a R,
    furigana_service: &'a F,
    synonyms: Option<Vec<Card>>,
    furigana_data: Option<String>,
    furigana_shown: bool,
    current_index: usize,
    total_count: usize,
}

impl<'a, R: UserRepository, F: FuriganaService> LearnCardApp<'a, R, F> {
    fn new(
        card: Card,
        user_id: Ulid,
        repository: &'a R,
        furigana_service: &'a F,
        current_index: usize,
        total_count: usize,
    ) -> Self {
        Self {
            card,
            state: CardState::Question,
            exit: false,
            user_id,
            repository,
            furigana_service,
            synonyms: None,
            furigana_data: None,
            furigana_shown: false,
            current_index,
            total_count,
        }
    }

    async fn run(&mut self) -> io::Result<Option<Rating>> {
        let mut terminal = ratatui::init();
        let mut rating = None;

        let get_furigana_usecase = GetFuriganaUseCase::new(self.repository, self.furigana_service);
        if let Ok(furigana) = get_furigana_usecase
            .execute(self.user_id, self.card.id())
            .await
        {
            if furigana != self.card.question().text() {
                self.furigana_data = Some(furigana);
            }
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut rating).await?;
        }

        ratatui::restore();
        Ok(rating)
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let vertical_layout =
            Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let main_area = vertical_layout[0];
        let footer_area = vertical_layout[1];

        let layout = if self.synonyms.is_some() {
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_area)
        } else {
            Layout::horizontal([Constraint::Percentage(100)]).split(main_area)
        };

        let card_area = layout[0];
        let synonyms_area = if layout.len() > 1 {
            Some(layout[1])
        } else {
            None
        };

        // Отрисовка карточки
        let card_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Green));

        let card_content = match &self.state {
            CardState::Question => {
                let mut lines = vec![];
                if self.furigana_shown {
                    if let Some(question_furigana) = &self.furigana_data {
                        lines.push(furigana_renderer::render_furigana(question_furigana));
                        lines.push(Line::from(""));
                    }
                }
                lines.push(Line::from(
                    self.card.question().text().bold().fg(Color::Magenta),
                ));
                lines.push(Line::from(""));
                lines.push(Line::from(
                    "Нажмите пробел чтобы показать ответ.".fg(Color::Gray),
                ));
                if self.synonyms.is_some() {
                    lines.push(Line::from(
                        "Нажмите \"h\" чтобы скрыть синонимы.".fg(Color::Gray),
                    ));
                } else {
                    lines.push(Line::from(
                        "Нажмите \"h\" чтобы показать синонимы.".fg(Color::Gray),
                    ));
                }
                if self.furigana_data.is_some() {
                    if self.furigana_shown {
                        lines.push(Line::from(
                            "Нажмите \"f\" чтобы скрыть фуригану.".fg(Color::Gray),
                        ));
                    } else {
                        lines.push(Line::from(
                            "Нажмите \"f\" чтобы показать фуригану.".fg(Color::Gray),
                        ));
                    }
                }
                lines.push(Line::from(
                    "Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray),
                ));
                lines
            }
            CardState::Answer => {
                let mut lines = vec![];
                if self.furigana_shown {
                    if let Some(question_furigana) = &self.furigana_data {
                        lines.push(furigana_renderer::render_furigana(question_furigana));
                        lines.push(Line::from(""));
                    }
                }
                lines.push(Line::from(
                    self.card.question().text().bold().fg(Color::Blue),
                ));
                lines.push(Line::from(
                    self.card.answer().text().bold().fg(Color::Magenta),
                ));
                lines.push(Line::from(""));
                lines.push(Line::from(
                    "Используйте цифры от 1 до 4 для оценки карточки.".fg(Color::Gray),
                ));
                lines.push(Line::from("1 - Легко".fg(Color::Gray)));
                lines.push(Line::from("2 - Нормально".fg(Color::Gray)));
                lines.push(Line::from("3 - Трудно".fg(Color::Gray)));
                lines.push(Line::from("4 - Очень трудно".fg(Color::Gray)));
                lines.push(Line::from(
                    "Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray),
                ));
                lines
            }
            CardState::Completed => {
                let mut lines = vec![];
                if self.furigana_shown {
                    if let Some(question_furigana) = &self.furigana_data {
                        lines.push(furigana_renderer::render_furigana(question_furigana));
                        lines.push(Line::from(""));
                    }
                }
                lines.push(Line::from(
                    self.card.question().text().bold().fg(Color::Blue),
                ));
                lines.push(Line::from(
                    self.card.answer().text().bold().fg(Color::Magenta),
                ));
                lines
            }
        };

        Paragraph::new(Text::from(card_content))
            .block(card_block)
            .alignment(Alignment::Left)
            .render(card_area, frame.buffer_mut());

        if let Some(synonyms_area) = synonyms_area {
            if let Some(synonyms) = &self.synonyms {
                let synonyms_block = Block::bordered()
                    .border_set(border::ROUNDED)
                    .border_style(Style::default().fg(Color::Yellow))
                    .title("Синонимы");

                let mut synonyms_lines = vec![Line::from("")];

                if synonyms.is_empty() {
                    synonyms_lines.push(Line::from("Синонимы не найдены.".fg(Color::Gray)));
                } else {
                    for synonym in synonyms {
                        synonyms_lines.push(Line::from(
                            format!("• {}", synonym.question().text()).fg(Color::Cyan),
                        ));
                    }
                }

                Paragraph::new(Text::from(synonyms_lines))
                    .block(synonyms_block)
                    .alignment(Alignment::Left)
                    .render(synonyms_area, frame.buffer_mut());
            }
        }

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
            .render(footer_area, frame.buffer_mut());
    }

    async fn handle_events(&mut self, rating: &mut Option<Rating>) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char(' ') => {
                        if matches!(self.state, CardState::Question) {
                            self.state = CardState::Answer;
                        }
                    }
                    KeyCode::Char('h') => {
                        if matches!(self.state, CardState::Question)
                            || matches!(self.state, CardState::Answer)
                        {
                            if self.synonyms.is_some() {
                                self.synonyms = None;
                            } else {
                                let find_synonyms_usecase =
                                    FindSynonymsUseCase::new(self.repository);
                                match find_synonyms_usecase
                                    .execute(self.user_id, self.card.id())
                                    .await
                                {
                                    Ok(synonyms) => {
                                        self.synonyms = Some(synonyms);
                                    }
                                    Err(_) => {
                                        self.synonyms = Some(vec![]);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char('f') => {
                        if matches!(self.state, CardState::Question)
                            || matches!(self.state, CardState::Answer)
                        {
                            if self.furigana_data.is_some() {
                                self.furigana_shown = !self.furigana_shown;
                            }
                        }
                    }
                    KeyCode::Char('s') => {
                        self.exit = true;
                    }
                    KeyCode::Char('1') => {
                        if matches!(self.state, CardState::Answer) {
                            *rating = Some(Rating::Easy);
                            self.state = CardState::Completed;
                            self.exit = true;
                        }
                    }
                    KeyCode::Char('2') => {
                        if matches!(self.state, CardState::Answer) {
                            *rating = Some(Rating::Good);
                            self.state = CardState::Completed;
                            self.exit = true;
                        }
                    }
                    KeyCode::Char('3') => {
                        if matches!(self.state, CardState::Answer) {
                            *rating = Some(Rating::Hard);
                            self.state = CardState::Completed;
                            self.exit = true;
                        }
                    }
                    KeyCode::Char('4') => {
                        if matches!(self.state, CardState::Answer) {
                            *rating = Some(Rating::Again);
                            self.state = CardState::Completed;
                            self.exit = true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub async fn handle_learn(user_id: Ulid, force_new_cards: bool) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    let start_study_usecase = StartStudySessionUseCase::new(settings.get_repository().await?);
    let cards = start_study_usecase.execute(user_id, force_new_cards).await?;

    if cards.is_empty() {
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
        })?;
        return Ok(());
    }

    let repository = settings.get_repository().await?;
    let srs_service = settings.get_srs_service().await?;
    let rate_usecase = RateCardUseCase::new(repository, srs_service);

    let total_count = cards.len();
    for (index, card) in cards.iter().enumerate() {
        let repository = settings.get_repository().await?;
        let furigana_service = settings.get_furigana_service().await?;
        let mut app = LearnCardApp::new(
            card.clone(),
            user_id,
            repository,
            furigana_service,
            index,
            total_count,
        );
        let rating = app.run().await.map_err(|e| JeersError::RepositoryError {
            reason: e.to_string(),
        })?;

        if let Some(rating) = rating {
            if let Err(e) = rate_usecase.execute(user_id, card.id(), rating).await {
                eprintln!("Error rating card: {:?}", e);
            }
        }
    }

    Ok(())
}
