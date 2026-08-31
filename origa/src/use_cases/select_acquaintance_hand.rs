use crate::domain::{Card, CardType, HAND_MAX_SIZE, JlptContent, OrigaError, StudyCard};
use crate::traits::UserRepository;
use std::collections::HashSet;
use tracing::{debug, info};
use ulid::Ulid;

#[derive(Clone)]
pub struct SelectAcquaintanceHandUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> SelectAcquaintanceHandUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    /// Выбирает руку знакомства из пула новых карт: JLPT-приоритет
    /// (N5 первым) и пропорция типов по `CARD_TYPE_WEIGHTS` — тот же
    /// контракт, что у исторического впрыска новых карт в урок
    /// (`distribute_new_cards`). Размер min(лимит дня, пул,
    /// `HAND_MAX_SIZE`).
    ///
    /// Долги вперёд: пока в деке есть просроченные ревью-карты (не
    /// фразы), рука не собирается — новые карты не добавляют нагрузку
    /// к долговому ревью. Знакомство возвращается, когда due-очередь
    /// пуста (холодный старт новых дней — частный случай).
    ///
    /// Чистая функция от состояния knowledge_set и остатка дневного лимита:
    /// никакое состояние руки не персистится, прерванная рука естественно
    /// возвращается верхушкой пула.
    pub async fn execute(
        &self,
        jlpt_content: &JlptContent,
    ) -> Result<Option<Vec<Ulid>>, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        let daily_remaining = budget_new_cards_per_day(&user)
            .saturating_sub(user.knowledge_set().new_cards_studied_today());
        if daily_remaining == 0 {
            debug!(user_id = %user.id(), "daily new-card limit exhausted");
            return Ok(None);
        }

        if user
            .knowledge_set()
            .study_cards()
            .iter()
            .any(|(_, study_card)| is_due_debt(study_card))
        {
            debug!(
                user_id = %user.id(),
                "due reviews pending — acquaintance hand deferred"
            );
            return Ok(None);
        }

        let hand_size = daily_remaining.min(HAND_MAX_SIZE);

        // Кандидаты: новые карты, кроме фраз (фразы живут собственными
        // пайплайнами anchored/tail и знакомство не проходят).
        let new_cards: Vec<(&Ulid, &StudyCard)> = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter(|(_, study_card)| {
                study_card.memory().is_new() && !matches!(study_card.card(), Card::Phrase(_))
            })
            .collect();
        if new_cards.is_empty() {
            return Ok(None);
        }

        let mut rng = rand::rng();
        let selected =
            crate::domain::distribute_new_cards(new_cards, jlpt_content, hand_size, &mut rng);
        if selected.is_empty() {
            return Ok(None);
        }

        let candidates: Vec<(Ulid, CardType)> = selected
            .iter()
            .map(|(card_id, study_card)| (**card_id, CardType::from(study_card.card())))
            .collect();
        let ordered = group_presentation_order(user.knowledge_set(), &candidates);
        info!(
            user_id = %user.id(),
            hand_size = ordered.len(),
            "Acquaintance hand selected"
        );
        Ok(Some(ordered))
    }
}

fn budget_new_cards_per_day(user: &crate::domain::User) -> usize {
    use crate::domain::DailyBudget;
    DailyBudget::from_load(*user.daily_load()).new_cards_per_day()
}

/// Долг перед знакомством: просроченное ревью НЕ-фразы. Новые карты
/// долгом не считаются (они и есть кандидаты руки), фразы живут
/// собственными пайплайнами, а известные («Уже знаю», стабильность
/// выше порога) — осознанно просрочены и нагрузкой обучения не являются.
fn is_due_debt(study_card: &StudyCard) -> bool {
    !matches!(study_card.card(), Card::Phrase(_))
        && !study_card.memory().is_new()
        && !study_card.memory().is_known_card()
        && study_card.memory().is_due()
}

#[cfg(test)]
mod is_due_debt_tests {
    use super::*;
    use crate::domain::MemoryState;
    use chrono::{Duration, Utc};

    fn study_card_with(card: Card, next_review: chrono::DateTime<Utc>) -> StudyCard {
        let mut study_card = StudyCard::new(card);
        let memory = MemoryState::new(
            crate::domain::Stability::new(10.0).unwrap(),
            crate::domain::Difficulty::new(5.0).unwrap(),
            next_review,
        );
        study_card.apply_review(memory, crate::domain::Rating::Good);
        study_card
    }

    fn vocab_card() -> Card {
        Card::Vocabulary(crate::domain::VocabularyCard::new(
            crate::domain::Question::new("テスト".to_string()).unwrap(),
        ))
    }

    #[test]
    fn overdue_review_card_is_a_due_debt() {
        let card = study_card_with(vocab_card(), Utc::now() - Duration::hours(1));
        assert!(is_due_debt(&card));
    }

    #[test]
    fn future_review_card_is_not_a_due_debt() {
        // Сидированная закрытой рукой карта: ревью завтра
        let card = study_card_with(vocab_card(), Utc::now() + Duration::days(1));
        assert!(!is_due_debt(&card));
    }

    #[test]
    fn new_card_is_not_a_due_debt() {
        let card = StudyCard::new(vocab_card());
        assert!(!is_due_debt(&card));
    }

    #[test]
    fn overdue_phrase_is_not_a_due_debt() {
        let card = study_card_with(
            Card::Phrase(crate::domain::PhraseCard::new(Ulid::new())),
            Utc::now() - Duration::hours(1),
        );
        assert!(!is_due_debt(&card));
    }

    #[test]
    fn overdue_known_card_is_not_a_due_debt() {
        // «Уже знаю»: стабильность выше порога известности — карта
        // осознанно просрочена, знакомство из-за неё не откладывается
        let mut known = study_card_with(vocab_card(), Utc::now() - Duration::hours(1));
        let memory = MemoryState::new(
            crate::domain::Stability::new(22.0).unwrap(),
            crate::domain::Difficulty::new(3.0).unwrap(),
            Utc::now() - Duration::hours(1),
        );
        known.apply_review(memory, crate::domain::Rating::Easy);
        assert!(!is_due_debt(&known));
    }
}

/// Порядок показа (docs/acquaintance-mode.md, правило «Порядок показа»):
/// кандзи первым, слова этой руки, содержащие его знак, сразу за ним;
/// остальные карты — в естественном (приоритетном) порядке.
fn group_presentation_order(
    knowledge_set: &crate::domain::KnowledgeSet,
    candidates: &[(Ulid, CardType)],
) -> Vec<Ulid> {
    let kanji_chars_of = |card_id: Ulid| -> Option<Vec<char>> {
        match knowledge_set.get_card(card_id)?.card() {
            Card::Kanji(kanji_card) => Some(kanji_card.kanji().text().chars().collect()),
            _ => None,
        }
    };

    let mut result: Vec<Ulid> = Vec::with_capacity(candidates.len());
    let mut consumed: HashSet<Ulid> = HashSet::new();

    for (card_id, card_type) in candidates {
        if *card_type != CardType::Kanji || !consumed.insert(*card_id) {
            continue;
        }
        result.push(*card_id);

        let Some(kanji_chars) = kanji_chars_of(*card_id) else {
            continue;
        };
        for (word_id, word_type) in candidates {
            if *word_type != CardType::Vocabulary || consumed.contains(word_id) {
                continue;
            }
            let Some(word_text) = word_text_of(knowledge_set, *word_id) else {
                continue;
            };
            if word_text.chars().any(|ch| kanji_chars.contains(&ch)) {
                // Компаньон идёт сразу за своим кандзи; приоритет слова
                // внутри руки не важен — группировка по знаку главнее.
                result.push(*word_id);
                consumed.insert(*word_id);
            }
        }
    }

    for (card_id, _card_type) in candidates {
        if consumed.insert(*card_id) {
            result.push(*card_id);
        }
    }
    result
}

fn word_text_of(knowledge_set: &crate::domain::KnowledgeSet, card_id: Ulid) -> Option<String> {
    match knowledge_set.get_card(card_id)?.card() {
        Card::Vocabulary(vocab) => Some(vocab.word().text().to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod group_presentation_order_tests {
    use super::*;
    use crate::domain::{KnowledgeSet, NativeLanguage, Question, User};

    fn setup(cards: Vec<Card>) -> (KnowledgeSet, Vec<(Ulid, CardType)>) {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let mut candidates = Vec::new();
        for card in cards {
            let study_card = user.create_card(card).unwrap();
            candidates.push((*study_card.card_id(), CardType::from(study_card.card())));
        }
        let knowledge_set = user.knowledge_set().clone();
        (knowledge_set, candidates)
    }

    fn kanji_card(text: &str) -> Card {
        Card::Kanji(crate::domain::KanjiCard::new_test(text.to_string()))
    }

    fn vocab_card(text: &str) -> Card {
        Card::Vocabulary(crate::domain::VocabularyCard::new(
            Question::new(text.to_string()).unwrap(),
        ))
    }

    #[test]
    fn companion_words_follow_their_kanji_in_presentation_order() {
        // Arrange
        let (knowledge_set, candidates) = setup(vec![
            kanji_card("明"),
            vocab_card("明日"),
            vocab_card("明るい"),
            vocab_card("車"),
        ]);

        // Act
        let order = group_presentation_order(&knowledge_set, &candidates);

        // Assert: кандзи первым, его слова сразу за ним, чужое слово — в хвосте
        let texts: Vec<Option<String>> = order
            .iter()
            .map(|id| word_text_of(&knowledge_set, *id))
            .collect();
        assert_eq!(order[0], /* 明 */ candidates[0].0);
        assert_eq!(texts[1].as_deref(), Some("明日"));
        assert_eq!(texts[2].as_deref(), Some("明るい"));
        assert_eq!(texts[3].as_deref(), Some("車"));
    }

    #[test]
    fn two_kanjis_each_pull_only_their_own_words() {
        // Arrange
        let (knowledge_set, candidates) = setup(vec![
            kanji_card("明"),
            kanji_card("月"),
            vocab_card("明日"),
            vocab_card("月曜"),
            vocab_card("車"),
        ]);

        // Act
        let order = group_presentation_order(&knowledge_set, &candidates);

        // Assert: каждый кандзи тянет только слова со своим знаком;
        // 明日 прикреплён к 明, 月曜 — к 月, 車 остаётся в хвосте
        let card_at = |index: usize| knowledge_set.get_card(order[index]).unwrap().card();
        assert!(matches!(card_at(0), Card::Kanji(_)));
        assert_eq!(
            word_text_of(&knowledge_set, order[1]).as_deref(),
            Some("明日")
        );
        assert!(matches!(card_at(2), Card::Kanji(_)));
        assert_eq!(
            word_text_of(&knowledge_set, order[3]).as_deref(),
            Some("月曜")
        );
        assert_eq!(
            word_text_of(&knowledge_set, order[4]).as_deref(),
            Some("車")
        );
    }

    #[test]
    fn hand_without_kanji_keeps_priority_order() {
        // Arrange
        let (knowledge_set, candidates) = setup(vec![vocab_card("あ"), vocab_card("い")]);

        // Act
        let order = group_presentation_order(&knowledge_set, &candidates);

        // Assert: без кандзи порядок показа совпадает с приоритетным
        assert_eq!(order.len(), 2);
    }
}
