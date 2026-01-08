use crate::domain::dictionary::{KANJI_DB, KanjiInfo};
use crate::domain::japanese::IsJapanese;
use crate::domain::value_objects::{Answer, ExamplePhrase, JapaneseLevel, Question};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VocabularyCard {
    word: Question,
    meaning: Answer,
    example_phrases: Vec<ExamplePhrase>,
}

impl VocabularyCard {
    pub fn new(word: Question, meaning: Answer, example_phrases: Vec<ExamplePhrase>) -> Self {
        Self {
            word,
            meaning,
            example_phrases,
        }
    }

    pub fn word(&self) -> &Question {
        &self.word
    }

    pub fn meaning(&self) -> &Answer {
        &self.meaning
    }

    pub fn example_phrases(&self) -> &[ExamplePhrase] {
        &self.example_phrases
    }

    pub fn get_kanji_cards(&self, current_level: &JapaneseLevel) -> Vec<&KanjiInfo> {
        self.word
            .text()
            .chars()
            .filter(|c| c.is_kanji())
            .filter_map(|c| KANJI_DB.get_kanji_info(&c.to_string()).ok())
            .filter(|k| k.jlpt() <= current_level)
            .collect::<Vec<_>>()
    }
}
