use crate::dictionary::grammar::{RelatedPattern, get_rule_by_id};
use crate::domain::OrigaError;
use crate::domain::{
    tokenizer::PartOfSpeech,
    value_objects::{CardAnswer, NativeLanguage, Question},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

macro_rules! get_content {
    ($self:expr, $lang:expr, $content_method:ident, $new_method:path) => {{
        let rule = get_rule_by_id(&$self.rule_id).ok_or(OrigaError::GrammarRuleNotFound {
            rule_id: $self.rule_id,
        })?;

        let text = rule.content($lang).$content_method();
        if text.is_empty() {
            return Err(OrigaError::GrammarContentNotFound {
                rule_id: $self.rule_id,
                lang: *$lang,
            });
        }

        $new_method(text.to_string())
    }};
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrammarRuleCard {
    rule_id: Ulid,
}

impl GrammarRuleCard {
    pub fn new(rule_id: Ulid) -> Result<Self, OrigaError> {
        get_rule_by_id(&rule_id).ok_or_else(|| OrigaError::RepositoryError {
            reason: format!("Grammar rule {} not found", rule_id),
        })?;
        Ok(Self { rule_id })
    }

    pub fn rule_id(&self) -> &Ulid {
        &self.rule_id
    }

    pub fn title(&self, lang: &NativeLanguage) -> Result<Question, OrigaError> {
        get_content!(self, lang, title, Question::new)
    }

    pub fn description(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        self.short_description(lang)
    }

    pub fn short_description(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        get_content!(self, lang, short_description, CardAnswer::text)
    }

    pub fn explanation(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        get_content!(self, lang, explanation, CardAnswer::text)
    }

    pub fn how_to_form(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        get_content!(self, lang, how_to_form, CardAnswer::text)
    }

    pub fn examples(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        get_content!(self, lang, examples, CardAnswer::text)
    }

    pub fn nuances(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        let rule = get_rule_by_id(&self.rule_id).ok_or(OrigaError::GrammarRuleNotFound {
            rule_id: self.rule_id,
        })?;

        let nuances = rule.content(lang).nuances();
        if nuances.is_empty() {
            return Err(OrigaError::GrammarContentNotFound {
                rule_id: self.rule_id,
                lang: *lang,
            });
        }

        CardAnswer::grammar_nuances(nuances.common_mistakes().to_vec(), nuances.notes().to_vec())
    }

    pub fn pro_tip(&self, lang: &NativeLanguage) -> Result<CardAnswer, OrigaError> {
        get_content!(self, lang, pro_tip, CardAnswer::text)
    }

    pub fn related_patterns(&self, lang: &NativeLanguage) -> &[RelatedPattern] {
        get_rule_by_id(&self.rule_id)
            .map(|rule| rule.content(lang).related_patterns())
            .unwrap_or(&[])
    }

    pub fn apply_to(&self) -> Vec<PartOfSpeech> {
        get_rule_by_id(&self.rule_id)
            .map(|rule| rule.apply_to())
            .unwrap_or_default()
    }

    pub fn has_format_map(&self) -> bool {
        get_rule_by_id(&self.rule_id)
            .map(|rule| rule.has_format_map())
            .unwrap_or(false)
    }
}

impl GrammarRuleCard {
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self {
            rule_id: Ulid::new(),
        }
    }

    #[cfg(test)]
    pub fn new_test_with_id(rule_id: Ulid) -> Self {
        Self { rule_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::grammar::{
        GrammarData, get_rules_by_level, init_grammar, is_grammar_loaded, iter_grammar_rules,
    };
    use crate::domain::{JapaneseLevel, NativeLanguage};
    use std::sync::Once;

    static INIT: Once = Once::new();

    // Single source of truth for the test grammar corpus location:
    // <workspace_root>/cdn/grammar/grammar_v2.json (schema v2)
    fn grammar_json_path() -> Option<std::path::PathBuf> {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
        Some(
            std::path::PathBuf::from(manifest_dir)
                .parent()?
                .join("cdn")
                .join("grammar")
                .join("grammar_v2.json"),
        )
    }

    fn init_test_grammar() {
        INIT.call_once(|| {
            if is_grammar_loaded() {
                return;
            }

            let grammar_path = grammar_json_path().expect("Failed to resolve grammar.json path");

            tracing::debug!("Reading grammar from: {:?}", grammar_path);
            let grammar_json =
                std::fs::read_to_string(&grammar_path).expect("Failed to read grammar.json");
            tracing::debug!("Read {} bytes from grammar.json", grammar_json.len());

            let grammar_data = GrammarData { grammar_json };
            init_grammar(grammar_data).expect("Failed to init grammar rules");
        });
    }

    fn get_first_rule_id() -> Ulid {
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    fn get_verb_rule_id() -> Ulid {
        Ulid::from_string("01G00000000000000024000000").expect("Invalid ULID")
    }

    #[test]
    fn get_rules_by_level_returns_n3_and_n2() {
        init_test_grammar();

        let total = iter_grammar_rules().count();
        let n5 = get_rules_by_level(&JapaneseLevel::N5).len();
        let n4 = get_rules_by_level(&JapaneseLevel::N4).len();
        let n3 = get_rules_by_level(&JapaneseLevel::N3).len();
        let n2 = get_rules_by_level(&JapaneseLevel::N2).len();
        let n1 = get_rules_by_level(&JapaneseLevel::N1).len();

        // Diagnostic for CI: visible with `--nocapture`, and cargo test also
        // prints captured stderr on failure, so this surfaces in CI logs when
        // the test actually breaks.
        let grammar_path = grammar_json_path().unwrap_or_default();
        let file_size = std::fs::metadata(&grammar_path)
            .map(|m| m.len())
            .unwrap_or(0);

        eprintln!("get_rules_by_level_returns_n3_and_n2 diagnostic:");
        eprintln!("  grammar.json path: {}", grammar_path.display());
        eprintln!("  grammar.json size: {} bytes", file_size);
        eprintln!("  total rules loaded: {}", total);
        eprintln!("  N5={}, N4={}, N3={}, N2={}, N1={}", n5, n4, n3, n2, n1);

        if n3 == 0 || n2 == 0 {
            panic!(
                "N3/N2 grammar rules missing. grammar.json at {} ({} bytes), \
                 loaded {} rules: N5={}, N4={}, N3={}, N2={}, N1={}. \
                 Likely causes: N3/N2 are 0 -> grammar.json is STALE (old CDN cache); \
                 total unexpectedly low -> file incompletely downloaded. \
                 Inspect the CDN download cache in CI (.github/actions/download-cdn) \
                 and purge the stale cache-key if needed.",
                grammar_path.display(),
                file_size,
                total,
                n5,
                n4,
                n3,
                n2,
                n1
            );
        }

        assert!(n3 >= 100, "Expected at least 100 N3 rules, got {}", n3);
        assert!(n2 >= 80, "Expected at least 80 N2 rules, got {}", n2);
    }

    mod new {
        use super::*;

        #[test]
        fn creates_card_with_valid_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id);

            assert!(card.is_ok());
            assert_eq!(*card.unwrap().rule_id(), rule_id);
        }

        #[test]
        fn returns_error_for_nonexistent_rule_id() {
            init_test_grammar();

            let nonexistent_id = Ulid::new();
            let result = GrammarRuleCard::new(nonexistent_id);

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, OrigaError::RepositoryError { .. }));
        }
    }

    mod rule_id {
        use super::*;

        #[test]
        fn returns_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            assert_eq!(*card.rule_id(), rule_id);
        }

        #[test]
        fn new_test_creates_card_with_any_rule_id() {
            let card = GrammarRuleCard::new_test();

            assert!(!card.rule_id().is_nil());
        }
    }

    mod title {
        use super::*;

        #[test]
        fn returns_title_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let title = card.title(&NativeLanguage::Russian);

            assert!(title.is_ok());
            assert!(!title.unwrap().text().is_empty());
        }

        #[test]
        fn returns_title_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let title = card.title(&NativeLanguage::English);

            assert!(title.is_ok());
            assert!(!title.unwrap().text().is_empty());
        }

        #[test]
        fn returns_different_titles_for_different_languages() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let russian_title = card.title(&NativeLanguage::Russian).unwrap();
            let english_title = card.title(&NativeLanguage::English).unwrap();

            assert_eq!(russian_title.text(), english_title.text());
        }
    }

    mod description {
        use super::*;

        #[test]
        fn returns_description_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let desc = card.description(&NativeLanguage::Russian);
            let short = card.short_description(&NativeLanguage::Russian);

            assert!(desc.is_ok());
            assert!(short.is_ok());
            assert_eq!(
                desc.unwrap(),
                short.unwrap(),
                "description() should delegate to short_description()"
            );
        }

        #[test]
        fn returns_description_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let desc = card.description(&NativeLanguage::English);
            let short = card.short_description(&NativeLanguage::English);

            assert!(desc.is_ok());
            assert!(short.is_ok());
            assert_eq!(
                desc.unwrap(),
                short.unwrap(),
                "description() should delegate to short_description()"
            );
        }

        #[test]
        fn description_equals_short_description() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            for lang in [NativeLanguage::Russian, NativeLanguage::English] {
                let desc = card.description(&lang).expect("description should work");
                let short = card
                    .short_description(&lang)
                    .expect("short_description should work");
                assert_eq!(
                    desc, short,
                    "description() should equal short_description() for {lang:?}"
                );
            }
        }
    }

    mod apply_to {
        use super::*;
        use crate::domain::tokenizer::PartOfSpeech;

        #[test]
        fn returns_parts_of_speech_for_rule_with_format_map() {
            init_test_grammar();

            let rule_id = get_verb_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let parts = card.apply_to();

            assert!(!parts.is_empty());
            assert!(parts.contains(&PartOfSpeech::Verb));
        }
    }

    mod short_description {
        use super::*;

        #[test]
        fn returns_short_description_in_russian() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let short_desc = card.short_description(&NativeLanguage::Russian);

            assert!(short_desc.is_ok());
            match short_desc.unwrap() {
                CardAnswer::Text(s) => assert!(!s.is_empty()),
                other => panic!("Expected Text variant, got {:?}", other),
            }
        }

        #[test]
        fn returns_short_description_in_english() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let short_desc = card.short_description(&NativeLanguage::English);

            assert!(short_desc.is_ok());
            match short_desc.unwrap() {
                CardAnswer::Text(s) => assert!(!s.is_empty()),
                other => panic!("Expected Text variant, got {:?}", other),
            }
        }

        #[test]
        fn returns_error_for_nonexistent_rule() {
            init_test_grammar();

            let card = GrammarRuleCard::new_test();
            let result = card.short_description(&NativeLanguage::Russian);

            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(matches!(err, OrigaError::GrammarRuleNotFound { .. }));
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn serialization_roundtrip() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let json = serde_json::to_string(&card).expect("Failed to serialize");
            let deserialized: GrammarRuleCard =
                serde_json::from_str(&json).expect("Failed to deserialize");

            assert_eq!(deserialized.rule_id(), card.rule_id());
        }

        #[test]
        fn serialization_contains_rule_id() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");

            let json = serde_json::to_string(&card).expect("Failed to serialize");

            assert!(json.contains("rule_id"));
        }

        #[test]
        fn card_serialization_roundtrip() {
            let card = GrammarRuleCard::new_test();

            let json = serde_json::to_string(&card).expect("Failed to serialize");
            let deserialized: GrammarRuleCard =
                serde_json::from_str(&json).expect("Failed to deserialize");

            assert_eq!(deserialized.rule_id(), card.rule_id());
        }
    }

    mod clone_and_equality {
        use super::*;

        #[test]
        fn clone_produces_equal_card() {
            init_test_grammar();

            let rule_id = get_first_rule_id();
            let card = GrammarRuleCard::new(rule_id).expect("Failed to create card");
            let cloned = card.clone();

            assert_eq!(card, cloned);
        }

        #[test]
        fn different_rule_ids_produce_different_cards() {
            init_test_grammar();

            let rule_id1 = get_first_rule_id();
            let rule_id2 = Ulid::from_string("01G00000000000000004000000").expect("Invalid ULID");

            let card1 = GrammarRuleCard::new(rule_id1).expect("Failed to create card1");
            let card2 = GrammarRuleCard::new(rule_id2).expect("Failed to create card2");

            assert_ne!(card1, card2);
        }
    }
}
