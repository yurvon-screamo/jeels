use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use ulid::Ulid;

use keikaku::{
    application::{CompleteLessonUseCase, RateCardUseCase, SelectCardsToLearnUseCase},
    domain::{
        JeersError, Rating,
        japanese::IsJapaneseText,
        kanji_card::ExampleKanjiWord,
        study_session::{KanjiStudySessionItem, StudySessionItem, VocabularyStudySessionItem},
        value_objects::ExamplePhrase,
    },
    settings::ApplicationEnvironment,
};
use crate::cli::{furigana_renderer, render_once};

enum CardState {
    Question,
    Answer,
    Completed,
}

enum CardExamples<'a> {
    Vocabulary(&'a [ExamplePhrase]),
    Kanji(&'a [ExampleKanjiWord]),
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
        let supports_relations = matches!(
            &card,
            StudySessionItem::Vocabulary(item)
                if !item.similarity().is_empty() || !item.homonyms().is_empty()
        );
        Self {
            card,
            state: CardState::Question,
            exit: false,
            exit_session: false,
            furigana_shown: furigana_force,
            similarity_shown: similarity_force && supports_relations,
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

    fn card_question_text(&self) -> String {
        match &self.card {
            StudySessionItem::Vocabulary(card) => card.word().to_string(),
            StudySessionItem::Kanji(card) => card.kanji().to_string(),
        }
    }

    fn card_answer_text(&self) -> String {
        match &self.card {
            StudySessionItem::Vocabulary(card) => card.meaning().to_string(),
            StudySessionItem::Kanji(card) => card.description().to_string(),
        }
    }

    fn card_similarity(&self) -> &[VocabularyStudySessionItem] {
        match &self.card {
            StudySessionItem::Vocabulary(card) => card.similarity(),
            StudySessionItem::Kanji(_) => &[] as &[VocabularyStudySessionItem],
        }
    }

    fn card_homonyms(&self) -> &[VocabularyStudySessionItem] {
        match &self.card {
            StudySessionItem::Vocabulary(card) => card.homonyms(),
            StudySessionItem::Kanji(_) => &[] as &[VocabularyStudySessionItem],
        }
    }

    fn card_examples(&self) -> CardExamples<'_> {
        match &self.card {
            StudySessionItem::Vocabulary(card) => CardExamples::Vocabulary(card.example_phrases()),
            StudySessionItem::Kanji(card) => CardExamples::Kanji(card.example_words()),
        }
    }

    fn card_kanji_info(&self) -> &[keikaku::domain::dictionary::KanjiInfo] {
        match &self.card {
            StudySessionItem::Vocabulary(card) => card.kanji(),
            StudySessionItem::Kanji(_) => &[] as &[keikaku::domain::dictionary::KanjiInfo],
        }
    }

    fn supports_relations(&self) -> bool {
        !self.card_similarity().is_empty() || !self.card_homonyms().is_empty()
    }

    fn related_cards_visible(&self) -> bool {
        self.supports_relations() && self.similarity_shown
    }

    fn has_furigana_content(&self) -> bool {
        let question = self.card_question_text();
        let answer = self.card_answer_text();
        question.has_furigana() || answer.has_furigana()
    }

    fn kanji_card(&self) -> Option<&KanjiStudySessionItem> {
        match &self.card {
            StudySessionItem::Kanji(card) => Some(card),
            _ => None,
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let (main_area, footer_area) = self.create_vertical_layout(area);
        let (card_area, right_panel_area) = self.create_horizontal_layout(main_area);

        let (main_card_area, kanji_grid_area) = self.create_card_and_kanji_layout(card_area);
        self.draw_card(frame, main_card_area);
        if let Some(kanji_grid_area) = kanji_grid_area {
            self.draw_kanji_grid(frame, kanji_grid_area);
        }
        if let Some(right_panel_area) = right_panel_area {
            let (similarity_area, homonyms_area) = self.create_right_panel_layout(right_panel_area);
            if let Some(similarity_area) = similarity_area {
                self.draw_similarity(frame, similarity_area);
            }
            if let Some(homonyms_area) = homonyms_area {
                self.draw_homonyms(frame, homonyms_area);
            }
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
        let similarity_shown = self.related_cards_visible();

        let layout = if similarity_shown {
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(main_area)
        } else {
            Layout::horizontal([Constraint::Percentage(100)]).split(main_area)
        };

        let card_area = layout[0];
        let right_panel_area = if similarity_shown && layout.len() > 1 {
            Some(layout[1])
        } else {
            None
        };
        (card_area, right_panel_area)
    }

    fn create_right_panel_layout(&self, right_panel_area: Rect) -> (Option<Rect>, Option<Rect>) {
        // Always show both panels when similarity_shown is true
        let layout = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(right_panel_area);
        (Some(layout[0]), Some(layout[1]))
    }

    fn create_card_and_kanji_layout(&self, card_area: Rect) -> (Rect, Option<Rect>) {
        let kanji_list = self.card_kanji_info();
        if kanji_list.is_empty() {
            return (card_area, None);
        }

        // Calculate kanji grid height based on number of kanji
        let kanji_count = kanji_list.len();
        let rows = kanji_count.div_ceil(3); // 3 kanji per row
        let kanji_height = (rows as u16 * 16).min(48); // Max 3 rows, 16 height per card

        let layout = Layout::vertical([Constraint::Min(0), Constraint::Length(kanji_height)])
            .split(card_area);

        (layout[0], Some(layout[1]))
    }

    fn draw_card(&self, frame: &mut Frame, area: Rect) {
        let card_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Green));

        let card_content = self.build_card_content();
        Paragraph::new(Text::from(card_content))
            .block(card_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true })
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
        if self.supports_relations() {
            lines.push(self.build_similarity_hint());
        }
        if self.has_furigana_content() {
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
        lines.push(self.render_answer_line());
        let example_lines = self.build_example_lines();
        if !example_lines.is_empty() {
            lines.push(Line::from(""));
            lines.extend(example_lines);
        }
        let kanji_details = self.build_kanji_details_lines();
        if !kanji_details.is_empty() {
            lines.push(Line::from(""));
            lines.extend(kanji_details);
        }
        lines.push(Line::from(""));
        lines.push(Line::from(
            "Используйте цифры от 1 до 4 для оценки карточки.".fg(Color::Gray),
        ));
        lines.push(Line::from("1 - Легко".fg(Color::Gray)));
        lines.push(Line::from("2 - Нормально".fg(Color::Gray)));
        lines.push(Line::from("3 - Трудно".fg(Color::Gray)));
        lines.push(Line::from("4 - Очень трудно".fg(Color::Gray)));
        if self.supports_relations() {
            lines.push(self.build_similarity_hint());
        }
        lines.push(Line::from(
            "Нажмите \"s\" чтобы пропустить карточку.".fg(Color::Gray),
        ));
        lines.push(Line::from("Нажмите \"q\" чтобы выйти.".fg(Color::Gray)));
        lines
    }

    fn build_completed_content(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];
        lines.push(self.render_question_line(Color::Blue));
        lines.push(self.render_answer_line());
        let example_lines = self.build_example_lines();
        if !example_lines.is_empty() {
            lines.push(Line::from(""));
            lines.extend(example_lines);
        }
        let kanji_details = self.build_kanji_details_lines();
        if !kanji_details.is_empty() {
            lines.push(Line::from(""));
            lines.extend(kanji_details);
        }
        lines
    }

    fn render_question_line(&self, color: Color) -> Line<'_> {
        let question = self.card_question_text();
        self.render_text_with_furigana(question, color)
    }

    fn render_answer_line(&self) -> Line<'_> {
        let answer = self.card_answer_text();
        self.render_text_with_furigana(answer, Color::Magenta)
    }

    fn render_text_with_furigana(&self, text: String, color: Color) -> Line<'static> {
        if self.furigana_shown {
            let furigana_text = text.as_str().as_furigana();
            if furigana_text != text {
                return furigana_renderer::render_furigana(&furigana_text).fg(color);
            }
        }
        Line::from(Span::raw(text).fg(color).bold())
    }

    fn draw_kanji_grid(&self, frame: &mut Frame, area: Rect) {
        let kanji_list = self.card_kanji_info();
        if kanji_list.is_empty() {
            return;
        }

        let kanji_count = kanji_list.len();
        let rows = kanji_count.div_ceil(3); // 3 kanji per row
        let row_constraints: Vec<Constraint> = (0..rows).map(|_| Constraint::Length(16)).collect();
        let rows_layout = Layout::vertical(row_constraints).split(area);

        for (row_idx, row_area) in rows_layout.iter().enumerate() {
            self.render_kanji_row(*row_area, kanji_list, row_idx, frame);
        }
    }

    fn render_kanji_row(
        &self,
        row_area: Rect,
        kanji_list: &[keikaku::domain::dictionary::KanjiInfo],
        row_idx: usize,
        frame: &mut Frame,
    ) {
        const KANJI_PER_ROW: usize = 3;
        let start_idx = row_idx * KANJI_PER_ROW;
        let end_idx = (start_idx + KANJI_PER_ROW).min(kanji_list.len());
        let kanji_in_row = end_idx - start_idx;

        if kanji_in_row == 0 {
            return;
        }

        let col_constraints: Vec<Constraint> = (0..kanji_in_row)
            .map(|_| Constraint::Percentage((100 / kanji_in_row) as u16))
            .collect();
        let cols_layout = Layout::horizontal(col_constraints).split(row_area);

        for (col_idx, kanji_area) in cols_layout.iter().enumerate() {
            let kanji_idx = start_idx + col_idx;
            if kanji_idx < kanji_list.len() {
                self.render_kanji_card(&kanji_list[kanji_idx], *kanji_area, frame);
            }
        }
    }

    fn render_kanji_card(
        &self,
        kanji: &keikaku::domain::dictionary::KanjiInfo,
        area: Rect,
        frame: &mut Frame,
    ) {
        let kanji_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Cyan))
            .title(format!("{} (N{})", kanji.kanji(), kanji.jlpt().as_number()));

        let mut lines = self.build_kanji_card_lines(kanji);
        truncate_lines_if_needed(&mut lines, area.height);

        Paragraph::new(Text::from(lines))
            .block(kanji_block)
            .alignment(Alignment::Left)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .render(area, frame.buffer_mut());
    }

    fn build_kanji_card_lines(
        &self,
        kanji: &keikaku::domain::dictionary::KanjiInfo,
    ) -> Vec<Line<'static>> {
        let mut lines = vec![];

        // Show description only when rating (Answer or Completed state)
        if self.should_show_answer() {
            lines.push(Line::from("Описание:".fg(Color::Yellow).bold()));
            lines.push(Line::from(kanji.description().to_string()));
            lines.push(Line::from(""));
        }

        // Radicals
        if !kanji.radicals().is_empty() {
            lines.push(Line::from("Радикалы:".fg(Color::Yellow).bold()));
            for radical in kanji.radicals().iter() {
                lines.push(Line::from(
                    format!("{} - {}", radical.radical(), radical.name()).fg(Color::Cyan),
                ));
                lines.push(Line::from(
                    format!("  {}", radical.description()).fg(Color::White),
                ));
            }
        } else {
            lines.push(Line::from("Радикалы не найдены.".fg(Color::Gray)));
        }

        lines
    }

    fn build_similarity_hint(&self) -> Line<'_> {
        if self.related_cards_visible() {
            Line::from("Нажмите \"h\" чтобы скрыть связанные карточки и омонимы.".fg(Color::Gray))
        } else {
            Line::from("Нажмите \"h\" чтобы показать связанные карточки и омонимы.".fg(Color::Gray))
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
        if !self.related_cards_visible() || self.card_similarity().is_empty() {
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

        if self.card_similarity().is_empty() {
            lines.push(Line::from("Связанные карточки не найдены.".fg(Color::Gray)));
        } else {
            for similar_card in self.card_similarity() {
                lines.push(self.render_similar_card_question(similar_card));
                if self.should_show_answer() {
                    lines.push(Line::from(
                        format!("  {}", similar_card.meaning()).fg(Color::Magenta),
                    ));
                }
                lines.push(Line::from(""));
            }
        }
        lines
    }

    fn render_similar_card_question(&self, similar_card: &VocabularyStudySessionItem) -> Line<'_> {
        let question = similar_card.word();
        if self.furigana_shown {
            let question_text = question.as_furigana();
            if question_text != question {
                let line = furigana_renderer::render_furigana(&question_text);
                let mut spans = vec![Span::from("• ")];
                spans.extend(line.spans);
                return Line::from(spans).fg(Color::Cyan);
            }
        }
        Line::from(format!("• {}", question).fg(Color::Cyan))
    }

    fn draw_homonyms(&self, frame: &mut Frame, area: Rect) {
        if !self.related_cards_visible() || self.card_homonyms().is_empty() {
            return;
        }

        let homonyms_block = Block::bordered()
            .border_set(border::ROUNDED)
            .border_style(Style::default().fg(Color::Blue))
            .title("Омонимы");

        let homonyms_lines = self.build_homonyms_lines();
        Paragraph::new(Text::from(homonyms_lines))
            .block(homonyms_block)
            .alignment(Alignment::Left)
            .render(area, frame.buffer_mut());
    }

    fn build_homonyms_lines(&self) -> Vec<Line<'_>> {
        let mut lines = vec![Line::from("")];

        if self.card_homonyms().is_empty() {
            lines.push(Line::from("Омонимы не найдены.".fg(Color::Gray)));
        } else {
            for homonym_card in self.card_homonyms() {
                lines.push(self.render_homonym_card_question(homonym_card));
                if self.should_show_answer() {
                    lines.push(Line::from(
                        format!("  {}", homonym_card.meaning()).fg(Color::Magenta),
                    ));
                }
                lines.push(Line::from(""));
            }
        }
        lines
    }

    fn render_homonym_card_question(&self, homonym_card: &VocabularyStudySessionItem) -> Line<'_> {
        let question = homonym_card.word();
        if self.furigana_shown {
            let question_text = question.as_furigana();
            if question_text != question {
                let line = furigana_renderer::render_furigana(&question_text);
                let mut spans = vec![Span::from("• ")];
                spans.extend(line.spans);
                return Line::from(spans).fg(Color::Cyan);
            }
        }
        Line::from(format!("• {}", question).fg(Color::Cyan))
    }

    fn should_show_answer(&self) -> bool {
        matches!(self.state, CardState::Answer | CardState::Completed)
    }

    fn build_example_lines(&self) -> Vec<Line<'_>> {
        match self.card_examples() {
            CardExamples::Vocabulary(examples) => self.build_vocabulary_examples(examples),
            CardExamples::Kanji(examples) => self.build_kanji_examples(examples),
        }
    }

    fn build_vocabulary_examples(&self, examples: &[ExamplePhrase]) -> Vec<Line<'_>> {
        let mut lines = vec![];
        if examples.is_empty() {
            return lines;
        }

        lines.push(Line::from(
            "Примеры использования:".fg(Color::Yellow).bold(),
        ));
        lines.push(Line::from(""));

        for example in examples.iter() {
            lines.push(self.render_example_text_line(example.text()));
            lines.push(Line::from(
                format!("  {}", example.translation()).fg(Color::White),
            ));
            lines.push(Line::from(""));
        }

        lines
    }

    fn build_kanji_examples(&self, examples: &[ExampleKanjiWord]) -> Vec<Line<'_>> {
        let mut lines = vec![];
        if examples.is_empty() {
            return lines;
        }

        lines.push(Line::from("Популярные слова:".fg(Color::Yellow).bold()));
        lines.push(Line::from(""));

        for example in examples.iter() {
            lines.push(self.render_example_text_line(example.word()));
            lines.push(Line::from(
                format!("  {}", example.meaning()).fg(Color::White),
            ));
            lines.push(Line::from(""));
        }

        lines
    }

    fn build_kanji_details_lines(&self) -> Vec<Line<'_>> {
        let mut lines = vec![];
        let Some(card) = self.kanji_card() else {
            return lines;
        };

        lines.push(Line::from(
            format!("JLPT: N{}", card.level().as_number())
                .fg(Color::Yellow)
                .bold(),
        ));
        lines.push(Line::from(""));
        lines.push(Line::from("Радикалы:".fg(Color::Yellow).bold()));

        if card.radicals().is_empty() {
            lines.push(Line::from("  Радикалы не найдены.".fg(Color::Gray)));
            return lines;
        }

        for radical in card.radicals().iter() {
            lines.push(Line::from(
                format!("  {} - {}", radical.radical(), radical.name()).fg(Color::Cyan),
            ));
            lines.push(Line::from(
                format!("    {}", radical.description()).fg(Color::White),
            ));
        }

        lines
    }

    fn render_example_text_line(&self, example_text: &str) -> Line<'_> {
        if self.furigana_shown {
            let furigana_text = example_text.as_furigana();
            if furigana_text != example_text {
                let line = furigana_renderer::render_furigana(&furigana_text);
                // Add indentation
                let mut spans = vec![Span::from("  ")];
                spans.extend(line.spans);
                return Line::from(spans);
            }
        }
        Line::from(format!("  {}", example_text).fg(Color::Cyan))
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
        if matches!(self.state, CardState::Question | CardState::Answer)
            && self.supports_relations()
        {
            self.similarity_shown = !self.similarity_shown;
        }
    }

    fn handle_f_key(&mut self) {
        if matches!(self.state, CardState::Question | CardState::Answer)
            && self.has_furigana_content()
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
    loop_mod: bool,
    limit: Option<usize>,
) -> Result<(), JeersError> {
    let settings = ApplicationEnvironment::get();

    if loop_mod {
        handle_loop_mode(
            user_id,
            new_cards_force,
            furigana_force,
            similarity_force,
            limit,
            settings,
        )
        .await
    } else {
        handle_normal_mode(
            user_id,
            new_cards_force,
            furigana_force,
            similarity_force,
            limit,
            settings,
        )
        .await
    }
}

async fn handle_loop_mode(
    user_id: Ulid,
    new_cards_force: bool,
    furigana_force: bool,
    similarity_force: bool,
    limit: Option<usize>,
    settings: &'static ApplicationEnvironment,
) -> Result<(), JeersError> {
    loop {
        let start_study_usecase = SelectCardsToLearnUseCase::new(settings.get_repository().await?);

        let cards = start_study_usecase
            .execute(user_id, new_cards_force, true, limit)
            .await?;

        if cards.is_empty() {
            render_empty_cards_message()?;
            break;
        }

        let exit_session =
            process_cards(user_id, &cards, furigana_force, similarity_force, settings).await?;

        if exit_session {
            break;
        }

        complete_lesson(user_id, settings).await;
    }

    Ok(())
}

async fn handle_normal_mode(
    user_id: Ulid,
    new_cards_force: bool,
    furigana_force: bool,
    similarity_force: bool,
    limit: Option<usize>,
    settings: &'static ApplicationEnvironment,
) -> Result<(), JeersError> {
    let start_study_usecase = SelectCardsToLearnUseCase::new(settings.get_repository().await?);

    let cards = start_study_usecase
        .execute(user_id, new_cards_force, false, limit)
        .await?;

    if cards.is_empty() {
        render_empty_cards_message()?;
        return Ok(());
    }

    process_cards(user_id, &cards, furigana_force, similarity_force, settings).await?;
    complete_lesson(user_id, settings).await;

    Ok(())
}

async fn process_cards(
    user_id: Ulid,
    cards: &[StudySessionItem],
    furigana_force: bool,
    similarity_force: bool,
    settings: &'static ApplicationEnvironment,
) -> Result<bool, JeersError> {
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

        if let Some(rating) = rating
            && let Err(e) = rate_usecase.execute(user_id, card.card_id(), rating).await
        {
            eprintln!("Error rating card: {:?}", e);
        }

        if exit_session {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn complete_lesson(user_id: Ulid, settings: &'static ApplicationEnvironment) {
    let complete_lesson_usecase =
        CompleteLessonUseCase::new(settings.get_repository().await.unwrap());
    if let Err(e) = complete_lesson_usecase.execute(user_id).await {
        eprintln!("Error completing lesson: {:?}", e);
    }
}

const BORDER_HEIGHT: u16 = 2;

fn truncate_lines_if_needed(lines: &mut Vec<Line<'_>>, area_height: u16) {
    let max_lines = (area_height.saturating_sub(BORDER_HEIGHT)) as usize;
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }
}
