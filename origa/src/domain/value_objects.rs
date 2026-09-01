use crate::dictionary::grammar::{CommonMistake, NuanceNote};
use crate::domain::OrigaError;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};

// Vocabulary answers carry an unbounded number of source-language translations
// in the raw dictionary (EN has up to 62 senses for some entries, vs. RU's
// natural cap of 7). Showing all of them on a card hurts review pacing and
// matches no peer app's UX. The cap is enforced at the CardAnswer boundary so
// every rendering path benefits. Source order is preserved — the dictionary
// already lists senses roughly by frequency, so the first 7 are the most
// useful. See #178 W-9.
const MAX_VOCABULARY_TRANSLATIONS: usize = 7;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Question {
    text: String,
}

impl Question {
    pub fn new(text: String) -> Result<Self, OrigaError> {
        let text = text.trim();
        if text.is_empty() {
            return Err(OrigaError::InvalidQuestion {
                reason: "Question text cannot be empty".to_string(),
            });
        }

        Ok(Self {
            text: text.to_string(),
        })
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardAnswer {
    Vocabulary {
        translations: Vec<String>,
        description: Option<String>,
    },
    Text(String),
    /// Structured grammar nuances (schema v2): wrong/correct pairs plus
    /// free-form notes. Rich surfaces render the structure natively;
    /// inherently-text surfaces use `text_projection`.
    GrammarNuances {
        common_mistakes: Vec<CommonMistake>,
        notes: Vec<NuanceNote>,
    },
}

impl CardAnswer {
    pub fn text(s: String) -> Result<Self, OrigaError> {
        let trimmed = s.trim().to_string();
        if trimmed.is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Answer text cannot be empty".to_string(),
            });
        }
        Ok(CardAnswer::Text(trimmed))
    }

    pub fn vocabulary(
        translations: Vec<String>,
        description: Option<String>,
    ) -> Result<Self, OrigaError> {
        let mut non_empty: Vec<String> = translations
            .into_iter()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect();
        if non_empty.is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Vocabulary answer must have at least one translation".to_string(),
            });
        }
        // Cap to MAX_VOCABULARY_TRANSLATIONS after filtering, preserving the
        // leading (highest-priority) senses. `truncate` is a no-op when the
        // vec is already at or below the cap, so RU answers (which never
        // exceed 7) are unaffected.
        non_empty.truncate(MAX_VOCABULARY_TRANSLATIONS);
        let desc = description.and_then(|d| {
            let trimmed = d.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        });
        Ok(CardAnswer::Vocabulary {
            translations: non_empty,
            description: desc,
        })
    }

    pub fn grammar_nuances(
        common_mistakes: Vec<CommonMistake>,
        notes: Vec<NuanceNote>,
    ) -> Result<Self, OrigaError> {
        if common_mistakes.is_empty() && notes.is_empty() {
            return Err(OrigaError::InvalidAnswer {
                reason: "Grammar nuances answer must have at least one mistake or note".to_string(),
            });
        }
        Ok(CardAnswer::GrammarNuances {
            common_mistakes,
            notes,
        })
    }

    /// Compact single-string projection for surfaces that are text by
    /// nature (quiz options, search, stats). Emoji-free by contract —
    /// rendering markers belong to the UI layer.
    pub fn text_projection(&self) -> String {
        match self {
            CardAnswer::Vocabulary { translations, .. } => translations.join(", "),
            CardAnswer::Text(s) => s.clone(),
            CardAnswer::GrammarNuances {
                common_mistakes,
                notes,
            } => {
                let mut parts: Vec<String> = common_mistakes
                    .iter()
                    .map(|m| format!("{} → {}", m.wrong(), m.correct()))
                    .collect();
                parts.extend(notes.iter().map(|n| n.text().to_string()));
                parts.join("; ")
            },
        }
    }

    pub fn translations(&self) -> &[String] {
        match self {
            CardAnswer::Vocabulary { translations, .. } => translations,
            CardAnswer::Text(s) => std::slice::from_ref(s),
            CardAnswer::GrammarNuances { .. } => &[],
        }
    }

    pub fn description(&self) -> Option<&str> {
        match self {
            CardAnswer::Vocabulary { description, .. } => description.as_deref(),
            CardAnswer::Text(_) | CardAnswer::GrammarNuances { .. } => None,
        }
    }

    pub fn is_vocabulary(&self) -> bool {
        matches!(self, CardAnswer::Vocabulary { .. })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum JapaneseLevel {
    N5,
    N4,
    N3,
    N2,
    N1,
}

impl JapaneseLevel {
    pub const ALL: [JapaneseLevel; 5] = [Self::N5, Self::N4, Self::N3, Self::N2, Self::N1];

    pub fn as_number(&self) -> u8 {
        match self {
            JapaneseLevel::N5 => 5,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N1 => 1,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            JapaneseLevel::N5 => "N5",
            JapaneseLevel::N4 => "N4",
            JapaneseLevel::N3 => "N3",
            JapaneseLevel::N2 => "N2",
            JapaneseLevel::N1 => "N1",
        }
    }

    pub fn from_str_or_default(s: &str) -> Self {
        s.parse().unwrap_or(JapaneseLevel::N1)
    }
}

impl fmt::Display for JapaneseLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_number())
    }
}

impl FromStr for JapaneseLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "N5" => Ok(JapaneseLevel::N5),
            "N4" => Ok(JapaneseLevel::N4),
            "N3" => Ok(JapaneseLevel::N3),
            "N2" => Ok(JapaneseLevel::N2),
            "N1" => Ok(JapaneseLevel::N1),
            other => Err(format!("Unknown Japanese level: {}", other)),
        }
    }
}

impl From<i32> for JapaneseLevel {
    fn from(value: i32) -> Self {
        match value {
            1 => JapaneseLevel::N1,
            2 => JapaneseLevel::N2,
            3 => JapaneseLevel::N3,
            4 => JapaneseLevel::N4,
            _ => JapaneseLevel::N5,
        }
    }
}

impl From<JapaneseLevel> for i32 {
    fn from(level: JapaneseLevel) -> Self {
        match level {
            JapaneseLevel::N1 => 1,
            JapaneseLevel::N2 => 2,
            JapaneseLevel::N3 => 3,
            JapaneseLevel::N4 => 4,
            JapaneseLevel::N5 => 5,
        }
    }
}

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NativeLanguage {
    English,
    Russian,
}

impl NativeLanguage {
    pub fn as_str(&self) -> &str {
        match self {
            NativeLanguage::English => "English",
            NativeLanguage::Russian => "Russian",
        }
    }
}

impl fmt::Display for NativeLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<i32> for NativeLanguage {
    fn from(value: i32) -> Self {
        match value {
            0 => NativeLanguage::English,
            _ => NativeLanguage::Russian,
        }
    }
}

impl From<NativeLanguage> for i32 {
    fn from(lang: NativeLanguage) -> Self {
        match lang {
            NativeLanguage::English => 0,
            NativeLanguage::Russian => 1,
        }
    }
}

#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DailyLoad {
    Minimal,
    Light,
    #[default]
    Medium,
    Hard,
    Heavy,
    Maximum,
}

impl DailyLoad {
    pub fn new_cards_per_day(&self) -> usize {
        match self {
            DailyLoad::Minimal => 3,
            DailyLoad::Light => 6,
            DailyLoad::Medium => 9,
            DailyLoad::Hard => 15,
            DailyLoad::Heavy => 21,
            DailyLoad::Maximum => 30,
        }
    }

    pub fn all() -> &'static [DailyLoad] {
        &[
            DailyLoad::Minimal,
            DailyLoad::Light,
            DailyLoad::Medium,
            DailyLoad::Hard,
            DailyLoad::Heavy,
            DailyLoad::Maximum,
        ]
    }
}

impl From<i32> for DailyLoad {
    fn from(value: i32) -> Self {
        match value {
            0 => DailyLoad::Minimal,
            1 => DailyLoad::Light,
            2 => DailyLoad::Medium,
            3 => DailyLoad::Hard,
            4 => DailyLoad::Heavy,
            5 => DailyLoad::Maximum,
            _ => DailyLoad::Medium,
        }
    }
}

impl From<DailyLoad> for i32 {
    fn from(val: DailyLoad) -> Self {
        match val {
            DailyLoad::Minimal => 0,
            DailyLoad::Light => 1,
            DailyLoad::Medium => 2,
            DailyLoad::Hard => 3,
            DailyLoad::Heavy => 4,
            DailyLoad::Maximum => 5,
        }
    }
}

impl DailyLoad {
    /// Per-lesson allowance of NEW anchored (interleaved) phrases, derived
    /// from the daily new-card pace. Historically this was a per-DAY budget
    /// (`new_cards_per_day * 2 - phrases_studied_today`) which starved every
    /// lesson after the first one of the day; it is now granted fresh to
    /// every lesson (ADR: per-lesson phrase cap). At Hard/Heavy/Maximum the
    /// binding constraint is usually structural (`INTERLEAVED_PHRASES_PER_WORD`
    /// × anchor words), not this cap — see the ADR for the parity rationale.
    pub fn new_phrases_per_lesson(&self) -> usize {
        self.new_cards_per_day() * PHRASES_PER_NEW_CARD
    }
}

/// How many new anchored phrases one new vocabulary card is worth. Kept next
/// to the budget derivation so the relationship is discoverable.
pub(crate) const PHRASES_PER_NEW_CARD: usize = 2;

/// Derived lesson-building limits for one user ([DailyLoad]).
///
/// `new_cards_per_day` bounds how many new cards may enter lessons on a
/// given day; `new_phrases_per_lesson` bounds how many NEW anchored phrases
/// a single lesson may interleave (fresh for every lesson of the day).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DailyBudget {
    new_cards_per_day: usize,
    new_phrases_per_lesson: usize,
}

impl DailyBudget {
    /// Derives the budget from the user's pace preference. The single
    /// production construction point (`SelectCardsToLessonUseCase`).
    pub fn from_load(load: DailyLoad) -> Self {
        Self {
            new_cards_per_day: load.new_cards_per_day(),
            new_phrases_per_lesson: load.new_phrases_per_lesson(),
        }
    }

    /// Arbitrary budget for tests that exercise limit mechanics with
    /// non-product values (e.g. `daily_new_limit = 1`).
    #[cfg(test)]
    pub fn new(new_cards_per_day: usize, new_phrases_per_lesson: usize) -> Self {
        Self {
            new_cards_per_day,
            new_phrases_per_lesson,
        }
    }

    /// Test convenience: the budget a [`DailyLoad`] with `cards` new cards
    /// per day would yield (phrase cap = cards × 2). Matches the historical
    /// first-lesson-of-the-day allowance for tests written before the
    /// per-lesson cap.
    #[cfg(test)]
    pub fn with_daily_cards(cards: usize) -> Self {
        Self::new(cards, cards * PHRASES_PER_NEW_CARD)
    }

    pub fn new_cards_per_day(&self) -> usize {
        self.new_cards_per_day
    }

    pub fn new_phrases_per_lesson(&self) -> usize {
        self.new_phrases_per_lesson
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json;

    // Question tests
    #[test]
    fn question_new_success() {
        let question = Question::new("valid text".to_string()).unwrap();
        assert_eq!(question.text(), "valid text");
    }

    #[test]
    fn question_new_trims_whitespace() {
        let question = Question::new("  trimmed  ".to_string()).unwrap();
        assert_eq!(question.text(), "trimmed");
    }

    #[test]
    fn question_new_empty_string_error() {
        let result = Question::new("".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidQuestion { .. })));
    }

    #[test]
    fn question_new_whitespace_only_error() {
        let result = Question::new("   ".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidQuestion { .. })));
    }

    // CardAnswer::text tests
    #[test]
    fn card_answer_text_success() {
        let answer = CardAnswer::text("hello".to_string()).unwrap();
        assert_eq!(answer, CardAnswer::Text("hello".to_string()));
    }

    #[test]
    fn card_answer_text_trims_whitespace() {
        let answer = CardAnswer::text("  hello  ".to_string()).unwrap();
        assert_eq!(answer, CardAnswer::Text("hello".to_string()));
    }

    #[test]
    fn card_answer_text_empty_string_error() {
        let result = CardAnswer::text("".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    #[test]
    fn card_answer_text_whitespace_only_error() {
        let result = CardAnswer::text("   ".to_string());
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    // CardAnswer::vocabulary tests
    #[test]
    fn card_answer_vocabulary_success() {
        let answer = CardAnswer::vocabulary(vec!["cat".to_string()], None).unwrap();
        assert!(answer.is_vocabulary());
        assert_eq!(answer.translations(), &["cat".to_string()]);
        assert_eq!(answer.description(), None);
    }

    #[test]
    fn card_answer_vocabulary_empty_translations_error() {
        let result = CardAnswer::vocabulary(vec![], None);
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    #[test]
    fn card_answer_vocabulary_filters_empty_translations() {
        let result = CardAnswer::vocabulary(vec!["".to_string(), "  ".to_string()], None);
        assert!(matches!(result, Err(OrigaError::InvalidAnswer { .. })));
    }

    #[test]
    fn card_answer_vocabulary_caps_to_max_seven_translations() {
        // 12 input translations, only the first 7 must survive.
        let input: Vec<String> = (1..=12).map(|i| format!("sense{i}")).collect();
        let answer = CardAnswer::vocabulary(input.clone(), None).unwrap();
        assert_eq!(
            answer.translations(),
            &input[..7],
            "translations must be capped to the first 7 entries"
        );
    }

    #[test]
    fn card_answer_vocabulary_caps_after_filtering_empties() {
        // Empty strings are filtered first; the cap then applies to the
        // remaining non-empty senses. This guards the filter-then-truncate
        // ordering: filtering 5 empties out of 12 leaves 7, which fits.
        let input: Vec<String> = vec![
            "".to_string(),
            "  ".to_string(),
            "sense1".to_string(),
            "sense2".to_string(),
            "".to_string(),
            "sense3".to_string(),
            "sense4".to_string(),
            "sense5".to_string(),
            "".to_string(),
            "sense6".to_string(),
            "sense7".to_string(),
            "sense8".to_string(),
            "".to_string(),
        ];
        let answer = CardAnswer::vocabulary(input, None).unwrap();
        assert_eq!(
            answer.translations(),
            &[
                "sense1", "sense2", "sense3", "sense4", "sense5", "sense6", "sense7",
            ][..]
        );
    }

    #[test]
    fn card_answer_vocabulary_short_lists_are_not_extended_or_truncated() {
        // Lists at or below the cap must round-trip unchanged.
        let one = CardAnswer::vocabulary(vec!["only".to_string()], None).unwrap();
        assert_eq!(one.translations(), &["only".to_string()][..]);

        let seven: Vec<String> = (1..=7).map(|i| format!("sense{i}")).collect();
        let seven_answer = CardAnswer::vocabulary(seven.clone(), None).unwrap();
        assert_eq!(seven_answer.translations(), &seven[..]);
    }

    #[test]
    fn card_answer_vocabulary_with_description() {
        let answer =
            CardAnswer::vocabulary(vec!["cat".to_string()], Some("feline animal".to_string()))
                .unwrap();
        assert_eq!(answer.description(), Some("feline animal"));
    }

    #[test]
    fn card_answer_vocabulary_trims_description() {
        let answer =
            CardAnswer::vocabulary(vec!["cat".to_string()], Some("  feline  ".to_string()))
                .unwrap();
        assert_eq!(answer.description(), Some("feline"));
    }

    #[test]
    fn card_answer_vocabulary_empty_description_becomes_none() {
        let answer =
            CardAnswer::vocabulary(vec!["cat".to_string()], Some("   ".to_string())).unwrap();
        assert_eq!(answer.description(), None);
    }

    // CardAnswer::translations() for Text variant
    #[test]
    fn card_answer_text_translations_returns_single() {
        let answer = CardAnswer::text("hello".to_string()).unwrap();
        assert_eq!(answer.translations(), &["hello".to_string()]);
    }

    // CardAnswer::description() for Text variant
    #[test]
    fn card_answer_text_description_is_none() {
        let answer = CardAnswer::text("hello".to_string()).unwrap();
        assert!(answer.description().is_none());
    }

    // CardAnswer::is_vocabulary
    #[test]
    fn card_answer_text_is_not_vocabulary() {
        let answer = CardAnswer::text("hello".to_string()).unwrap();
        assert!(!answer.is_vocabulary());
    }

    // Serde roundtrip
    #[test]
    fn card_answer_text_serde_roundtrip() {
        let answer = CardAnswer::text("hello".to_string()).unwrap();
        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: CardAnswer = serde_json::from_str(&json).unwrap();
        assert_eq!(answer, deserialized);
    }

    #[test]
    fn card_answer_vocabulary_serde_roundtrip() {
        let answer = CardAnswer::vocabulary(
            vec!["cat".to_string(), "feline".to_string()],
            Some("a small domesticated carnivorous mammal".to_string()),
        )
        .unwrap();
        let json = serde_json::to_string(&answer).unwrap();
        let deserialized: CardAnswer = serde_json::from_str(&json).unwrap();
        assert_eq!(answer, deserialized);
    }

    // JapaneseLevel::from_str tests
    #[rstest]
    #[case("N5", Ok(JapaneseLevel::N5))]
    #[case("N4", Ok(JapaneseLevel::N4))]
    #[case("N3", Ok(JapaneseLevel::N3))]
    #[case("N2", Ok(JapaneseLevel::N2))]
    #[case("N1", Ok(JapaneseLevel::N1))]
    fn japanese_level_from_str_success(
        #[case] input: &str,
        #[case] expected: Result<JapaneseLevel, String>,
    ) {
        let result: Result<JapaneseLevel, String> = input.parse();
        assert_eq!(result, expected);
    }

    #[test]
    fn japanese_level_from_str_lowercase_success() {
        let result: Result<JapaneseLevel, String> = "n5".parse();
        assert_eq!(result, Ok(JapaneseLevel::N5));
    }

    #[test]
    fn japanese_level_from_str_invalid_error() {
        let result: Result<JapaneseLevel, String> = "invalid".parse();
        assert!(result.is_err());
    }

    #[test]
    fn japanese_level_from_str_empty_error() {
        let result: Result<JapaneseLevel, String> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn japanese_level_from_str_trims_whitespace() {
        let result: Result<JapaneseLevel, String> = "  N5  ".parse();
        assert_eq!(result, Ok(JapaneseLevel::N5));
    }

    // JapaneseLevel::from(i32) tests
    #[rstest]
    #[case(5, JapaneseLevel::N5)]
    #[case(4, JapaneseLevel::N4)]
    #[case(3, JapaneseLevel::N3)]
    #[case(2, JapaneseLevel::N2)]
    #[case(1, JapaneseLevel::N1)]
    fn japanese_level_from_i32_success(#[case] input: i32, #[case] expected: JapaneseLevel) {
        let result = JapaneseLevel::from(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(0, JapaneseLevel::N5)]
    #[case(-1, JapaneseLevel::N5)]
    #[case(99, JapaneseLevel::N5)]
    fn japanese_level_from_i32_fallback(#[case] input: i32, #[case] expected: JapaneseLevel) {
        let result = JapaneseLevel::from(input);
        assert_eq!(result, expected);
    }

    // JapaneseLevel::as_number tests
    #[rstest]
    #[case(JapaneseLevel::N5, 5)]
    #[case(JapaneseLevel::N4, 4)]
    #[case(JapaneseLevel::N3, 3)]
    #[case(JapaneseLevel::N2, 2)]
    #[case(JapaneseLevel::N1, 1)]
    fn japanese_level_as_number(#[case] level: JapaneseLevel, #[case] expected: u8) {
        assert_eq!(level.as_number(), expected);
    }

    // JapaneseLevel::code tests
    #[rstest]
    #[case(JapaneseLevel::N5, "N5")]
    #[case(JapaneseLevel::N4, "N4")]
    #[case(JapaneseLevel::N3, "N3")]
    #[case(JapaneseLevel::N2, "N2")]
    #[case(JapaneseLevel::N1, "N1")]
    fn japanese_level_code(#[case] level: JapaneseLevel, #[case] expected: &str) {
        assert_eq!(level.code(), expected);
    }

    // JapaneseLevel::from_str_or_default tests
    #[rstest]
    #[case("N5", JapaneseLevel::N5)]
    #[case("N4", JapaneseLevel::N4)]
    #[case("N3", JapaneseLevel::N3)]
    #[case("N2", JapaneseLevel::N2)]
    #[case("N1", JapaneseLevel::N1)]
    fn japanese_level_from_str_or_default_success(
        #[case] input: &str,
        #[case] expected: JapaneseLevel,
    ) {
        let result = JapaneseLevel::from_str_or_default(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn japanese_level_from_str_or_default_fallback() {
        let result = JapaneseLevel::from_str_or_default("invalid");
        assert_eq!(result, JapaneseLevel::N1);
    }

    // JapaneseLevel Display trait test
    #[rstest]
    #[case(JapaneseLevel::N5, "5")]
    #[case(JapaneseLevel::N4, "4")]
    #[case(JapaneseLevel::N3, "3")]
    #[case(JapaneseLevel::N2, "2")]
    #[case(JapaneseLevel::N1, "1")]
    fn japanese_level_display(#[case] level: JapaneseLevel, #[case] expected: &str) {
        assert_eq!(format!("{}", level), expected);
    }

    // JapaneseLevel ALL constant test
    #[test]
    fn japanese_level_all_contains_all_levels() {
        assert_eq!(JapaneseLevel::ALL.len(), 5);
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N5));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N4));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N3));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N2));
        assert!(JapaneseLevel::ALL.contains(&JapaneseLevel::N1));
    }

    // NativeLanguage::from(i32) tests
    #[rstest]
    #[case(0, NativeLanguage::English)]
    #[case(1, NativeLanguage::Russian)]
    fn native_language_from_i32_success(#[case] input: i32, #[case] expected: NativeLanguage) {
        let result = NativeLanguage::from(input);
        assert_eq!(result, expected);
    }

    #[rstest]
    #[case(-1, NativeLanguage::Russian)]
    #[case(99, NativeLanguage::Russian)]
    fn native_language_from_i32_fallback(#[case] input: i32, #[case] expected: NativeLanguage) {
        let result = NativeLanguage::from(input);
        assert_eq!(result, expected);
    }

    // NativeLanguage::as_str tests
    #[rstest]
    #[case(NativeLanguage::English, "English")]
    #[case(NativeLanguage::Russian, "Russian")]
    fn native_language_as_str(#[case] lang: NativeLanguage, #[case] expected: &str) {
        assert_eq!(lang.as_str(), expected);
    }

    // NativeLanguage Display trait test
    #[rstest]
    #[case(NativeLanguage::English, "English")]
    #[case(NativeLanguage::Russian, "Russian")]
    fn native_language_display(#[case] lang: NativeLanguage, #[case] expected: &str) {
        assert_eq!(format!("{}", lang), expected);
    }

    // Conversion from JapaneseLevel to i32
    #[rstest]
    #[case(JapaneseLevel::N1, 1)]
    #[case(JapaneseLevel::N2, 2)]
    #[case(JapaneseLevel::N3, 3)]
    #[case(JapaneseLevel::N4, 4)]
    #[case(JapaneseLevel::N5, 5)]
    fn japanese_level_into_i32(#[case] level: JapaneseLevel, #[case] expected: i32) {
        let result: i32 = level.into();
        assert_eq!(result, expected);
    }

    // Conversion from NativeLanguage to i32
    #[rstest]
    #[case(NativeLanguage::English, 0)]
    #[case(NativeLanguage::Russian, 1)]
    fn native_language_into_i32(#[case] lang: NativeLanguage, #[case] expected: i32) {
        let result: i32 = lang.into();
        assert_eq!(result, expected);
    }
}

#[cfg(test)]
#[cfg(test)]
#[cfg(test)]
mod tests_grammar_nuances_answer {
    use super::*;
    use crate::dictionary::grammar::{CommonMistake, NuanceNote, NuanceTag};

    fn mistake(wrong: &str, correct: &str) -> CommonMistake {
        CommonMistake::for_test(wrong, correct)
    }

    #[test]
    fn grammar_nuances_with_items_is_accepted() {
        // Arrange
        let mistakes = vec![mistake("友達を傘を貸してくれた", "友達が傘を貸してくれた")];
        let notes = vec![NuanceNote::for_test(NuanceTag::Register, "spoken register")];

        // Act
        let answer = CardAnswer::grammar_nuances(mistakes, notes);

        // Assert
        assert!(answer.is_ok());
    }

    #[test]
    fn grammar_nuances_without_any_items_is_rejected() {
        // Arrange / Act
        let answer = CardAnswer::grammar_nuances(Vec::new(), Vec::new());

        // Assert
        assert!(answer.is_err());
    }

    #[test]
    fn text_projection_renders_mistake_pair_and_note_without_emoji() {
        // Arrange
        let mistakes = vec![mistake(" を使う", " がを使う")];
        let notes = vec![NuanceNote::for_test(NuanceTag::Variation, "casual: だ")];
        let answer = CardAnswer::grammar_nuances(mistakes, notes).expect("valid answer");

        // Act
        let projection = answer.text_projection();

        // Assert
        assert_eq!(projection, " を使う →  がを使う; casual: だ");
        assert!(!projection.chars().any(|c| c > '\u{2600}' && c < '\u{27C0}'));
    }

    #[test]
    fn translations_is_empty_for_grammar_nuances() {
        // Arrange
        let answer =
            CardAnswer::grammar_nuances(vec![mistake("a", "b")], Vec::new()).expect("valid answer");

        // Act + Assert
        assert!(answer.translations().is_empty());
        assert!(!answer.is_vocabulary());
    }
}

#[cfg(test)]
mod tests_daily_load {
    use super::*;
    use rstest::rstest;

    #[test]
    fn default_is_medium() {
        assert_eq!(DailyLoad::default(), DailyLoad::Medium);
    }

    #[test]
    fn new_cards_per_day_values() {
        assert_eq!(DailyLoad::Minimal.new_cards_per_day(), 3);
        assert_eq!(DailyLoad::Light.new_cards_per_day(), 6);
        assert_eq!(DailyLoad::Medium.new_cards_per_day(), 9);
        assert_eq!(DailyLoad::Hard.new_cards_per_day(), 15);
        assert_eq!(DailyLoad::Heavy.new_cards_per_day(), 21);
        assert_eq!(DailyLoad::Maximum.new_cards_per_day(), 30);
    }

    #[test]
    fn from_i32_roundtrip() {
        for val in DailyLoad::all() {
            let i = i32::from(*val);
            assert_eq!(DailyLoad::from(i), *val);
        }
    }

    #[test]
    fn from_i32_unknown_falls_back_to_medium() {
        assert_eq!(DailyLoad::from(999), DailyLoad::Medium);
    }

    #[rstest]
    #[case::minimal(DailyLoad::Minimal, 3, 6)]
    #[case::light(DailyLoad::Light, 6, 12)]
    #[case::medium(DailyLoad::Medium, 9, 18)]
    #[case::hard(DailyLoad::Hard, 15, 30)]
    #[case::heavy(DailyLoad::Heavy, 21, 42)]
    #[case::maximum(DailyLoad::Maximum, 30, 60)]
    fn daily_budget_from_load_phrases_double_new_cards(
        #[case] load: DailyLoad,
        #[case] expected_cards: usize,
        #[case] expected_phrases: usize,
    ) {
        let budget = DailyBudget::from_load(load);
        assert_eq!(budget.new_cards_per_day(), expected_cards);
        assert_eq!(
            budget.new_phrases_per_lesson(),
            expected_phrases,
            "per-lesson phrase cap must equal the former first-lesson daily allowance"
        );
    }
}
