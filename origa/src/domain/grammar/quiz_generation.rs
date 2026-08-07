use std::collections::HashSet;

use rand::Rng;
use rand::prelude::IndexedRandom;
use rand::prelude::SliceRandom;

use crate::dictionary::grammar::{FormatAction, FormatActionGroup, GrammarRule};
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::{Card, OrigaError, PartOfSpeech, apply_format_actions};

/// Realistic N3/N2 grammatical postfixes that attach directly to dictionary-form words.
/// Used to generate plausible distractors for Universal-only chains (AddPostfix/ReplacePostfix
/// without any conjugating action). Pool intentionally excludes postfixes that require
/// a conjugated base (e.g. ください expects te-form), since those would produce invalid Japanese.
const UNIVERSAL_DISTRACTOR_POSTFIXES: &[&str] = &[
    "こと",
    "ところ",
    "はず",
    "つもり",
    "よう",
    "まま",
    "あいだ",
    "ため",
    "ばあい",
    "おかげ",
    "せい",
    "うえ",
];

#[derive(Debug, Clone, PartialEq)]
pub struct GrammarPracticeQuestion {
    word_text: String,
    options: Vec<String>,
    correct_index: usize,
}

impl GrammarPracticeQuestion {
    pub fn word_text(&self) -> &str {
        &self.word_text
    }

    pub fn options(&self) -> &[String] {
        &self.options
    }

    pub fn correct_index(&self) -> usize {
        self.correct_index
    }
}

pub fn generate_grammar_practice_questions(
    rule: &GrammarRule,
    knowledge_set: &KnowledgeSet,
    count: usize,
    rng: &mut impl Rng,
) -> Result<Vec<GrammarPracticeQuestion>, OrigaError> {
    let pos = rule
        .apply_to()
        .into_iter()
        .next()
        .ok_or_else(|| OrigaError::GrammarFormatError {
            reason: "Rule has no supported parts of speech".to_string(),
        })?;

    let mut words = find_known_vocab_words_for_pos(knowledge_set, &pos);
    words.shuffle(rng);

    let selected_words: Vec<String> = words.into_iter().take(count).collect();

    let mut questions = Vec::with_capacity(selected_words.len());

    for word in selected_words {
        let correct = rule.format(&word, &pos)?;

        let actions = match rule.format_actions_for_pos(&pos) {
            Some(a) => a,
            None => continue,
        };

        let distractors = generate_grammar_distractors(actions, &word, &pos, &correct, 3, rng);

        if distractors.len() < 3 {
            continue;
        }

        let mut options = distractors;
        let correct_index = rng.random_range(0..=options.len());
        options.insert(correct_index, correct);

        questions.push(GrammarPracticeQuestion {
            word_text: word,
            options,
            correct_index,
        });
    }

    Ok(questions)
}

pub fn find_known_vocab_words_for_pos(
    knowledge_set: &KnowledgeSet,
    pos: &PartOfSpeech,
) -> Vec<String> {
    knowledge_set
        .study_cards()
        .values()
        .filter(|sc| sc.memory().is_known_card() || sc.memory().is_in_progress())
        .filter_map(|sc| match sc.card() {
            Card::Vocabulary(v) => {
                let word = v.word().text().to_string();
                let vocab_pos = v.part_of_speech().ok()?;
                if vocab_pos == *pos { Some(word) } else { None }
            },
            _ => None,
        })
        .collect()
}

pub fn generate_grammar_distractors(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    correct_text: &str,
    count: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let mut distractors = Vec::new();
    let mut seen = HashSet::new();
    seen.insert(correct_text.to_string());

    let max_attempts = count * 10;
    let mut attempts = 0;

    while distractors.len() < count && attempts < max_attempts {
        attempts += 1;

        if let Some(distractor) = apply_mutated_pattern(rules, source_word, pos, rng) {
            if !seen.contains(&distractor) {
                seen.insert(distractor.clone());
                distractors.push(distractor);
            }
        }
    }

    distractors
}

pub fn apply_mutated_pattern(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    rng: &mut impl Rng,
) -> Option<String> {
    let mutable_indices: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, a)| a.group() != FormatActionGroup::Universal)
        .map(|(i, _)| i)
        .collect();

    if mutable_indices.is_empty() {
        return mutate_universal_only(rules, source_word, pos, rng);
    }

    let idx = mutable_indices.choose(rng)?;
    let original_action = &rules[*idx];
    let alternatives = original_action.mutation_alternatives();

    if alternatives.is_empty() {
        return None;
    }

    let alternative = alternatives.choose(rng)?;
    let mut mutated: Vec<FormatAction> = rules.to_vec();
    mutated[*idx] = (*alternative).clone();

    apply_format_actions(source_word, &mutated, pos).ok()
}

/// Generates a distractor for chains consisting solely of Universal actions
/// (AddPostfix / ReplacePostfix / RemovePostfix without any conjugating action).
///
/// These chains cannot be mutated by swapping conjugation (there is none).
/// Instead, a different dictionary-form postfix from
/// [`UNIVERSAL_DISTRACTOR_POSTFIXES`] is appended directly to the **source
/// word**. This preserves the base word intact and always produces valid
/// Japanese, regardless of the original chain type: for `ReplacePostfix` the
/// stem must not be truncated before adding a foreign postfix, and for
/// `RemovePostfix` the source word itself is the only stable base.
///
/// Distinctness between the distractor and the correct answer is guaranteed
/// differently depending on the chain type:
/// - For `AddPostfix` chains, postfixes already present in the chain are
///   excluded from the candidate pool, so the chosen postfix differs from the
///   original one.
/// - For `ReplacePostfix`/`RemovePostfix` chains, `used_postfixes` is empty,
///   but for well-formed rules (the postfix being replaced/removed is a real,
///   non-empty suffix of the source word) the distractor (source word + pool
///   postfix) is structurally distinct from the correct answer (trimmed stem
///   + new postfix), so they cannot coincide.
///
/// In all cases the `seen` HashSet in `generate_grammar_distractors` provides
/// an ultimate deduplication safety net that filters any residual collision.
fn mutate_universal_only(
    rules: &[FormatAction],
    source_word: &str,
    pos: &PartOfSpeech,
    rng: &mut impl Rng,
) -> Option<String> {
    let used_postfixes: HashSet<&str> = rules
        .iter()
        .filter_map(|a| match a {
            FormatAction::AddPostfix { postfix } => Some(postfix.as_str()),
            _ => None,
        })
        .collect();

    let candidates: Vec<&str> = UNIVERSAL_DISTRACTOR_POSTFIXES
        .iter()
        .copied()
        .filter(|p| !used_postfixes.contains(*p))
        .collect();

    let alternative_postfix = candidates.choose(rng).copied()?;
    let distractor_actions = vec![FormatAction::AddPostfix {
        postfix: alternative_postfix.to_string(),
    }];
    apply_format_actions(source_word, &distractor_actions, pos).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_mutated_pattern_returns_distractor_for_universal_only() {
        let actions = vec![FormatAction::AddPostfix {
            postfix: "ため".into(),
        }];
        let result = apply_mutated_pattern(&actions, "行く", &PartOfSpeech::Verb, &mut rand::rng());
        assert!(
            result.is_some(),
            "Universal-only chain should now produce a distractor"
        );
        assert_ne!(
            result.unwrap(),
            "行くため",
            "Distractor must differ from correct answer"
        );
    }

    #[test]
    fn apply_mutated_pattern_universal_only_never_collides_under_stress() {
        // Stress test: each Universal-only chain type is exercised 30 times to
        // confirm the generated distractor never equals the correct answer.
        // AddPostfix relies on the `used_postfixes` exclusion filter;
        // ReplacePostfix/RemovePostfix exercise `mutate_universal_only` with an
        // empty `used_postfixes` set and rely on structural distinctness.
        let mut rng = rand::rng();

        // AddPostfix: correct = source + P_orig, distractor = source + P_pool.
        let add_actions = vec![FormatAction::AddPostfix {
            postfix: "ため".into(),
        }];
        for _ in 0..30 {
            let distractor =
                apply_mutated_pattern(&add_actions, "行く", &PartOfSpeech::Verb, &mut rng)
                    .expect("AddPostfix chain should always produce a distractor");
            assert_ne!(
                distractor, "行くため",
                "AddPostfix distractor must differ from correct answer"
            );
        }

        // ReplacePostfix: correct = source - old + new, distractor = source + P_pool.
        let replace_actions = vec![FormatAction::ReplacePostfix {
            old_postfix: "く".into(),
            new_postfix: "いて".into(),
        }];
        for _ in 0..30 {
            let distractor =
                apply_mutated_pattern(&replace_actions, "行く", &PartOfSpeech::Verb, &mut rng)
                    .expect("ReplacePostfix chain should always produce a distractor");
            assert_ne!(
                distractor, "行いて",
                "ReplacePostfix distractor must differ from correct answer"
            );
        }

        // RemovePostfix: correct = source - postfix, distractor = source + P_pool.
        let remove_actions = vec![FormatAction::RemovePostfix {
            postfix: "く".into(),
        }];
        for _ in 0..30 {
            let distractor =
                apply_mutated_pattern(&remove_actions, "行く", &PartOfSpeech::Verb, &mut rng)
                    .expect("RemovePostfix chain should always produce a distractor");
            assert_ne!(
                distractor, "行",
                "RemovePostfix distractor must differ from correct answer"
            );
        }
    }

    #[test]
    fn apply_mutated_pattern_handles_replace_postfix_universal_only() {
        let actions = vec![FormatAction::ReplacePostfix {
            old_postfix: "く".into(),
            new_postfix: "いて".into(),
        }];
        let result = apply_mutated_pattern(&actions, "行く", &PartOfSpeech::Verb, &mut rand::rng());
        assert!(
            result.is_some(),
            "ReplacePostfix universal-only chain should produce a distractor"
        );
        let distractor = result.unwrap();
        assert_ne!(
            distractor, "行いて",
            "Distractor must differ from correct answer"
        );
        assert!(
            distractor.starts_with("行く"),
            "Distractor must preserve source word stem (valid Japanese): got {distractor}"
        );
    }

    #[test]
    fn apply_mutated_pattern_produces_distractor_for_remove_postfix_only() {
        let actions = vec![FormatAction::RemovePostfix {
            postfix: "く".into(),
        }];
        let result = apply_mutated_pattern(&actions, "行く", &PartOfSpeech::Verb, &mut rand::rng());
        assert!(
            result.is_some(),
            "RemovePostfix-only chain produces a distractor via source-word postfix"
        );
        let distractor = result.unwrap();
        assert_ne!(
            distractor, "行",
            "Distractor must differ from correct answer (source word minus postfix)"
        );
        assert!(
            distractor.starts_with("行く"),
            "Distractor must preserve source word: got {distractor}"
        );
    }

    #[test]
    fn generate_distractors_excludes_correct() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        assert!(distractors.iter().all(|d| d != "行きます"));
    }

    #[test]
    fn generate_distractors_no_duplicates() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        let unique: HashSet<_> = distractors.iter().collect();
        assert_eq!(unique.len(), distractors.len());
    }

    #[test]
    fn generate_distractors_returns_up_to_count() {
        let actions = vec![FormatAction::VerbToMasu {}];
        let distractors = generate_grammar_distractors(
            &actions,
            "行く",
            &PartOfSpeech::Verb,
            "行きます",
            3,
            &mut rand::rng(),
        );
        assert!(distractors.len() <= 3);
    }

    #[test]
    fn find_known_vocab_returns_empty_for_empty_set() {
        let ks = KnowledgeSet::new();
        let result = find_known_vocab_words_for_pos(&ks, &PartOfSpeech::Verb);
        assert!(result.is_empty());
    }

    mod generate_grammar_practice_questions {
        use super::*;
        use crate::dictionary::grammar::{FormatAction, GrammarRule, GrammarRuleContent};
        use crate::domain::knowledge::VocabularyCard;
        use crate::domain::memory::MemoryState;
        use crate::domain::value_objects::{NativeLanguage, Question};
        use crate::domain::{JapaneseLevel, Rating};
        use std::collections::HashMap;
        use ulid::Ulid;

        fn create_verb_rule() -> GrammarRule {
            GrammarRule::new(
                Ulid::new(),
                JapaneseLevel::N5,
                HashMap::from([(
                    NativeLanguage::English,
                    GrammarRuleContent::new(
                        "Test Masu".to_string(),
                        "Test desc".to_string(),
                        "Explanation".to_string(),
                        "How to form".to_string(),
                        "Examples".to_string(),
                        "Nuances".to_string(),
                        "Pro tip".to_string(),
                        None,
                    ),
                )]),
                Some(HashMap::from([(
                    PartOfSpeech::Verb,
                    vec![FormatAction::VerbToMasu {}],
                )])),
            )
        }

        fn create_known_vocab_card(word: &str) -> Card {
            Card::Vocabulary(VocabularyCard::new(
                Question::new(word.to_string()).unwrap(),
            ))
        }

        fn add_known_vocab(ks: &mut KnowledgeSet, word: &str) {
            let mut study_card = ks.create_card(create_known_vocab_card(word)).unwrap();

            let memory = MemoryState::new(
                crate::domain::memory::Stability::new(15.0).unwrap(),
                crate::domain::memory::Difficulty::new(2.0).unwrap(),
                chrono::Utc::now(),
            );
            study_card.apply_review(memory, Rating::Good);
        }

        #[test]
        fn returns_error_when_no_supported_pos() {
            let rule = GrammarRule::new(
                Ulid::new(),
                JapaneseLevel::N5,
                HashMap::from([(
                    NativeLanguage::English,
                    GrammarRuleContent::new(
                        "Empty".to_string(),
                        "No POS".to_string(),
                        "Explanation".to_string(),
                        "How to form".to_string(),
                        "Examples".to_string(),
                        "Nuances".to_string(),
                        "Pro tip".to_string(),
                        None,
                    ),
                )]),
                None,
            );

            let ks = KnowledgeSet::new();
            let result = generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng());

            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                OrigaError::GrammarFormatError { .. }
            ));
        }

        #[test]
        fn returns_empty_for_empty_knowledge_set() {
            crate::use_cases::init_real_dictionaries();

            let rule = create_verb_rule();
            let ks = KnowledgeSet::new();
            let result = generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng());

            assert!(result.unwrap().is_empty());
        }

        #[test]
        fn practice_questions_have_unique_words() {
            crate::use_cases::init_real_dictionaries();

            let rule = create_verb_rule();
            let mut ks = KnowledgeSet::new();

            for word in ["行く", "食べる", "飲む", "読む", "書く"] {
                add_known_vocab(&mut ks, word);
            }

            let questions =
                generate_grammar_practice_questions(&rule, &ks, 5, &mut rand::rng()).unwrap();

            let words: HashSet<&str> = questions.iter().map(|q| q.word_text()).collect();
            assert_eq!(
                words.len(),
                questions.len(),
                "All word texts must be unique"
            );
        }

        #[test]
        fn returns_fewer_when_not_enough_words() {
            crate::use_cases::init_real_dictionaries();

            let rule = create_verb_rule();
            let mut ks = KnowledgeSet::new();

            add_known_vocab(&mut ks, "行く");
            add_known_vocab(&mut ks, "食べる");

            let questions =
                generate_grammar_practice_questions(&rule, &ks, 10, &mut rand::rng()).unwrap();

            assert!(
                questions.len() <= 2,
                "Expected at most 2 questions, got {}",
                questions.len()
            );
        }

        #[test]
        fn correct_answer_is_at_correct_index() {
            crate::use_cases::init_real_dictionaries();

            let rule = create_verb_rule();
            let mut ks = KnowledgeSet::new();

            for word in ["行く", "食べる", "飲む"] {
                add_known_vocab(&mut ks, word);
            }

            let questions =
                generate_grammar_practice_questions(&rule, &ks, 3, &mut rand::rng()).unwrap();

            for q in &questions {
                let correct = &q.options[q.correct_index];
                let expected = rule.format(q.word_text(), &PartOfSpeech::Verb).unwrap();
                assert_eq!(
                    correct, &expected,
                    "Option at correct_index should be the formatted word"
                );
            }
        }

        #[test]
        fn each_question_has_four_options() {
            crate::use_cases::init_real_dictionaries();

            let rule = create_verb_rule();
            let mut ks = KnowledgeSet::new();

            for word in ["行く", "食べる", "飲む", "読む"] {
                add_known_vocab(&mut ks, word);
            }

            let questions =
                generate_grammar_practice_questions(&rule, &ks, 4, &mut rand::rng()).unwrap();

            for q in &questions {
                assert_eq!(
                    q.options.len(),
                    4,
                    "Each question must have exactly 4 options (1 correct + 3 distractors)"
                );
            }
        }
    }
}
