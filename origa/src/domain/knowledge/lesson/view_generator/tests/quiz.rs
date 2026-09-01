use super::*;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::value_objects::NativeLanguage;
use crate::use_cases::init_real_dictionaries;
use ulid::Ulid;

mod grammar_quiz {
    use super::*;

    fn get_first_grammar_rule_id() -> Ulid {
        init_real_dictionaries();
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    #[test]
    fn grammar_card_generates_quiz_with_sufficient_distinct_cards() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let rule_ids = vec![
            "01G00000000000000004000000",
            "01G00000000000000008000000",
            "01G0000000000000000C000000",
        ];
        let other_cards: Vec<Card> = rule_ids
            .into_iter()
            .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
            .collect();

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card, &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert!(quiz.options().iter().any(|o| o.is_correct()));
            },
            _ => panic!("Expected Quiz view for grammar card with sufficient distractors"),
        }
    }

    #[test]
    fn grammar_card_returns_normal_with_insufficient_distinct_cards() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let other_cards: Vec<Card> = vec![];

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card.clone(), &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Normal(card) => {
                assert_eq!(card, grammar_card);
            },
            _ => panic!("Expected Normal view for grammar card with insufficient distractors"),
        }
    }

    #[test]
    fn grammar_quiz_options_contain_correct_answer() {
        init_real_dictionaries();

        let grammar_rule_id = get_first_grammar_rule_id();
        let grammar_card = create_grammar_card(grammar_rule_id);

        let rule_ids = vec![
            "01G00000000000000004000000",
            "01G00000000000000008000000",
            "01G0000000000000000C000000",
        ];
        let other_cards: Vec<Card> = rule_ids
            .into_iter()
            .map(|id| create_grammar_card(Ulid::from_string(id).expect("Invalid ULID")))
            .collect();

        let lang = NativeLanguage::Russian;

        let result = generation::generate_quiz(grammar_card.clone(), &other_cards, &lang);

        assert!(result.is_ok());

        match result.unwrap() {
            LessonCardView::Quiz(quiz) => {
                let correct_answer = grammar_card.answer(&lang).unwrap();
                let correct_text = correct_answer.text_projection();
                assert!(
                    quiz.options().iter().any(|o| o.text() == correct_text),
                    "Quiz options should contain the correct answer"
                );
            },
            _ => panic!("Expected Quiz view"),
        }
    }
}

mod generate_grammar_quiz_tests {
    use super::*;
    use crate::domain::RateMode;

    fn get_verb_rule_id() -> Ulid {
        init_real_dictionaries();
        Ulid::from_string("01G00000000000000024000000").expect("Invalid ULID")
    }

    fn get_non_format_map_rule_id() -> Ulid {
        init_real_dictionaries();
        Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").expect("Invalid ULID")
    }

    fn create_known_vocab_set(word: &str) -> crate::domain::knowledge::KnowledgeSet {
        let mut ks = crate::domain::knowledge::KnowledgeSet::new();
        let card = Card::Vocabulary(crate::domain::VocabularyCard::new(
            crate::domain::value_objects::Question::new(word.to_string()).unwrap(),
        ));
        let study_card = ks.create_card(card).unwrap();
        let id = *study_card.card_id();
        ks.rate_card(id, Rating::Easy, RateMode::StandardLesson)
            .unwrap();
        ks.rate_card(id, Rating::Easy, RateMode::StandardLesson)
            .unwrap();
        ks
    }

    #[test]
    fn grammar_quiz_falls_back_to_normal_without_format_map() {
        init_real_dictionaries();

        let rule_id = get_non_format_map_rule_id();
        let grammar_card = create_grammar_card(rule_id);

        let ks = crate::domain::knowledge::KnowledgeSet::new();
        let result =
            generation::generate_grammar_quiz(grammar_card.clone(), &ks, &NativeLanguage::Russian);

        match result.unwrap() {
            LessonCardView::Normal(card) => assert_eq!(card, grammar_card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }

    #[test]
    fn grammar_quiz_generates_quiz_with_format_map() {
        init_real_dictionaries();

        let rule_id = get_verb_rule_id();
        let grammar_card = create_grammar_card(rule_id);

        let ks = create_known_vocab_set("食べる");

        let result = generation::generate_grammar_quiz(grammar_card, &ks, &NativeLanguage::Russian);
        let view = result.expect("should succeed");

        match &view {
            LessonCardView::GrammarQuiz(gq) => {
                assert_eq!(gq.quiz().options().len(), 4);
                assert_eq!(
                    gq.quiz()
                        .options()
                        .iter()
                        .filter(|o| o.is_correct())
                        .count(),
                    1
                );
                assert!(!gq.word_text().is_empty());
                assert!(gq.grammar_info().rule_id().is_some());
            },
            other => panic!("Expected GrammarQuiz, got {:?}", other),
        }
    }

    #[test]
    fn grammar_quiz_falls_back_without_known_vocab() {
        init_real_dictionaries();

        let rule_id = get_verb_rule_id();
        let grammar_card = create_grammar_card(rule_id);

        let ks = crate::domain::knowledge::KnowledgeSet::new();
        let result =
            generation::generate_grammar_quiz(grammar_card.clone(), &ks, &NativeLanguage::Russian);

        match result.unwrap() {
            LessonCardView::Normal(card) => assert_eq!(card, grammar_card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }
}

mod generate_quiz_vocab_kanji_tests {
    use super::*;

    #[test]
    fn generate_quiz_vocab_with_distinct_answers() {
        init_real_dictionaries();
        let words = ["猫", "犬", "鳥", "魚"];
        let cards: Vec<Card> = words.iter().map(|w| create_vocab_card(w)).collect();

        let result =
            generation::generate_quiz(cards[0].clone(), &cards[1..], &NativeLanguage::Russian);

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_vocab_fallback_no_cards() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let result = generation::generate_quiz(card.clone(), &[], &NativeLanguage::Russian);
        match result.unwrap() {
            LessonCardView::Normal(c) => assert_eq!(c, card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_vocab_fallback_insufficient_distinct() {
        init_real_dictionaries();
        let card = create_vocab_card("猫");
        let same = vec![card.clone()];
        let result = generation::generate_quiz(card.clone(), &same, &NativeLanguage::Russian);
        match result.unwrap() {
            LessonCardView::Normal(c) => assert_eq!(c, card),
            other => panic!("Expected Normal, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_kanji_with_distinct_answers() {
        init_real_dictionaries();
        let kanji_cards: Vec<Card> = ["日", "月", "水", "火"]
            .iter()
            .map(|k| Card::Kanji(KanjiCard::new_test(k.to_string())))
            .collect();

        let result = generation::generate_quiz(
            kanji_cards[0].clone(),
            &kanji_cards[1..],
            &NativeLanguage::Russian,
        );

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz for kanji, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_uses_english_translations_for_english_locale() {
        init_real_dictionaries();
        let words = ["猫", "犬", "鳥", "魚"];
        let cards: Vec<Card> = words.iter().map(|w| create_vocab_card(w)).collect();

        let english_text = answer_text(&cards[0], NativeLanguage::English);
        let russian_text = answer_text(&cards[0], NativeLanguage::Russian);
        assert_ne!(
            english_text, russian_text,
            "test fixture: EN and RU translations must differ for {:?}",
            words[0]
        );

        let result =
            generation::generate_quiz(cards[0].clone(), &cards[1..], &NativeLanguage::English);

        match result.expect("English-locale quiz must be generated") {
            LessonCardView::Quiz(quiz) => {
                let correct = quiz
                    .options()
                    .iter()
                    .find(|o| o.is_correct())
                    .expect("quiz must have a correct option");
                assert_eq!(
                    correct.text(),
                    english_text,
                    "correct option must be the ENGLISH translation"
                );
                assert_ne!(
                    correct.text(),
                    russian_text,
                    "regression guard: option must not be the Russian translation"
                );
            },
            other => panic!("Expected Quiz, got {:?}", other),
        }
    }
}

mod generate_quiz_pos_filter_tests {
    use super::*;
    use crate::domain::knowledge::lesson::types::LessonCardView;
    use crate::domain::value_objects::NativeLanguage;
    use crate::use_cases::init_real_dictionaries;
    use std::collections::HashSet;

    #[test]
    fn generate_quiz_prefers_same_pos_distractors() {
        init_real_dictionaries();

        let verbs: Vec<Card> = ["食べる", "飲む", "読む", "書く"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let nouns: Vec<Card> = ["猫", "犬", "鳥"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let mut pool: Vec<Card> = verbs[1..].to_vec();
        pool.extend(nouns.clone());

        let verb_answers: HashSet<String> = verbs[1..]
            .iter()
            .filter_map(|c| c.answer(&NativeLanguage::Russian).ok())
            .map(|a| a.text_projection())
            .collect();

        let noun_answers: HashSet<String> = nouns
            .iter()
            .filter_map(|c| c.answer(&NativeLanguage::Russian).ok())
            .map(|a| a.text_projection())
            .collect();

        for _ in 0..20 {
            let result =
                generation::generate_quiz(verbs[0].clone(), &pool, &NativeLanguage::Russian);

            match result.expect("should succeed") {
                LessonCardView::Quiz(quiz) => {
                    for option in quiz.options().iter().filter(|o| !o.is_correct()) {
                        assert!(
                            verb_answers.contains(option.text()),
                            "Distractor '{}' should be a verb translation, got noun",
                            option.text()
                        );
                        assert!(
                            !noun_answers.contains(option.text()),
                            "Distractor '{}' should not be a noun translation",
                            option.text()
                        );
                    }
                },
                other => panic!("Expected Quiz, got {:?}", other),
            }
        }
    }

    #[test]
    fn generate_quiz_falls_back_to_other_pos() {
        init_real_dictionaries();

        let verb = create_vocab_card("食べる");
        let nouns: Vec<Card> = ["猫", "犬", "鳥"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let result = generation::generate_quiz(verb, &nouns, &NativeLanguage::Russian);

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz with fallback, got {:?}", other),
        }
    }

    #[test]
    fn generate_quiz_all_same_pos() {
        init_real_dictionaries();

        let verbs: Vec<Card> = ["食べる", "飲む", "読む", "書く"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let result =
            generation::generate_quiz(verbs[0].clone(), &verbs[1..], &NativeLanguage::Russian);

        match result.expect("should succeed") {
            LessonCardView::Quiz(quiz) => {
                assert_eq!(quiz.options().len(), 4);
                assert_eq!(quiz.options().iter().filter(|o| o.is_correct()).count(), 1);
            },
            other => panic!("Expected Quiz, got {:?}", other),
        }
    }

    #[test]
    fn generate_yesno_prefers_same_pos_distractor() {
        init_real_dictionaries();
        use rand::{SeedableRng, rngs::StdRng};

        let verbs: Vec<Card> = ["食べる", "飲む", "読む"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let nouns: Vec<Card> = ["猫", "犬", "鳥"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let mut pool: Vec<Card> = verbs[1..].to_vec();
        pool.extend(nouns.clone());

        let noun_answers: HashSet<String> = nouns
            .iter()
            .filter_map(|c| c.answer(&NativeLanguage::Russian).ok())
            .map(|a| a.text_projection())
            .collect();

        let mut noun_used_count = 0;
        let iterations = 100;

        for seed in 0..iterations {
            let mut rng = StdRng::seed_from_u64(seed);
            let result = generation::generate_yesno(
                verbs[0].clone(),
                &pool,
                &NativeLanguage::Russian,
                &mut rng,
            );

            if let Ok(LessonCardView::YesNo(yn)) = result {
                if !yn.is_correct() {
                    let uses_noun = noun_answers.iter().any(|na| yn.statement().contains(na));
                    if uses_noun {
                        noun_used_count += 1;
                    }
                }
            }
        }

        assert!(
            noun_used_count == 0,
            "No noun distractors when same-POS verbs available, got {}",
            noun_used_count
        );
    }

    #[test]
    fn generate_yesno_falls_back_to_other_pos() {
        init_real_dictionaries();
        use rand::{SeedableRng, rngs::StdRng};

        let verb = create_vocab_card("食べる");
        let nouns: Vec<Card> = ["猫", "犬", "鳥"]
            .iter()
            .map(|w| create_vocab_card(w))
            .collect();

        let mut rng = StdRng::seed_from_u64(42);
        let result = generation::generate_yesno(verb, &nouns, &NativeLanguage::Russian, &mut rng);

        match result.expect("should succeed") {
            LessonCardView::YesNo(yn) => {
                assert!(!yn.statement().is_empty());
            },
            other => panic!("Expected YesNo with fallback, got {:?}", other),
        }
    }
}
