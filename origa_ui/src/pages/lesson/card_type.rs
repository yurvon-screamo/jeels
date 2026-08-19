use crate::i18n::*;
use crate::ui_components::TagVariant;
use origa::domain::Card as DomainCard;

#[cfg(test)]
use origa::domain::VocabularyCard;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum CardType {
    #[default]
    Vocabulary,
    Kanji,
    Grammar,
    Phrase,
}

impl CardType {
    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            CardType::Vocabulary => i18n
                .get_keys_untracked()
                .lesson()
                .word()
                .inner()
                .to_string(),
            CardType::Kanji => i18n
                .get_keys_untracked()
                .lesson()
                .kanji()
                .inner()
                .to_string(),
            CardType::Grammar => i18n
                .get_keys_untracked()
                .lesson()
                .grammar()
                .inner()
                .to_string(),
            CardType::Phrase => i18n
                .get_keys_untracked()
                .lesson()
                .phrase()
                .inner()
                .to_string(),
        }
    }

    pub fn tag_variant(&self) -> TagVariant {
        match self {
            CardType::Vocabulary => TagVariant::Default,
            CardType::Kanji => TagVariant::Olive,
            CardType::Grammar => TagVariant::Terracotta,
            CardType::Phrase => TagVariant::Sage,
        }
    }

    /// Stable ordering used by the onboarding scoring step to group cards by
    /// type so the user evaluates grammar first, then kanji, then vocabulary,
    /// then phrases — rather than the hash-randomized order of `HashMap`.
    /// Lower numbers come first.
    pub fn sort_order(&self) -> u8 {
        match self {
            CardType::Grammar => 0,
            CardType::Kanji => 1,
            CardType::Vocabulary => 2,
            CardType::Phrase => 3,
        }
    }
}

impl From<&DomainCard> for CardType {
    fn from(card: &DomainCard) -> Self {
        match card {
            DomainCard::Vocabulary(_) => CardType::Vocabulary,
            DomainCard::Kanji(_) => CardType::Kanji,
            DomainCard::Grammar(_) => CardType::Grammar,
            DomainCard::Phrase(_) => CardType::Phrase,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_components::TagVariant;

    #[test]
    fn tag_variant_mapping() {
        assert_eq!(CardType::Vocabulary.tag_variant(), TagVariant::Default);
        assert_eq!(CardType::Kanji.tag_variant(), TagVariant::Olive);
        assert_eq!(CardType::Grammar.tag_variant(), TagVariant::Terracotta);
        assert_eq!(CardType::Phrase.tag_variant(), TagVariant::Sage);
    }

    #[test]
    fn sort_order_is_grammar_first_then_kanji_vocab_phrase() {
        assert_eq!(CardType::Grammar.sort_order(), 0);
        assert_eq!(CardType::Kanji.sort_order(), 1);
        assert_eq!(CardType::Vocabulary.sort_order(), 2);
        assert_eq!(CardType::Phrase.sort_order(), 3);
    }

    #[test]
    fn from_domain_card_vocabulary() {
        // Only test the From impl when the dictionary is available
        let card = VocabularyCard::from_known_word("猫", &origa::domain::NativeLanguage::Russian);
        if let Ok(vocab) = card {
            let domain = DomainCard::Vocabulary(vocab);
            assert_eq!(CardType::from(&domain), CardType::Vocabulary);
        }
    }
}
