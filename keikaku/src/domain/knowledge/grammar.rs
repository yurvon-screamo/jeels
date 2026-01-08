use crate::domain::{
    KeikakuError,
    grammar::GrammarRule,
    value_objects::{Answer, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleCard {
    title: Question,
    description: Answer,
}

impl GrammarRuleCard {
    pub fn new(rule: &Box<dyn GrammarRule>, lang: &NativeLanguage) -> Result<Self, KeikakuError> {
        Ok(Self {
            title: Question::new(rule.title(lang))?,
            description: Answer::new(rule.md_description(lang))?,
        })
    }

    pub fn title(&self) -> &Question {
        &self.title
    }

    pub fn description(&self) -> &Answer {
        &self.description
    }
}
