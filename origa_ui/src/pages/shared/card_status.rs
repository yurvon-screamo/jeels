use crate::i18n::Locale;
use crate::ui_components::TagVariant;
use leptos_i18n::I18nContext;
use origa::domain::StudyCard;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CardStatus {
    #[default]
    New,
    Hard,
    InProgress,
    Learned,
}

impl CardStatus {
    pub fn from_study_card(card: &StudyCard) -> Self {
        let memory = card.memory();
        if memory.is_new() {
            CardStatus::New
        } else if memory.is_high_difficulty() {
            CardStatus::Hard
        } else if memory.is_known_card() {
            CardStatus::Learned
        } else {
            CardStatus::InProgress
        }
    }

    pub fn label(&self, i18n: &I18nContext<Locale>) -> String {
        match self {
            CardStatus::New => i18n.get_keys().shared().status_new().inner().to_string(),
            CardStatus::Hard => i18n.get_keys().shared().status_hard().inner().to_string(),
            CardStatus::InProgress => i18n
                .get_keys()
                .shared()
                .status_in_progress()
                .inner()
                .to_string(),
            CardStatus::Learned => i18n
                .get_keys()
                .shared()
                .status_learned()
                .inner()
                .to_string(),
        }
    }

    pub fn tag_variant(&self) -> TagVariant {
        match self {
            CardStatus::New => TagVariant::Default,
            CardStatus::Hard => TagVariant::Terracotta,
            CardStatus::InProgress => TagVariant::Filled,
            CardStatus::Learned => TagVariant::Olive,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{Card, Question, StudyCard, VocabularyCard};

    fn fresh_vocab_card(word: &str) -> StudyCard {
        let vocab =
            VocabularyCard::new_with_pos(Question::new(word.to_string()).unwrap(), None, None);
        StudyCard::new(Card::Vocabulary(vocab))
    }

    #[test]
    fn tag_variant_maps_every_status() {
        assert_eq!(CardStatus::New.tag_variant(), TagVariant::Default);
        assert_eq!(CardStatus::Hard.tag_variant(), TagVariant::Terracotta);
        assert_eq!(CardStatus::InProgress.tag_variant(), TagVariant::Filled);
        assert_eq!(CardStatus::Learned.tag_variant(), TagVariant::Olive);
    }

    #[test]
    fn from_study_card_fresh_card_is_new() {
        let card = fresh_vocab_card("ねこ");
        assert_eq!(CardStatus::from_study_card(&card), CardStatus::New);
    }
}
