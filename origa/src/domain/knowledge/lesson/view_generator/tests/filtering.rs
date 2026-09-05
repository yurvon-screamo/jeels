use super::*;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::memory::Rating;
use crate::use_cases::init_real_dictionaries;
use rand::{SeedableRng, rngs::StdRng};

use super::super::LessonViewGenerator;

mod yesno_view_filtering {
    use super::*;

    fn create_high_difficulty_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 3.0, 7.0, Rating::Hard);
        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_in_progress_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 5.0, 3.0, Rating::Good);
        assert!(study_card.memory().is_in_progress());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        study_card
    }

    const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
    const ITERATIONS: u64 = 500;

    fn count_yesno_views(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize) {
        let mut generator = LessonViewGenerator::new(ks, NativeLanguage::Russian);
        let mut yesno_count = 0;
        let mut other_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

            match view {
                LessonCardView::YesNo(_) => yesno_count += 1,
                _ => other_count += 1,
            }
        }

        (yesno_count, other_count)
    }

    #[test]
    fn high_difficulty_card_never_gets_yesno_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card("猫");

        let (yesno, _other) = count_yesno_views(&study_card, &ks);

        assert_eq!(
            yesno, 0,
            "high_difficulty card should never get YesNo view, got {yesno} YesNo out of {ITERATIONS} iterations"
        );
    }

    #[test]
    fn in_progress_card_can_get_yesno_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_in_progress_card("猫");

        let (yesno, _other) = count_yesno_views(&study_card, &ks);

        assert!(
            yesno > 0,
            "in_progress card should be able to get YesNo view, got 0 YesNo out of {ITERATIONS} iterations"
        );
    }
}

mod reversed_view_filtering {
    use super::*;

    fn create_high_difficulty_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 3.0, 7.0, Rating::Hard);
        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_known_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 22.0, 2.0, Rating::Easy);
        assert!(study_card.memory().is_known_card());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_in_progress_card(word: &str) -> StudyCard {
        let study_card = create_study_card_with_memory(word, 5.0, 3.0, Rating::Good);
        assert!(study_card.memory().is_in_progress());
        assert!(!study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        study_card
    }

    const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
    const ITERATIONS: u64 = 500;

    fn count_view_types(study_card: &StudyCard, ks: &KnowledgeSet) -> (usize, usize, usize) {
        let mut generator = LessonViewGenerator::new(ks, NativeLanguage::Russian);
        let mut reversed_count = 0;
        let mut grammar_mutated_count = 0;
        let mut normal_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rng);

            match view {
                LessonCardView::Reversed(_) => reversed_count += 1,
                LessonCardView::GrammarMutated { .. } => grammar_mutated_count += 1,
                LessonCardView::Normal(_) => normal_count += 1,
                _ => {},
            }
        }

        (reversed_count, grammar_mutated_count, normal_count)
    }

    #[test]
    fn high_difficulty_card_never_gets_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert_eq!(
            reversed, 0,
            "high_difficulty card should never get Reversed view"
        );
        assert_eq!(
            grammar_mutated, 0,
            "high_difficulty card should never get GrammarMutated view"
        );
        assert!(
            normal > 0,
            "high_difficulty card should get Normal view as fallback in advanced range"
        );
    }

    #[test]
    fn known_card_can_get_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_known_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert!(
            reversed > 0 || grammar_mutated > 0,
            "known card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
        );
    }

    #[test]
    fn in_progress_card_can_get_reversed_view() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_in_progress_card("猫");

        let (reversed, grammar_mutated, normal) = count_view_types(&study_card, &ks);

        assert!(
            reversed > 0 || grammar_mutated > 0,
            "in_progress card should get Reversed or GrammarMutated views, got {reversed} reversed, {grammar_mutated} grammar_mutated, {normal} normal"
        );
    }

    #[test]
    fn review_vocab_never_mutates_by_new_grammar_rule() {
        init_real_dictionaries();

        // Arrange: единственная применимая грамматика — глагольное
        // правило, ещё НОВОЕ (не прошедшее руку знакомства)
        let mut ks = KnowledgeSet::new();
        for word in DISTRACTOR_WORDS {
            ks.create_card(create_vocab_card(word)).unwrap();
        }
        let rule_id = ulid::Ulid::from_string("01G00000000000000024000000").expect("valid ULID");
        let grammar_sc = ks.create_card(create_grammar_card(rule_id)).unwrap();
        assert!(grammar_sc.is_new(), "fixture: правило должно быть новым");

        // Ревью-слово 食べる (глагол): правило применимо к его POS —
        // без фильтра мутация брала бы новое правило
        let vocab = VocabularyCard::from_known_word("食べる", &NativeLanguage::Russian)
            .expect("fixture: 食べる есть в словаре");
        let mut study_card = StudyCard::new(Card::Vocabulary(vocab));
        study_card.apply_review(
            MemoryState::new(
                Stability::new(5.0).unwrap(),
                Difficulty::new(3.0).unwrap(),
                Utc::now(),
            ),
            Rating::Good,
        );

        // Act
        let (_reversed, grammar_mutated, _normal) = count_view_types(&study_card, &ks);

        // Assert: новое правило не участвует в мутациях — слово не
        // уводится из «новых» рейтингом мутированной карты
        assert_eq!(
            grammar_mutated, 0,
            "новая грамматика не должна попадать в known_grammars"
        );

        // Позитивный контроль: то же правило после изучения — мутация
        // работает (фикстура остаётся осмысленной при дрейфе словаря)
        ks.mark_card_as_known(*grammar_sc.card_id()).unwrap();
        let (_reversed, grammar_mutated_known, _normal) = count_view_types(&study_card, &ks);
        assert!(
            grammar_mutated_known > 0,
            "изученное правило должно давать GrammarMutated для ревью-глагола"
        );
    }
}

mod reversed_view_easy_reviews {
    use super::*;

    fn create_high_difficulty_card_with_easy_reviews(word: &str, easy_count: usize) -> StudyCard {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ));
        let mut study_card = StudyCard::new(card);

        for _ in 0..easy_count {
            let memory = MemoryState::new(
                Stability::new(3.0).unwrap(),
                Difficulty::new(7.0).unwrap(),
                Utc::now(),
            );
            study_card.apply_review(memory, Rating::Easy);
        }

        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    fn create_high_difficulty_card_with_good_reviews(word: &str, good_count: usize) -> StudyCard {
        let card = Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ));
        let mut study_card = StudyCard::new(card);

        for _ in 0..good_count {
            let memory = MemoryState::new(
                Stability::new(3.0).unwrap(),
                Difficulty::new(7.0).unwrap(),
                Utc::now(),
            );
            study_card.apply_review(memory, Rating::Good);
        }

        assert!(study_card.memory().is_high_difficulty());
        assert!(!study_card.memory().is_known_card());
        assert!(!study_card.memory().is_in_progress());
        study_card
    }

    const DISTRACTOR_WORDS: &[&str] = &["猫", "犬", "鳥", "魚", "馬", "牛"];
    const ITERATIONS: u64 = 500;

    #[test]
    fn high_difficulty_with_many_easy_reviews_gets_reversed() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_easy_reviews("猫", 3);

        assert_eq!(study_card.memory().easy_review_count(), 3);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut reversed_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&study_card, false, &mut rng),
                LessonCardView::Reversed(_)
            ) {
                reversed_count += 1;
            }
        }

        assert!(
            reversed_count > 0,
            "high-difficulty card with 3 Easy reviews should get Reversed view, got {reversed_count} out of {ITERATIONS}"
        );
    }

    #[test]
    fn high_difficulty_with_few_easy_reviews_no_reversed() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_easy_reviews("猫", 2);

        assert_eq!(study_card.memory().easy_review_count(), 2);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(&study_card, false, &mut rng);
            assert!(
                !matches!(view, LessonCardView::Reversed(_)),
                "high-difficulty card with only 2 Easy reviews should not get Reversed view"
            );
        }
    }

    #[test]
    fn high_difficulty_with_many_easy_reviews_no_grammar_mutated() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_easy_reviews("猫", 3);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(&study_card, false, &mut rng);
            assert!(
                !matches!(view, LessonCardView::GrammarMutated { .. }),
                "high-difficulty card should never get GrammarMutated even with Easy reviews"
            );
        }
    }

    #[test]
    fn high_difficulty_with_enough_good_reviews_gets_reversed() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_good_reviews("猫", 4);

        assert_eq!(study_card.memory().good_review_count(), 4);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut reversed_count = 0;

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&study_card, false, &mut rng),
                LessonCardView::Reversed(_)
            ) {
                reversed_count += 1;
            }
        }

        assert!(
            reversed_count > 0,
            "high-difficulty card with 4 Good reviews should get Reversed view, got {reversed_count} out of {ITERATIONS}"
        );
    }

    #[test]
    fn high_difficulty_with_few_good_reviews_no_reversed() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_good_reviews("猫", 3);

        assert_eq!(study_card.memory().good_review_count(), 3);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(&study_card, false, &mut rng);
            assert!(
                !matches!(view, LessonCardView::Reversed(_)),
                "high-difficulty card with only 3 Good reviews should not get Reversed view"
            );
        }
    }

    #[test]
    fn high_difficulty_with_enough_good_reviews_no_grammar_mutated() {
        init_real_dictionaries();

        let ks = create_knowledge_set_with_vocab(DISTRACTOR_WORDS);
        let study_card = create_high_difficulty_card_with_good_reviews("猫", 4);

        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        for seed in 0..ITERATIONS {
            let mut rng = StdRng::seed_from_u64(seed);
            let view = generator.apply_view(&study_card, false, &mut rng);
            assert!(
                !matches!(view, LessonCardView::GrammarMutated { .. }),
                "high-difficulty card should never get GrammarMutated even with Good reviews"
            );
        }
    }
}
