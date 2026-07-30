use crate::dictionary::kanji::get_kanji_info;
use crate::dictionary::radical::{RadicalInfo, get_radical_info};
use crate::dictionary::vocabulary::get_translation;
use crate::domain::{CardAnswer, JapaneseLevel, NativeLanguage, OrigaError, Question};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KanjiCard {
    kanji: Question,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExampleKanjiWord {
    word: String,
    meaning: String,
}

impl KanjiCard {
    pub fn new(kanji: String) -> Result<Self, OrigaError> {
        get_kanji_info(&kanji)?;
        Ok(Self {
            kanji: Question::new(kanji)?,
        })
    }

    pub fn kanji(&self) -> &Question {
        &self.kanji
    }

    pub fn description(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        let info = get_kanji_info(self.kanji.text()).map_err(|_| OrigaError::KanjiNotFound {
            kanji: self.kanji.text().to_string(),
        })?;
        CardAnswer::text(info.description(lang).to_string()).map_err(|e| {
            OrigaError::InvalidAnswer {
                reason: e.to_string(),
            }
        })
    }

    pub fn example_words(&self, lang: &NativeLanguage) -> Vec<ExampleKanjiWord> {
        get_kanji_info(self.kanji.text())
            .map(|info| {
                info.popular_words()
                    .iter()
                    .filter_map(|word| {
                        let meaning = get_translation(word, lang).unwrap_or_default();
                        if meaning.is_empty() {
                            None
                        } else {
                            Some(ExampleKanjiWord {
                                word: word.clone(),
                                meaning,
                            })
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn jlpt(&self) -> JapaneseLevel {
        get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.jlpt().to_owned())
            .unwrap_or(JapaneseLevel::N1)
    }

    pub fn used_in(&self) -> u32 {
        get_kanji_info(self.kanji.text())
            .map(|kanji_info| kanji_info.used_in())
            .unwrap_or(0)
    }

    pub fn radicals_info(&self) -> Result<Vec<&'static RadicalInfo>, OrigaError> {
        let kanji_info = get_kanji_info(self.kanji.text())?;
        kanji_info
            .radicals_chars()
            .iter()
            .map(|&r| get_radical_info(r))
            .collect()
    }

    pub fn radicals_chars(&self) -> Vec<char> {
        get_kanji_info(self.kanji.text())
            .map(|info| info.radicals_chars().to_vec())
            .unwrap_or_default()
    }

    pub fn on_readings(&self) -> Vec<String> {
        get_kanji_info(self.kanji.text())
            .map(|info| info.on_readings().to_vec())
            .unwrap_or_default()
    }

    pub fn kun_readings(&self) -> Vec<String> {
        get_kanji_info(self.kanji.text())
            .map(|info| info.kun_readings().to_vec())
            .unwrap_or_default()
    }

    /// On readings paired with their corpus frequency and rarity flag. Pairs
    /// preserve the order of [`on_readings`]. Frequency is `0` and rarity is
    /// `false` when no data is available (legacy kanji.json); callers receive
    /// a fully-resolved view so the UI never has to interpret the threshold.
    pub fn on_readings_with_freq(&self) -> Vec<(String, u32, bool)> {
        get_kanji_info(self.kanji.text())
            .map(|info| {
                info.on_readings()
                    .iter()
                    .map(|r| {
                        (
                            r.clone(),
                            info.reading_frequency(r).unwrap_or(0),
                            info.is_rare_reading(r),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Kun readings paired with their corpus frequency and rarity flag. See
    /// [`on_readings_with_freq`].
    pub fn kun_readings_with_freq(&self) -> Vec<(String, u32, bool)> {
        get_kanji_info(self.kanji.text())
            .map(|info| {
                info.kun_readings()
                    .iter()
                    .map(|r| {
                        (
                            r.clone(),
                            info.reading_frequency(r).unwrap_or(0),
                            info.is_rare_reading(r),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether the given reading should be treated as rare for this kanji.
    /// Delegates to [`KanjiInfo::is_rare_reading`]; returns `false` if the
    /// kanji is not in the dictionary.
    pub fn is_rare_reading(&self, reading: &str) -> bool {
        get_kanji_info(self.kanji.text())
            .map(|info| info.is_rare_reading(reading))
            .unwrap_or(false)
    }
}

impl ExampleKanjiWord {
    pub fn word(&self) -> &str {
        &self.word
    }

    pub fn meaning(&self) -> &str {
        &self.meaning
    }
}

impl KanjiCard {
    #[cfg(test)]
    pub fn new_test(kanji: String) -> Self {
        Self {
            kanji: Question::new(kanji).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::init_real_dictionaries;

    fn setup() {
        init_real_dictionaries();
    }

    #[test]
    fn new_test_creates_valid_card() {
        let kanji = KanjiCard::new_test("日".to_string());
        assert_eq!(kanji.kanji().text(), "日");
    }

    #[test]
    fn kanji_returns_question() {
        let kanji = KanjiCard::new_test("月".to_string());
        let question = kanji.kanji();
        assert_eq!(question.text(), "月");
    }

    #[test]
    fn description_returns_answer_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let result = kanji.description(&NativeLanguage::Russian);
        assert!(result.is_ok());
        match result.unwrap() {
            CardAnswer::Text(s) => assert!(!s.is_empty()),
            other => panic!("Expected Text variant for kanji, got {:?}", other),
        }
    }

    #[test]
    fn example_words_returns_words_with_meanings() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let words = kanji.example_words(&NativeLanguage::Russian);
        assert!(!words.is_empty());
        for word in &words {
            assert!(!word.word().is_empty());
            assert!(!word.meaning().is_empty());
        }
    }

    #[test]
    fn example_words_english_returns_words_with_meanings() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let words = kanji.example_words(&NativeLanguage::English);
        assert!(!words.is_empty());
    }

    #[test]
    fn jlpt_returns_level_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let level = kanji.jlpt();
        assert!(matches!(
            level,
            JapaneseLevel::N5
                | JapaneseLevel::N4
                | JapaneseLevel::N3
                | JapaneseLevel::N2
                | JapaneseLevel::N1
        ));
    }

    #[test]
    fn jlpt_returns_n1_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let level = kanji.jlpt();
        assert_eq!(level, JapaneseLevel::N1);
    }

    #[test]
    fn used_in_returns_count_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let count = kanji.used_in();
        assert!(count > 0);
    }

    #[test]
    fn used_in_returns_zero_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let count = kanji.used_in();
        assert_eq!(count, 0);
    }

    #[test]
    fn radicals_chars_returns_chars_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let radicals = kanji.radicals_chars();
        assert!(!radicals.is_empty());
    }

    #[test]
    fn radicals_chars_returns_empty_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let radicals = kanji.radicals_chars();
        assert!(radicals.is_empty());
    }

    #[test]
    fn radicals_info_returns_infos_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let result = kanji.radicals_info();
        assert!(result.is_ok());
        let infos = result.unwrap();
        assert!(!infos.is_empty());
    }

    #[test]
    fn radicals_info_returns_error_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let result = kanji.radicals_info();
        assert!(result.is_err());
    }

    #[test]
    fn on_readings_returns_readings_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let readings = kanji.on_readings();
        assert!(!readings.is_empty());
    }

    #[test]
    fn on_readings_returns_empty_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let readings = kanji.on_readings();
        assert!(readings.is_empty());
    }

    #[test]
    fn kun_readings_returns_readings_for_known_kanji() {
        setup();
        let kanji = KanjiCard::new_test("日".to_string());
        let readings = kanji.kun_readings();
        assert!(!readings.is_empty());
    }

    #[test]
    fn kun_readings_returns_empty_for_unknown_kanji() {
        let kanji = KanjiCard::new_test("𛀀".to_string());
        let readings = kanji.kun_readings();
        assert!(readings.is_empty());
    }

    #[test]
    fn serialization_roundtrip() {
        let original = KanjiCard::new_test("日".to_string());
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: KanjiCard = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn example_kanji_word_returns_word() {
        let word = ExampleKanjiWord {
            word: "日本".to_string(),
            meaning: "Япония".to_string(),
        };
        assert_eq!(word.word(), "日本");
    }

    #[test]
    fn example_kanji_word_returns_meaning() {
        let word = ExampleKanjiWord {
            word: "日本".to_string(),
            meaning: "Япония".to_string(),
        };
        assert_eq!(word.meaning(), "Япония");
    }

    #[test]
    fn example_kanji_word_serialization_roundtrip() {
        let original = ExampleKanjiWord {
            word: "日本".to_string(),
            meaning: "Япония".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ExampleKanjiWord = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn new_creates_card_for_known_kanji() {
        setup();
        let result = KanjiCard::new("日".to_string());
        assert!(result.is_ok());
        let kanji = result.unwrap();
        assert_eq!(kanji.kanji().text(), "日");
    }

    #[test]
    fn new_fails_for_empty_kanji() {
        let result = KanjiCard::new("".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn new_fails_for_unknown_kanji() {
        let result = KanjiCard::new("𛀀".to_string());
        assert!(result.is_err());
    }
}
