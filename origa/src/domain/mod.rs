mod error;
mod furigana;
mod furigana_annotator;
pub(crate) mod grammar;
mod japanese;
mod jlpt_content;
mod jlpt_progress;
mod knowledge;
mod memory;
mod score_content;
mod serde_utils;
mod srs;
mod stats;
mod tokenizer;
mod user;
pub(crate) mod value_objects;
mod well_known_set;

pub use error::{ErrorCategory, OrigaError};
pub use furigana::{FuriganaSegment, furiganize_segments, furiganize_text, furiganize_text_html};
pub use furigana_annotator::{AnnotatedSpan, annotate_text};
pub use grammar::apply_format_actions;
pub use grammar::quiz_generation::{
    GrammarPracticeQuestion, apply_mutated_pattern, find_known_vocab_words_for_pos,
    generate_grammar_distractors, generate_grammar_practice_questions,
};
pub use grammar::{detect_format_map_rules, detect_grammar_rules_in_text, detect_keyword_rules};
pub use japanese::{JapaneseChar, JapaneseText};
pub use japanese::{hiragana_to_katakana, katakana_to_hiragana};
pub use jlpt_content::{JlptContent, JlptContentError};
pub use jlpt_progress::{
    CategoryCounts, CategoryProgress, JlptProgress, LevelProgressDetail, ProgressUpdate,
};
pub use knowledge::{
    Card, CardType, DailyHistoryItem, ExampleKanjiWord, GrammarInfo, GrammarQuizCard,
    GrammarRuleCard, KanjiCard, KnowledgeSet, LessonCard, LessonCardView, LessonData,
    LessonEmptyDiagnosis, LessonViewGenerator, MultiQuizResult, PhraseCard, QuizCard, QuizMode,
    QuizOption, StudyCard, VocabularyCard, YesNoCard, diagnose_empty_lesson,
    estimate_completion_date,
};

/// Re-exported so the UI can stay layering-clean: presentation code reaches
/// the rare-reading threshold through the domain, not by reaching into the
/// dictionary module directly. The constant itself lives next to
/// [`KanjiInfo`](crate::dictionary::kanji::KanjiInfo) because it describes
/// dictionary data semantics.
pub use crate::dictionary::kanji::RARE_READING_MAX_FREQ;

pub(crate) use knowledge::collect_known_vocabulary_words;
pub use memory::{CardState, Difficulty, MemoryHistory, MemoryState, Rating, Stability};
pub use score_content::ScoreContentResult;
pub use srs::RateMode;
pub use stats::{RatingRatio, TodayOverview, compute_rating_ratio, compute_today_overview};
pub use tokenizer::{
    DictionaryData, PartOfSpeech, TokenInfo, TokenTranslation, init_dictionary,
    is_dictionary_loaded, lookup_tokens_translations, tokenize_text,
};
pub use user::{ONBOARDING_COMPLETED_KEY, ONBOARDING_SKIPPED_KEY, User, WordKnowledge};
pub use value_objects::{
    CardAnswer, DailyBudget, DailyLoad, JapaneseLevel, NativeLanguage, Question,
};
pub use well_known_set::{
    SetType, TypeMeta, TypesMeta, WellKnownSet, WellKnownSetMeta, get_types_meta, id_to_set_type,
    resolve_set_path, set_types_meta,
};
