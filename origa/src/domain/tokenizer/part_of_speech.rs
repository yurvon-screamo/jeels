use serde::{Deserialize, Serialize};

use crate::domain::OrigaError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartOfSpeech {
    Verb,
    Noun,
    IAdjective,
    NaAdjective,
    Adverb,
    PreNounAdjectival,
    Conjunction,
    Interjection,
    Prefix,
    Suffix,
    Particle,
    AuxiliaryVerb,
    Pronoun,
    ProperNoun,
    Numeral,
    Determiner,
    Unspecified,
    Other,
    Symbol,
    Whitespace,
    AuxiliarySymbol,
}

impl PartOfSpeech {
    pub fn is_vocabulary_word(&self) -> bool {
        matches!(
            self,
            PartOfSpeech::Noun
                | PartOfSpeech::ProperNoun
                | PartOfSpeech::Verb
                | PartOfSpeech::IAdjective
                | PartOfSpeech::NaAdjective
                | PartOfSpeech::Adverb
                | PartOfSpeech::PreNounAdjectival
                | PartOfSpeech::Pronoun
                | PartOfSpeech::Numeral
                | PartOfSpeech::Determiner
                | PartOfSpeech::Conjunction
                | PartOfSpeech::Interjection
        )
    }
}

impl std::str::FromStr for PartOfSpeech {
    type Err = OrigaError;

    fn from_str(japanese: &str) -> Result<Self, Self::Err> {
        Ok(match japanese {
            "動詞" => PartOfSpeech::Verb,
            "名詞" => PartOfSpeech::Noun,
            "形容詞" => PartOfSpeech::IAdjective,
            "形状詞" => PartOfSpeech::NaAdjective,
            "副詞" => PartOfSpeech::Adverb,
            "連体詞" => PartOfSpeech::PreNounAdjectival,
            "接続詞" => PartOfSpeech::Conjunction,
            "感動詞" => PartOfSpeech::Interjection,
            "接頭辞" => PartOfSpeech::Prefix,
            "接尾辞" => PartOfSpeech::Suffix,
            "助詞" => PartOfSpeech::Particle,
            "助動詞" => PartOfSpeech::AuxiliaryVerb,
            "代名詞" => PartOfSpeech::Pronoun,
            "数詞" => PartOfSpeech::Numeral,
            "限定詞" => PartOfSpeech::Determiner,
            "未特定" => PartOfSpeech::Unspecified,
            "その他" => PartOfSpeech::Other,
            "記号" => PartOfSpeech::Symbol,
            "空白" => PartOfSpeech::Whitespace,
            "補助記号" => PartOfSpeech::AuxiliarySymbol,
            _ => {
                return Err(OrigaError::TokenizerError {
                    reason: format!("Unknown part of speech: '{japanese}'"),
                });
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_str {
        use super::*;

        #[test]
        fn parses_verb() {
            let result = "動詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Verb));
        }

        #[test]
        fn parses_noun() {
            let result = "名詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Noun));
        }

        #[test]
        fn parses_i_adjective() {
            let result = "形容詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::IAdjective));
        }

        #[test]
        fn parses_na_adjective() {
            let result = "形状詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::NaAdjective));
        }

        #[test]
        fn parses_adverb() {
            let result = "副詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Adverb));
        }

        #[test]
        fn parses_pre_noun_adjectival() {
            let result = "連体詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::PreNounAdjectival));
        }

        #[test]
        fn parses_conjunction() {
            let result = "接続詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Conjunction));
        }

        #[test]
        fn parses_interjection() {
            let result = "感動詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Interjection));
        }

        #[test]
        fn parses_prefix() {
            let result = "接頭辞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Prefix));
        }

        #[test]
        fn parses_suffix() {
            let result = "接尾辞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Suffix));
        }

        #[test]
        fn parses_particle() {
            let result = "助詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Particle));
        }

        #[test]
        fn parses_auxiliary_verb() {
            let result = "助動詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::AuxiliaryVerb));
        }

        #[test]
        fn parses_pronoun() {
            let result = "代名詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Pronoun));
        }

        #[test]
        fn parses_numeral() {
            let result = "数詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Numeral));
        }

        #[test]
        fn parses_determiner() {
            let result = "限定詞".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Determiner));
        }

        #[test]
        fn parses_unspecified() {
            let result = "未特定".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Unspecified));
        }

        #[test]
        fn parses_other() {
            let result = "その他".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Other));
        }

        #[test]
        fn parses_symbol() {
            let result = "記号".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Symbol));
        }

        #[test]
        fn parses_whitespace() {
            let result = "空白".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::Whitespace));
        }

        #[test]
        fn parses_auxiliary_symbol() {
            let result = "補助記号".parse::<PartOfSpeech>();
            assert_eq!(result, Ok(PartOfSpeech::AuxiliarySymbol));
        }

        #[test]
        fn returns_error_for_unknown_input() {
            let result = "unknown".parse::<PartOfSpeech>();
            assert!(matches!(result, Err(OrigaError::TokenizerError { .. })));
        }
    }

    mod is_vocabulary_word {
        use super::*;

        #[test]
        fn returns_true_for_noun() {
            assert!(PartOfSpeech::Noun.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_proper_noun() {
            assert!(PartOfSpeech::ProperNoun.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_verb() {
            assert!(PartOfSpeech::Verb.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_i_adjective() {
            assert!(PartOfSpeech::IAdjective.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_na_adjective() {
            assert!(PartOfSpeech::NaAdjective.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_adverb() {
            assert!(PartOfSpeech::Adverb.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_pre_noun_adjectival() {
            assert!(PartOfSpeech::PreNounAdjectival.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_conjunction() {
            assert!(PartOfSpeech::Conjunction.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_pronoun() {
            assert!(PartOfSpeech::Pronoun.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_numeral() {
            assert!(PartOfSpeech::Numeral.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_determiner() {
            assert!(PartOfSpeech::Determiner.is_vocabulary_word());
        }

        #[test]
        fn returns_true_for_interjection() {
            // Interjections like ありがとう / こんにちは are real dictionary
            // words learners study — they must reach card analysis.
            assert!(PartOfSpeech::Interjection.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_prefix() {
            assert!(!PartOfSpeech::Prefix.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_suffix() {
            assert!(!PartOfSpeech::Suffix.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_particle() {
            assert!(!PartOfSpeech::Particle.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_auxiliary_verb() {
            assert!(!PartOfSpeech::AuxiliaryVerb.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_unspecified() {
            assert!(!PartOfSpeech::Unspecified.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_other() {
            assert!(!PartOfSpeech::Other.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_symbol() {
            assert!(!PartOfSpeech::Symbol.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_whitespace() {
            assert!(!PartOfSpeech::Whitespace.is_vocabulary_word());
        }

        #[test]
        fn returns_false_for_auxiliary_symbol() {
            assert!(!PartOfSpeech::AuxiliarySymbol.is_vocabulary_word());
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn roundtrip_noun() {
            let json = serde_json::to_string(&PartOfSpeech::Noun).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Noun, decoded);
        }

        #[test]
        fn roundtrip_proper_noun() {
            let json = serde_json::to_string(&PartOfSpeech::ProperNoun).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::ProperNoun, decoded);
        }

        #[test]
        fn roundtrip_verb() {
            let json = serde_json::to_string(&PartOfSpeech::Verb).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Verb, decoded);
        }

        #[test]
        fn roundtrip_i_adjective() {
            let json = serde_json::to_string(&PartOfSpeech::IAdjective).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::IAdjective, decoded);
        }

        #[test]
        fn roundtrip_na_adjective() {
            let json = serde_json::to_string(&PartOfSpeech::NaAdjective).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::NaAdjective, decoded);
        }

        #[test]
        fn roundtrip_adverb() {
            let json = serde_json::to_string(&PartOfSpeech::Adverb).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Adverb, decoded);
        }

        #[test]
        fn roundtrip_pre_noun_adjectival() {
            let json = serde_json::to_string(&PartOfSpeech::PreNounAdjectival).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::PreNounAdjectival, decoded);
        }

        #[test]
        fn roundtrip_conjunction() {
            let json = serde_json::to_string(&PartOfSpeech::Conjunction).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Conjunction, decoded);
        }

        #[test]
        fn roundtrip_interjection() {
            let json = serde_json::to_string(&PartOfSpeech::Interjection).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Interjection, decoded);
        }

        #[test]
        fn roundtrip_prefix() {
            let json = serde_json::to_string(&PartOfSpeech::Prefix).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Prefix, decoded);
        }

        #[test]
        fn roundtrip_suffix() {
            let json = serde_json::to_string(&PartOfSpeech::Suffix).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Suffix, decoded);
        }

        #[test]
        fn roundtrip_particle() {
            let json = serde_json::to_string(&PartOfSpeech::Particle).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Particle, decoded);
        }

        #[test]
        fn roundtrip_auxiliary_verb() {
            let json = serde_json::to_string(&PartOfSpeech::AuxiliaryVerb).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::AuxiliaryVerb, decoded);
        }

        #[test]
        fn roundtrip_pronoun() {
            let json = serde_json::to_string(&PartOfSpeech::Pronoun).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Pronoun, decoded);
        }

        #[test]
        fn roundtrip_numeral() {
            let json = serde_json::to_string(&PartOfSpeech::Numeral).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Numeral, decoded);
        }

        #[test]
        fn roundtrip_determiner() {
            let json = serde_json::to_string(&PartOfSpeech::Determiner).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Determiner, decoded);
        }

        #[test]
        fn roundtrip_unspecified() {
            let json = serde_json::to_string(&PartOfSpeech::Unspecified).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Unspecified, decoded);
        }

        #[test]
        fn roundtrip_other() {
            let json = serde_json::to_string(&PartOfSpeech::Other).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Other, decoded);
        }

        #[test]
        fn roundtrip_symbol() {
            let json = serde_json::to_string(&PartOfSpeech::Symbol).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Symbol, decoded);
        }

        #[test]
        fn roundtrip_whitespace() {
            let json = serde_json::to_string(&PartOfSpeech::Whitespace).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::Whitespace, decoded);
        }

        #[test]
        fn roundtrip_auxiliary_symbol() {
            let json = serde_json::to_string(&PartOfSpeech::AuxiliarySymbol).unwrap();
            let decoded: PartOfSpeech = serde_json::from_str(&json).unwrap();
            assert_eq!(PartOfSpeech::AuxiliarySymbol, decoded);
        }
    }
}
