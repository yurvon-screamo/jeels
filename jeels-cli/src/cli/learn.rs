use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use crate::{
    application::{FindSynonymsUseCase, RateCardUseCase, StartStudySessionUseCase, UserRepository},
    cli::render_once,
    domain::{Card, JeersError, Rating},
    settings::ApplicationEnvironment,
};

enum CardState {
    Question,
    Answer,
    Completed,
    Synonyms(Vec<Card>),
}

struct LearnCardApp<'a, R: UserRepository> {
    card: Card,
    state: CardState,
    exit: bool,
    user_id: Ulid,
    repository: &'a R,
}

impl<'a, R: UserRepository> LearnCardApp<'a, R> {
    fn new(card: Card, user_id: Ulid, repository: &'a R) -> Self {
        Self {
            card,
            state: CardState::Question,
            exit: false,
            user_id,
            repository,
        }
    }

    async fn run(&mut self) -> io::Result<Option<Rating>> {
        let mut terminal = ratatui::init();
        let mut rating = None;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events(&mut rating).await?;
        }

        ratatui::restore();
        Ok(rating)
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Green));

        let content = match &self.state {
            CardState::Question => {
                vec![
                    Line::from(self.card.question().text().bold().fg(Color::Magenta)),
                    Line::from(""),
                    Line::from("Нажмите пробел чтобы показать ответ.".fg(Color::Gray)),
                    Line::from("Нажмите \"h\" чтобы показать синонимы.".fg(Color::Gray)),
                    Line::from("Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray)),
                ]
            }
            CardState::Answer => {
                vec![
                    Line::from(self.card.question().text().bold().fg(Color::Blue)),
                    Line::from(self.card.answer().text().bold().fg(Color::Magenta)),
                    Line::from(""),
                    Line::from("Используйте цифры от 1 до 4 для оценки карточки.".fg(Color::Gray)),
                    Line::from("1 - Легко".fg(Color::Gray)),
                    Line::from("2 - Нормально".fg(Color::Gray)),
                    Line::from("3 - Трудно".fg(Color::Gray)),
                    Line::from("4 - Очень трудно".fg(Color::Gray)),
                    Line::from("Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray)),
                ]
            }
            CardState::Completed => {
                vec![
                    Line::from(self.card.question().text().bold().fg(Color::Blue)),
                    Line::from(self.card.answer().text().bold().fg(Color::Magenta)),
                ]
            }
            CardState::Synonyms(synonyms) => {
                let mut lines = vec![
                    Line::from("Синонимы:".bold().fg(Color::Yellow)),
                    Line::from(""),
                ];

                if synonyms.is_empty() {
                    lines.push(Line::from("Синонимы не найдены.".fg(Color::Gray)));
                } else {
                    for synonym in synonyms {
                        lines.push(Line::from(
                            format!("• {}", synonym.question().text()).fg(Color::Cyan),
                        ));
                    }
                }

                lines.push(Line::from(""));
                lines.push(Line::from(
                    "Нажмите пробел чтобы вернуться к вопросу.".fg(Color::Gray),
                ));

                lines
            }
        };

        Paragraph::new(Text::from(content))
            .block(block)
            .alignment(Alignment::Left)
            .render(area, frame.buffer_mut());
    }

    async fn handle_events(&mut self, rating: &mut Option<Rating>) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char(' ') => match self.state {
                        CardState::Question => {
                            self.state = CardState::Answer;
                        }
                        CardState::Synonyms(_) => {
                            self.state = CardState::Question;
                        }
                        _ => {}
                    },
                    KeyCode::Char('h') => {
                        if matches!(self.state, CardState::Question) {
                            let find_synonyms_usecase = FindSynonymsUseCase::new(self.repository);
                            match find_synonyms_usecase
                                .execute(self.user_id, self.card.id())
                                .await
                            {
                                Ok(synonyms) => {
                                    self.state = CardState::Synonyms(synonyms);
                                }
                                Err(_) => {
                                    self.state = CardState::Synonyms(vec![]);
                                }
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

pub async fn handle_learn(user_id: Ulid) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    let start_study_usecase = StartStudySessionUseCase::new(settings.get_repository().await?);
    let cards = start_study_usecase.execute(user_id).await?;

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

    for card in cards {
        let repository = settings.get_repository().await?;
        let mut app = LearnCardApp::new(card.clone(), user_id, repository);
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
