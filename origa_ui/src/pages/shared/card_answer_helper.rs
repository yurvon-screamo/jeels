use origa::domain::{Card, CardAnswer, NativeLanguage};

/// Extract plain-text representation of a card answer for display
/// (search, metrics, card lists).
///
/// Returns `translations.join(", ")` for Vocabulary,
/// text for Text, or an empty string on error.
pub fn format_answer_text(card: &Card, lang: &NativeLanguage) -> String {
    match card.answer(lang) {
        Ok(answer) => answer.text_projection(),
        Err(_) => String::new(),
    }
}

/// Extract translations + description for WordTranslations component.
///
/// Returns `(translations, description)` for Vocabulary,
/// `(vec![text], None)` for Text, or `(vec![], None)` on error.
pub fn format_answer_parts(card: &Card, lang: &NativeLanguage) -> (Vec<String>, Option<String>) {
    match card.answer(lang) {
        Ok(CardAnswer::Vocabulary {
            translations,
            description,
        }) => (translations, description),
        Ok(other) => (vec![other.text_projection()], None),
        Err(_) => (vec![], None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Question, VocabularyCard};

    #[test]
    fn answer_parts_reversed_vocab_returns_reverse_side() {
        // reverse_side answers bypass the translation dictionary entirely.
        let vocab = VocabularyCard::new_with_pos(
            Question::new("cat".to_string()).unwrap(),
            None,
            Some(Question::new("ねこ".to_string()).unwrap()),
        );
        let card = Card::Vocabulary(vocab);
        let (translations, description) = format_answer_parts(&card, &NativeLanguage::Russian);
        assert_eq!(translations, vec!["ねこ".to_string()]);
        assert_eq!(description, None);
    }

    #[test]
    fn answer_text_reversed_vocab_returns_reverse_side() {
        let vocab = VocabularyCard::new_with_pos(
            Question::new("cat".to_string()).unwrap(),
            None,
            Some(Question::new("ねこ".to_string()).unwrap()),
        );
        let card = Card::Vocabulary(vocab);
        assert_eq!(format_answer_text(&card, &NativeLanguage::English), "ねこ");
    }

    // NOTE: a kanji-card case is not testable here — `KanjiCard::new`
    // itself requires the kradfile dictionary, so there is no dictionary-
    // free kanji fixture. Kanji degradation paths are covered where the
    // dictionary is initialised (origa crate integration tests).
}
