mod analyze_text_for_cards;
mod complete_acquaintance_hand;
mod complete_onboarding_scoring;
mod create_cards_from_analysis;
mod create_grammar_card;
mod create_kanji_card;
mod create_phrase_card;
mod create_vocabulary_card;
mod delete_card;
mod extract_text_from_image;
mod import_anki_pack;
mod import_onboarding_sets;
mod mark_card_as_known;
mod migrate_kanji_companions;
mod migrate_vocabulary_lemmas;
mod migrate_vocabulary_part_of_speech;
mod rate_card;
mod rate_card_with_side_effects;
mod seed_ready_phrases;
mod select_acquaintance_hand;
mod select_cards_to_lesson;
mod toggle_favorite;
mod transcribe_audio;
mod update_user_profile;

#[cfg(test)]
mod tests;
#[cfg(test)]
pub use tests::fixtures::{init_phrase_index_from_cdn, init_real_dictionaries};

pub use analyze_text_for_cards::{AnalyzeTextForCardsUseCase, AnalyzeTextResult, AnalyzedWord};
pub use complete_acquaintance_hand::CompleteAcquaintanceHandUseCase;
pub use complete_onboarding_scoring::CompleteOnboardingScoringUseCase;
pub use create_cards_from_analysis::{
    CreateCardsFromAnalysisResult, CreateCardsFromAnalysisUseCase, WordToCreate,
};
pub use create_grammar_card::CreateGrammarCardUseCase;
pub use create_kanji_card::CreateKanjiCardUseCase;
pub use create_phrase_card::CreatePhraseCardUseCase;
pub use create_vocabulary_card::CreateVocabularyCardUseCase;
pub use delete_card::DeleteCardUseCase;
pub use extract_text_from_image::ExtractTextFromImageUseCase;
pub use import_anki_pack::{
    AnkiCard, AnkiDeckInfo, AnkiFieldInfo, ImportAnkiPackResult, ImportAnkiPackUseCase,
    extract_anki_db_bytes, extract_cards, parse_cards, read_anki_database,
};
pub use import_onboarding_sets::{ImportOnboardingResult, ImportOnboardingSetsUseCase};
pub use mark_card_as_known::MarkCardAsKnownUseCase;
pub use migrate_kanji_companions::{MigrateKanjiCompanionsUseCase, MigrationResult};
pub use migrate_vocabulary_lemmas::{LemmaMigrationResult, MigrateVocabularyLemmasUseCase};
pub use migrate_vocabulary_part_of_speech::{
    MigrateVocabularyPartOfSpeechUseCase, PartOfSpeechMigrationResult,
};
pub use rate_card::RateCardUseCase;
pub use rate_card_with_side_effects::RateCardWithSideEffectsUseCase;
pub use seed_ready_phrases::SeedReadyPhrasesUseCase;
pub use seed_ready_phrases::collect_known_grammar_rules;
pub use seed_ready_phrases::{classify_orphaned_phrases, delete_phrase_cards_by_phrase_ids};
pub use select_acquaintance_hand::SelectAcquaintanceHandUseCase;
pub use select_cards_to_lesson::SelectCardsToLessonUseCase;
pub use toggle_favorite::ToggleFavoriteUseCase;
pub use transcribe_audio::TranscribeAudioUseCase;
pub use update_user_profile::UpdateUserProfileUseCase;
