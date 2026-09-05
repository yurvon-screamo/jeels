use crate::domain::{Card, CardType, JlptContent, MAX_COMPANION_WORDS, OrigaError};
use crate::traits::UserRepository;
use std::collections::HashSet;
use tracing::debug;
use ulid::Ulid;

/// Замена карты руки знакомства при «Уже знаю»: сначала кластерное
/// восполнение — если у кандзи руки меньше
/// `MAX_COMPANION_WORDS` слов с его знаком и в пуле есть такое новое
/// слово, замещается именно оно (кандзи и его компаньоны остаются в одной
/// руке); иначе — верхушка пула новых карт по JLPT-приоритету (N5 первым),
/// исключая карты, уже занятые рукой. Пропорции `CARD_TYPE_WEIGHTS` здесь
/// не применяются — замещается один слот, а не собирается рука. Дневной
/// лимит не проверяется: замена не добавляет сидируемых карт — лимит
/// спишется при закрытии руки за фактически сидированные.
#[derive(Clone)]
pub struct TakeAcquaintanceReplacementUseCase<'a, R: UserRepository> {
    repository: &'a R,
}

impl<'a, R: UserRepository> TakeAcquaintanceReplacementUseCase<'a, R> {
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        jlpt_content: &JlptContent,
        exclude: &[Ulid],
    ) -> Result<Option<(Ulid, CardType)>, OrigaError> {
        let user = self
            .repository
            .get_current_user()
            .await?
            .ok_or(OrigaError::CurrentUserNotExist)?;

        if let Some(companion) = cluster_replacement(user.knowledge_set(), exclude, jlpt_content) {
            debug!(
                user_id = %user.id(),
                replacement = %companion.0,
                "Acquaintance replacement taken (cluster top-up)"
            );
            return Ok(Some(companion));
        }

        let excluded: HashSet<Ulid> = exclude.iter().copied().collect();
        let candidate = user
            .knowledge_set()
            .study_cards()
            .iter()
            .filter(|(card_id, study_card)| {
                study_card.memory().is_new()
                    && !matches!(study_card.card(), Card::Phrase(_))
                    && !excluded.contains(*card_id)
            })
            .map(|(card_id, study_card)| {
                let card_type = CardType::from(study_card.card());
                let priority = crate::domain::jlpt_sort_key(study_card.card(), jlpt_content);
                (priority, *card_id, card_type)
            })
            // Семантика «верхушки» как у SelectAcquaintanceHandUseCase:
            // максимальный JLPT-приоритет, при равном — меньший Ulid
            // (right.1.cmp(&left.1) отдаёт победу меньшему left.1).
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)));

        match candidate {
            Some((_priority, card_id, card_type)) => {
                debug!(user_id = %user.id(), replacement = %card_id, "Acquaintance replacement taken");
                Ok(Some((card_id, card_type)))
            },
            None => {
                debug!(user_id = %user.id(), "Acquaintance replacement pool empty");
                Ok(None)
            },
        }
    }
}

/// Кластерное восполнение: кандзи руки перебираются в порядке показа;
/// первый, у которого меньше `MAX_COMPANION_WORDS` слов с его знаком,
/// добирает из пула нового компаньона с максимальным JLPT-приоритетом
/// (при равном — меньший Ulid).
fn cluster_replacement(
    knowledge_set: &crate::domain::KnowledgeSet,
    exclude: &[Ulid],
    jlpt_content: &JlptContent,
) -> Option<(Ulid, CardType)> {
    let excluded: HashSet<Ulid> = exclude.iter().copied().collect();

    let contains_char =
        |study_card: &crate::domain::StudyCard, kanji_char: char| match study_card.card() {
            Card::Vocabulary(vocab) => vocab.word().text().chars().any(|ch| ch == kanji_char),
            _ => false,
        };

    for card_id in exclude {
        // Карта руки может быть удалена посреди руки — такой слот просто
        // пропускается, а не глушит кластерное восполнение целиком.
        let Some(study_card) = knowledge_set.get_card(*card_id) else {
            continue;
        };
        let Card::Kanji(kanji_card) = study_card.card() else {
            continue;
        };
        let Some(kanji_char) = kanji_card.kanji().text().chars().next() else {
            continue;
        };

        let companions_in_hand = knowledge_set
            .study_cards()
            .iter()
            .filter(|(id, _)| excluded.contains(*id))
            .filter(|(_, sc)| contains_char(sc, kanji_char))
            .count();
        if companions_in_hand >= MAX_COMPANION_WORDS {
            continue;
        }

        // Слово этого кандзи в пуле нет — пробуем следующие кандзи руки,
        // а не сдаёмся на первом
        if let Some((_priority, card_id)) = knowledge_set
            .study_cards()
            .iter()
            .filter(|(id, study_card)| {
                !excluded.contains(*id)
                    && study_card.memory().is_new()
                    && !matches!(study_card.card(), Card::Phrase(_))
                    && contains_char(study_card, kanji_char)
            })
            .map(|(id, study_card)| {
                (
                    crate::domain::jlpt_sort_key(study_card.card(), jlpt_content),
                    *id,
                )
            })
            .max_by(|left, right| left.0.cmp(&right.0).then(right.1.cmp(&left.1)))
        {
            return Some((card_id, CardType::Vocabulary));
        }
    }
    None
}

#[cfg(test)]
mod cluster_replacement_tests {
    use super::*;
    use crate::domain::{KnowledgeSet, NativeLanguage, Question, User};

    fn setup(cards: Vec<Card>) -> (KnowledgeSet, Vec<Ulid>) {
        let mut user = User::new(
            "test@example.com".to_string(),
            NativeLanguage::Russian,
            None,
        );
        let mut ids = Vec::new();
        for card in cards {
            let study_card = user.create_card(card).unwrap();
            ids.push(*study_card.card_id());
        }
        let knowledge_set = user.knowledge_set().clone();
        (knowledge_set, ids)
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
    fn kanji_short_of_companions_pulls_one_from_pool() {
        // Arrange: в руке кандзи 明 с одним словом, в пуле — второе
        let (knowledge_set, ids) = setup(vec![
            kanji_card("明"),
            vocab_card("明日"),
            vocab_card("明るい"),
        ]);
        let hand = &ids[..2];

        // Act
        let replacement =
            cluster_replacement(&knowledge_set, hand, &crate::domain::JlptContent::new());

        // Assert: замещение добирает кластерное слово, а не любое
        assert_eq!(replacement.map(|(id, _)| id), Some(ids[2]));
    }

    #[test]
    fn full_cluster_falls_back_to_none() {
        // Arrange: у кандзи уже MAX_COMPANION_WORDS слов — добора нет,
        // в пуле постороннее слово
        let (knowledge_set, ids) = setup(vec![
            kanji_card("明"),
            vocab_card("明日"),
            vocab_card("明るい"),
            vocab_card("明治"),
            vocab_card("車"),
        ]);
        let hand = &ids[..4];

        // Act
        let replacement =
            cluster_replacement(&knowledge_set, hand, &crate::domain::JlptContent::new());

        // Assert: кластер полон — кластерного замещения нет
        assert_eq!(replacement, None);
    }

    #[test]
    fn hand_without_kanji_has_no_cluster_replacement() {
        // Arrange
        let (knowledge_set, ids) = setup(vec![vocab_card("車"), vocab_card("犬")]);

        // Act
        let replacement =
            cluster_replacement(&knowledge_set, &ids, &crate::domain::JlptContent::new());

        // Assert
        assert_eq!(replacement, None);
    }

    #[test]
    fn replacement_falls_through_to_next_kanji_with_pool_word() {
        // Arrange: у первого кандзи руки компаньонов в пуле нет, у второго
        // есть — замещение должно дойти до второго, а не сдаться на первом
        let (knowledge_set, ids) =
            setup(vec![kanji_card("明"), kanji_card("月"), vocab_card("月曜")]);
        let hand = &ids[..2];

        // Act
        let replacement =
            cluster_replacement(&knowledge_set, hand, &crate::domain::JlptContent::new());

        // Assert: взят компаньон второго кандзи
        assert_eq!(replacement.map(|(id, _)| id), Some(ids[2]));
    }

    #[test]
    fn deleted_hand_card_does_not_disable_cluster_replacement() {
        // Arrange: первый слот руки удалён из словаря посреди руки
        // (документированный сценарий спеки) — восполнение продолжает
        // работать по оставшимся картам
        let (knowledge_set, ids) = setup(vec![kanji_card("明"), vocab_card("明日")]);
        let hand = vec![Ulid::new(), ids[0]];

        // Act
        let replacement =
            cluster_replacement(&knowledge_set, &hand, &crate::domain::JlptContent::new());

        // Assert: кандзи руки найден, компаньон предложен
        assert_eq!(replacement.map(|(id, _)| id), Some(ids[1]));
    }
}
