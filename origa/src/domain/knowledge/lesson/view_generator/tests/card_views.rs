use super::*;
use crate::domain::StudyCard;
use crate::domain::knowledge::KanjiCard;
use crate::domain::knowledge::lesson::types::LessonCardView;
use crate::domain::memory::{Difficulty, MemoryState, Rating, Stability};
use crate::use_cases::init_real_dictionaries;
use chrono::Utc;
use rand::{SeedableRng, rngs::StdRng};

use super::super::LessonViewGenerator;

mod grammar_card_view_tests {
    use super::*;

    fn make_ks() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        for w in ["猫", "犬", "鳥"] {
            ks.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(w.to_string()).unwrap(),
            )))
            .unwrap();
        }
        ks
    }

    #[test]
    fn grammar_new_card_always_normal() {
        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").unwrap();
        let sc = StudyCard::new(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)));
        let ks = make_ks();
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(matches!(
                generator.apply_view(&sc, true, &mut rng),
                LessonCardView::Normal(_)
            ));
        }
    }

    #[test]
    fn grammar_review_card_always_normal() {
        let rule_id = ulid::Ulid::from_string("01KJ9AVWBGC2BT0DMFPDYYFEWB").unwrap();
        let sc = StudyCard::new(Card::Grammar(GrammarRuleCard::new_test_with_id(rule_id)));
        let ks = make_ks();
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        for seed in 0u64..50 {
            let mut rng = StdRng::seed_from_u64(seed);
            assert!(matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::Normal(_)
            ));
        }
    }
}

mod kanji_view_tests {
    use super::*;

    fn make_ks() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        for k in ["日", "月", "水", "火", "木"] {
            ks.create_card(Card::Kanji(KanjiCard::new_test(k.to_string())))
                .unwrap();
        }
        ks
    }

    fn make_reviewed_kanji(
        kanji: &str,
        stability: f64,
        difficulty: f64,
        rating: Rating,
    ) -> StudyCard {
        let card = Card::Kanji(KanjiCard::new_test(kanji.to_string()));
        let mut sc = StudyCard::new(card);
        let mem = MemoryState::new(
            Stability::new(stability).unwrap(),
            Difficulty::new(difficulty).unwrap(),
            Utc::now(),
        );
        sc.apply_review(mem, rating);
        sc
    }

    #[test]
    fn new_kanji_produces_normal_quiz_and_writing() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = StudyCard::new(Card::Kanji(KanjiCard::new_test("日".to_string())));
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut counts = std::collections::HashMap::<&str, usize>::new();

        for seed in 0..300 {
            let mut rng = StdRng::seed_from_u64(seed);
            let key = match generator.apply_view(&sc, true, &mut rng) {
                LessonCardView::Normal(_) => "normal",
                LessonCardView::Quiz(_) => "quiz",
                LessonCardView::Writing(_) => "writing",
                other => panic!("Unexpected view for new kanji: {:?}", other),
            };
            *counts.entry(key).or_default() += 1;
        }

        assert!(counts.get("normal").copied().unwrap_or(0) > 0);
        assert!(counts.get("quiz").copied().unwrap_or(0) > 0);
        assert!(counts.get("writing").copied().unwrap_or(0) > 0);
    }

    #[test]
    fn review_kanji_produces_yesno() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        let mut yesno_count = 0;
        for seed in 0..300 {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::YesNo(_)
            ) {
                yesno_count += 1;
            }
        }
        assert!(
            yesno_count > 0,
            "review kanji (not high diff) should get YesNo sometimes"
        );
    }

    #[test]
    fn review_kanji_not_high_difficulty_produces_writing() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = make_reviewed_kanji("日", 5.0, 3.0, Rating::Good);
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);

        let mut writing_count = 0;
        for seed in 0..300 {
            let mut rng = StdRng::seed_from_u64(seed);
            if matches!(
                generator.apply_view(&sc, false, &mut rng),
                LessonCardView::Writing(_)
            ) {
                writing_count += 1;
            }
        }
        assert!(
            writing_count > 0,
            "review kanji should get Writing sometimes"
        );
    }
}

mod new_vocab_view_tests {
    use super::*;

    fn make_ks() -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        for w in ["猫", "犬", "鳥", "魚"] {
            ks.create_card(Card::Vocabulary(VocabularyCard::new(
                Question::new(w.to_string()).unwrap(),
            )))
            .unwrap();
        }
        ks
    }

    #[test]
    fn new_vocab_produces_normal_and_quiz_only() {
        init_real_dictionaries();
        let ks = make_ks();
        let sc = StudyCard::new(create_vocab_card("猫"));
        let mut generator = LessonViewGenerator::new(&ks, NativeLanguage::Russian);
        let mut normal = 0;
        let mut quiz = 0;

        for seed in 0..200 {
            let mut rng = StdRng::seed_from_u64(seed);
            match generator.apply_view(&sc, true, &mut rng) {
                LessonCardView::Normal(_) => normal += 1,
                LessonCardView::Quiz(_) => quiz += 1,
                other => panic!("New vocab should be Normal or Quiz, got {:?}", other),
            }
        }
        assert!(normal > 0, "new vocab should sometimes get Normal");
        assert!(quiz > 0, "new vocab should sometimes get Quiz");
    }
}
