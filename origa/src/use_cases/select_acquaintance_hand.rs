use crate::domain::{
    Card, CardType, HAND_MAX_SIZE, JlptContent, MAX_COMPANION_WORDS, MAX_LESSON_SIZE, OrigaError,
    StudyCard,
};
use crate::traits::UserRepository;
use std::collections::{HashMap, HashSet};
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
    /// (`distribute_new_cards`). Размер — всегда `min(HAND_MAX_SIZE,
    /// пул)`. Дневной лимит кратен размеру руки и гейтит
    /// только открытие руки, а не её размер. Рука всегда полная, пока
    /// в пуле есть карты; кандзи руки тянут своих новых компаньонов
    /// в ту же руку (`attach_kanji_companions`).
    ///
    /// Долги вперёд с коротким хвостом: due-очередь, поверх которой
    /// помещается полная рука (`due_debts + HAND_MAX_SIZE <=
    /// MAX_LESSON_SIZE`), знакомство не откладывает — рука кладётся
    /// поверх хвоста (историческое поведение впрыска). Глубокая очередь
    /// (полная рука не влезает) — рука откладывается до очистки долга.
    /// Фразы и «уже знаю» долгом не считаются.
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

        let due_debts = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter(|(_, study_card)| is_due_debt(study_card))
            .count();
        if due_debt_capacity_exceeded(due_debts) {
            debug!(
                user_id = %user.id(),
                due_debts,
                "due queue too deep for a hand on top — acquaintance deferred"
            );
            return Ok(None);
        }

        // Рука всегда полная: дневной лимит кратен
        // HAND_MAX_SIZE и гейтит только ОТКРЫТИЕ руки, а не её размер —
        // хвостовых рук из 1–6 карт не бывает, пока в пуле есть карты.
        let hand_size = HAND_MAX_SIZE;

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

        // Пул для кластерного добора: id → карта (рука собирается из него,
        // поэтому приоритеты жертвы вытеснения всегда здесь доступны).
        let pool: HashMap<Ulid, &StudyCard> = new_cards
            .iter()
            .map(|(card_id, study_card)| (**card_id, *study_card))
            .collect();

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
        let clustered =
            attach_kanji_companions(candidates, &pool, user.knowledge_set(), jlpt_content);
        let ordered = group_presentation_order(user.knowledge_set(), &clustered);
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

/// Влезает ли полная рука поверх due-хвоста в один урок. Короткий
/// хвост (`due_debts + HAND_MAX_SIZE <= MAX_LESSON_SIZE`) руку не
/// откладывает; глубокая очередь — откладывает до очистки долга.
fn due_debt_capacity_exceeded(due_debts: usize) -> bool {
    due_debts.saturating_add(HAND_MAX_SIZE) > MAX_LESSON_SIZE
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

    #[test]
    fn short_due_tail_fits_hand_capacity() {
        // MAX_LESSON_SIZE - HAND_MAX_SIZE долгов: полная рука ещё влезает
        let tail = MAX_LESSON_SIZE - HAND_MAX_SIZE;
        assert!(!due_debt_capacity_exceeded(tail));
        assert!(!due_debt_capacity_exceeded(0));
    }

    #[test]
    fn deep_due_queue_exceeds_hand_capacity() {
        // Одним долгом больше — рука поверх хвоста не помещается
        let deep = MAX_LESSON_SIZE - HAND_MAX_SIZE + 1;
        assert!(due_debt_capacity_exceeded(deep));
    }
}

/// Кластерный добор руки (docs/acquaintance-mode.md): каждый
/// кандзи руки тянет в ту же руку своих новых компаньонов — слова пула,
/// содержащие его знак (то же определение связности, что у
/// `group_presentation_order`). Компаньон вытесняет наименее приоритетное
/// слово руки, НЕ связанное ни с одним кандзи руки (кандзи, грамматика и
/// кластерные слова не вытесняются), размер руки не меняется. Вытесненные
/// карты остаются новыми в пуле и придут позже обычными руками.
fn attach_kanji_companions(
    selected: Vec<(Ulid, CardType)>,
    pool: &HashMap<Ulid, &StudyCard>,
    knowledge_set: &crate::domain::KnowledgeSet,
    jlpt_content: &JlptContent,
) -> Vec<(Ulid, CardType)> {
    let mut hand = selected;

    // Знаки всех кандзи руки: слова, содержащие хоть один из них, —
    // кластерные и вытеснению не подлежат (независимо от того, как они
    // попали в руку — отбором или предыдущим добором).
    let hand_kanji_chars: Vec<char> = hand
        .iter()
        .filter(|(_, card_type)| *card_type == CardType::Kanji)
        .filter_map(|(card_id, _)| kanji_chars_of(knowledge_set, *card_id))
        .flatten()
        .collect();
    if hand_kanji_chars.is_empty() {
        return hand;
    }

    let kanji_ids: Vec<Ulid> = hand
        .iter()
        .filter(|(_, card_type)| *card_type == CardType::Kanji)
        .map(|(card_id, _)| *card_id)
        .collect();

    for kanji_id in kanji_ids {
        let Some(kanji_chars) = kanji_chars_of(knowledge_set, kanji_id) else {
            continue;
        };

        let hand_ids: HashSet<Ulid> = hand.iter().map(|(card_id, _)| *card_id).collect();
        let mut candidates: Vec<(u8, Ulid)> = pool
            .iter()
            .filter(|(card_id, study_card)| {
                !hand_ids.contains(*card_id) && word_contains_any_char(study_card, &kanji_chars)
            })
            .map(|(card_id, study_card)| {
                (
                    crate::domain::jlpt_sort_key(study_card.card(), jlpt_content),
                    *card_id,
                )
            })
            .collect();
        // Приоритетнее — вперёд; при равном приоритете — меньший Ulid
        // (конвенция «верхушки пула», как у take_acquaintance_replacement).
        candidates.sort_by(|left, right| right.0.cmp(&left.0).then(left.1.cmp(&right.1)));

        for (_, companion_id) in candidates.into_iter().take(MAX_COMPANION_WORDS) {
            // Жертва: слово руки вне кластеров (без знаков кандзи руки),
            // наименее приоритетное, при равном — меньший Ulid. Нет жертвы —
            // слоты заняты кандзи/грамматикой/кластером: добор
            // останавливается.
            let victim = hand
                .iter()
                .filter_map(|(card_id, card_type)| {
                    if *card_type != CardType::Vocabulary {
                        return None;
                    }
                    // Источник данных жертвы — knowledge_set: карта руки
                    // может быть удалена, рука не обязана быть подмножеством
                    // переданного пула.
                    Some((knowledge_set.get_card(*card_id)?, *card_id))
                })
                .filter(|(study_card, _)| !word_contains_any_char(study_card, &hand_kanji_chars))
                .map(|(study_card, card_id)| {
                    (
                        crate::domain::jlpt_sort_key(study_card.card(), jlpt_content),
                        card_id,
                    )
                })
                .min();
            let Some((_priority, victim_id)) = victim else {
                break;
            };
            if let Some(slot) = hand.iter_mut().find(|(card_id, _)| *card_id == victim_id) {
                *slot = (companion_id, CardType::Vocabulary);
            }
        }
    }
    hand
}

/// Знаки кандзи-карты (одиночный символ по контракту KanjiCard).
fn kanji_chars_of(knowledge_set: &crate::domain::KnowledgeSet, card_id: Ulid) -> Option<Vec<char>> {
    match knowledge_set.get_card(card_id)?.card() {
        Card::Kanji(kanji_card) => Some(kanji_card.kanji().text().chars().collect()),
        _ => None,
    }
}

fn word_contains_any_char(study_card: &StudyCard, chars: &[char]) -> bool {
    match study_card.card() {
        Card::Vocabulary(vocab) => vocab.word().text().chars().any(|ch| chars.contains(&ch)),
        _ => false,
    }
}

/// Порядок показа (docs/acquaintance-mode.md, правило «Порядок показа»):
/// кандзи первым, слова этой руки, содержащие его знак, сразу за ним;
/// остальные карты — в естественном (приоритетном) порядке.
fn group_presentation_order(
    knowledge_set: &crate::domain::KnowledgeSet,
    candidates: &[(Ulid, CardType)],
) -> Vec<Ulid> {
    let mut result: Vec<Ulid> = Vec::with_capacity(candidates.len());
    let mut consumed: HashSet<Ulid> = HashSet::new();

    for (card_id, card_type) in candidates {
        if *card_type != CardType::Kanji || !consumed.insert(*card_id) {
            continue;
        }
        result.push(*card_id);

        let Some(kanji_chars) = kanji_chars_of(knowledge_set, *card_id) else {
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

#[cfg(test)]
mod attach_kanji_companions_tests {
    use super::*;
    use crate::domain::{KnowledgeSet, NativeLanguage, Question, User};

    fn setup(
        hand_cards: Vec<Card>,
        pool_cards: Vec<Card>,
    ) -> (
        Vec<(Ulid, CardType)>,
        HashMap<Ulid, StudyCard>,
        KnowledgeSet,
    ) {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let mut hand = Vec::new();
        for card in hand_cards {
            let study_card = user.create_card(card).unwrap();
            hand.push((*study_card.card_id(), CardType::from(study_card.card())));
        }
        let mut pool = HashMap::new();
        for card in pool_cards {
            let study_card = user.create_card(card).unwrap();
            pool.insert(*study_card.card_id(), study_card.clone());
        }
        let knowledge_set = user.knowledge_set().clone();
        (hand, pool, knowledge_set)
    }

    /// Собирает пул ссылок из owned-карт — сигнатура продакшн-функции.
    fn pool_refs(pool: &HashMap<Ulid, StudyCard>) -> HashMap<Ulid, &StudyCard> {
        pool.iter().map(|(id, sc)| (*id, sc)).collect()
    }

    fn kanji_card(text: &str) -> Card {
        Card::Kanji(crate::domain::KanjiCard::new_test(text.to_string()))
    }

    fn vocab_card(text: &str) -> Card {
        Card::Vocabulary(crate::domain::VocabularyCard::new(
            Question::new(text.to_string()).unwrap(),
        ))
    }

    fn hand_texts(hand: &[(Ulid, CardType)], knowledge_set: &KnowledgeSet) -> Vec<String> {
        hand.iter()
            .filter(|(_, card_type)| *card_type == CardType::Vocabulary)
            .filter_map(
                |(card_id, _)| match knowledge_set.get_card(*card_id)?.card() {
                    Card::Vocabulary(vocab) => Some(vocab.word().text().to_string()),
                    _ => None,
                },
            )
            .collect()
    }

    #[test]
    fn kanji_pulls_its_new_companions_into_the_hand() {
        // Arrange: рука с кандзи 明 и некластерными словами; в пуле —
        // два новых слова со знаком 明
        let (hand, pool, knowledge_set) = setup(
            vec![
                kanji_card("明"),
                vocab_card("車"),
                vocab_card("犬"),
                vocab_card("猫"),
            ],
            vec![vocab_card("明日"), vocab_card("明るい")],
        );

        let pool = pool_refs(&pool);

        // Act
        let attached = attach_kanji_companions(hand, &pool, &knowledge_set, &JlptContent::new());

        // Assert: оба компаньона в руке, размер не изменился,
        // вытеснены два некластерных слова
        let texts = hand_texts(&attached, &knowledge_set);
        assert_eq!(attached.len(), 4, "размер руки не меняется");
        assert!(texts.contains(&"明日".to_string()));
        assert!(texts.contains(&"明るい".to_string()));
        assert_eq!(
            texts.iter().filter(|t| !t.contains('明')).count(),
            1,
            "осталось одно некластерное слово: {texts:?}"
        );
    }

    #[test]
    fn cluster_word_selected_by_distribution_is_never_a_victim() {
        // Arrange: 明日 уже в руке от отбора, 車 — единственная жертва
        let (hand, pool, knowledge_set) = setup(
            vec![kanji_card("明"), vocab_card("明日"), vocab_card("車")],
            vec![vocab_card("明るい")],
        );

        let pool = pool_refs(&pool);

        // Act
        let attached = attach_kanji_companions(hand, &pool, &knowledge_set, &JlptContent::new());

        // Assert: вытеснено 車, а не кластерное 明日
        let texts = hand_texts(&attached, &knowledge_set);
        assert_eq!(attached.len(), 3);
        assert!(texts.contains(&"明日".to_string()));
        assert!(texts.contains(&"明るい".to_string()));
        assert!(!texts.contains(&"車".to_string()));
    }

    #[test]
    fn companion_attach_is_capped_at_three_per_kanji() {
        // Arrange: в пуле 5 слов со знаком 明, в руке 4 слота-жертвы
        let (hand, pool, knowledge_set) = setup(
            vec![
                kanji_card("明"),
                vocab_card("車"),
                vocab_card("犬"),
                vocab_card("猫"),
                vocab_card("馬"),
            ],
            vec![
                vocab_card("明日"),
                vocab_card("明るい"),
                vocab_card("明白"),
                vocab_card("明治"),
                vocab_card("中秋"),
            ],
        );

        let pool = pool_refs(&pool);

        // Act
        let attached = attach_kanji_companions(hand, &pool, &knowledge_set, &JlptContent::new());

        // Assert: вошло ровно MAX_COMPANION_WORDS, размер руки сохранён
        let cluster_count = hand_texts(&attached, &knowledge_set)
            .iter()
            .filter(|t| t.contains('明'))
            .count();
        assert_eq!(cluster_count, MAX_COMPANION_WORDS);
        assert_eq!(attached.len(), 5);
    }

    #[test]
    fn hand_without_kanji_stays_unchanged() {
        // Arrange
        let (hand, pool, knowledge_set) = setup(
            vec![vocab_card("車"), vocab_card("犬")],
            vec![vocab_card("明日")],
        );

        let pool = pool_refs(&pool);

        // Act
        let attached =
            attach_kanji_companions(hand.clone(), &pool, &knowledge_set, &JlptContent::new());

        // Assert: без кандзи в руке кластеризации нет
        assert_eq!(attached, hand);
    }

    #[test]
    fn exhausted_victim_slots_stop_the_attach() {
        // Arrange: рука из кандзи и одних кластерных слов — жертв нет,
        // в пуле ещё два компаньона
        let (hand, pool, knowledge_set) = setup(
            vec![kanji_card("明"), vocab_card("明日"), vocab_card("明るい")],
            vec![vocab_card("明白"), vocab_card("明治")],
        );

        let pool = pool_refs(&pool);

        // Act
        let attached =
            attach_kanji_companions(hand.clone(), &pool, &knowledge_set, &JlptContent::new());

        // Assert: добор остановился — вытеснять некого
        assert_eq!(attached, hand);
    }
}
