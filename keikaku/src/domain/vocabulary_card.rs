use crate::domain::dictionary::{KANJI_DB, KanjiInfo};
use crate::domain::japanese::IsJapanese;
use crate::domain::review::{MemoryHistory, MemoryState, Review};
use crate::domain::value_objects::{Answer, CardContent, ExamplePhrase, JapaneseLevel, Question};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabularyCard {
    id: Ulid,
    word: Question,
    meaning: Answer,
    example_phrases: Vec<ExamplePhrase>,
    memory_history: MemoryHistory,
}

impl VocabularyCard {
    pub fn new(question: Question, content: CardContent) -> Self {
        Self {
            id: Ulid::new(),
            word: question,
            memory_history: MemoryHistory::new(),
            meaning: content.answer().clone(),
            example_phrases: content.example_phrases().to_vec(),
        }
    }

    pub fn id(&self) -> Ulid {
        self.id
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

    pub fn memory(&self) -> &MemoryHistory {
        &self.memory_history
    }

    pub(crate) fn edit(
        &mut self,
        new_question: Question,
        new_answer: Answer,
        new_example_phrases: Vec<ExamplePhrase>,
    ) {
        self.word = new_question;
        self.meaning = new_answer;
        self.example_phrases = new_example_phrases;
    }

    pub(crate) fn add_review(&mut self, memory_state: MemoryState, review: Review) {
        self.memory_history.add_review(memory_state, review);
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
