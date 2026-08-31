use crate::i18n::*;
use crate::pages::lesson::card_type::CardType;
use origa::domain::{Card as DomainCard, CardAnswer, NativeLanguage, StudyCard};
use ulid::Ulid;

#[derive(Clone)]
pub(super) struct ScoringCard {
    pub card_id: Ulid,
    pub question: String,
    pub answer: String,
    /// On/kun readings rendered alongside the answer for kanji cards so the
    /// user can decide whether they recognise the kanji without the question
    /// area giving the reading away as furigana. `None` for non-kanji cards.
    pub readings: Option<String>,
    pub card_type: CardType,
}

fn format_kanji_readings(card: &origa::domain::KanjiCard, locale: Locale) -> Option<String> {
    let on = card.on_readings();
    let kun = card.kun_readings();
    if on.is_empty() && kun.is_empty() {
        return None;
    }
    let on_label = td_string!(locale, onboarding.scoring.reading_on);
    let kun_label = td_string!(locale, onboarding.scoring.reading_kun);
    let mut parts: Vec<String> = Vec::new();
    if !on.is_empty() {
        parts.push(format!("{}: {}", on_label, on.join(", ")));
    }
    if !kun.is_empty() {
        parts.push(format!("{}: {}", kun_label, kun.join(", ")));
    }
    Some(parts.join(" | "))
}

pub(super) fn extract_card_data(
    study_card: &StudyCard,
    lang: &NativeLanguage,
    locale: Locale,
) -> ScoringCard {
    let card_id = *study_card.card_id();
    let no_translation = || td_string!(locale, common.no_translation).to_string();
    match study_card.card() {
        DomainCard::Vocabulary(v) => ScoringCard {
            card_id,
            question: v.word().text().to_string(),
            answer: match v.answer(lang).ok() {
                Some(CardAnswer::Vocabulary { translations, .. }) => translations.join(", "),
                Some(CardAnswer::Text(s)) => s,
                None => no_translation(),
            },
            readings: None,
            card_type: CardType::Vocabulary,
        },
        DomainCard::Kanji(k) => ScoringCard {
            card_id,
            question: k.kanji().text().to_string(),
            answer: match k.description(lang).ok() {
                Some(CardAnswer::Vocabulary { translations, .. }) => translations.join(", "),
                Some(CardAnswer::Text(s)) => s,
                None => no_translation(),
            },
            readings: format_kanji_readings(k, locale),
            card_type: CardType::Kanji,
        },
        DomainCard::Grammar(g) => ScoringCard {
            card_id,
            question: g
                .title(lang)
                .ok()
                .map(|q| q.text().to_string())
                .unwrap_or_default(),
            answer: match g.description(lang).ok() {
                Some(CardAnswer::Vocabulary { translations, .. }) => translations.join(", "),
                Some(CardAnswer::Text(s)) => s,
                None => no_translation(),
            },
            readings: None,
            card_type: CardType::Grammar,
        },
        DomainCard::Phrase(p) => ScoringCard {
            card_id,
            question: p.question().unwrap_or_default(),
            answer: p.answer(lang).unwrap_or_else(no_translation),
            readings: None,
            card_type: CardType::Phrase,
        },
    }
}

/// Builds the scoring queue ordered by [`CardType::sort_order`] (Grammar →
/// Kanji → Vocabulary → Phrase) and then alphabetically by question within
/// each section so the ordering is stable across reloads.
pub(super) fn build_scoring_cards(
    study_cards: &std::collections::HashMap<Ulid, StudyCard>,
    lang: &NativeLanguage,
    locale: Locale,
) -> Vec<ScoringCard> {
    let mut cards: Vec<ScoringCard> = study_cards
        .values()
        .filter(|sc| sc.memory().is_new())
        .map(|sc| extract_card_data(sc, lang, locale))
        .collect();
    cards.sort_by(|a, b| {
        a.card_type
            .sort_order()
            .cmp(&b.card_type.sort_order())
            .then_with(|| a.question.cmp(&b.question))
    });
    cards
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::JapaneseLevel;

    #[test]
    fn sort_order_grammar_before_kanji_before_vocabulary_before_phrase() {
        // Act
        let order_grammar = CardType::Grammar.sort_order();
        let order_kanji = CardType::Kanji.sort_order();
        let order_vocab = CardType::Vocabulary.sort_order();
        let order_phrase = CardType::Phrase.sort_order();

        // Assert — the onboarding scoring step relies on this exact ordering
        // (Grammar → Kanji → Vocabulary → Phrase) to lay out progress-bar
        // sections. A regression here would silently scramble the queue.
        assert!(order_grammar < order_kanji);
        assert!(order_kanji < order_vocab);
        assert!(order_vocab < order_phrase);
    }

    #[test]
    fn sort_order_values_are_stable() {
        // The actual numeric values are part of the contract (e.g. progress
        // markers depend on the relative order, not the absolute numbers, but
        // keeping the values dense u8 starting at 0 makes the section math
        // easy to follow in ScoringProgressBar).
        assert_eq!(CardType::Grammar.sort_order(), 0);
        assert_eq!(CardType::Kanji.sort_order(), 1);
        assert_eq!(CardType::Vocabulary.sort_order(), 2);
        assert_eq!(CardType::Phrase.sort_order(), 3);
    }

    #[test]
    fn japanese_level_ordering_n5_lowest() {
        // Sanity-check the JapaneseLevel ordering assumed by import code paths
        // that derive target_level — keeps the test module self-documenting.
        assert!(JapaneseLevel::N5 < JapaneseLevel::N4);
        assert!(JapaneseLevel::N4 < JapaneseLevel::N3);
        assert!(JapaneseLevel::N3 < JapaneseLevel::N2);
        assert!(JapaneseLevel::N2 < JapaneseLevel::N1);
    }
}
