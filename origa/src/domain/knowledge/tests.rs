use super::lesson_builder::MAX_LESSON_SIZE;
use super::*;
use crate::domain::DailyBudget;
use crate::domain::JapaneseLevel;
use crate::domain::JlptContent;
use crate::domain::NativeLanguage;
use crate::domain::memory::{KNOWN_CARD_STABILITY_THRESHOLD, MemoryState};
use crate::domain::value_objects::Question;

fn create_vocab_card(word: &str) -> Card {
    Card::Vocabulary(VocabularyCard::new(
        Question::new(word.to_string()).unwrap(),
    ))
}

fn create_known_memory_state() -> MemoryState {
    MemoryState::new(
        crate::domain::memory::Stability::new(KNOWN_CARD_STABILITY_THRESHOLD + 1.0).unwrap(),
        crate::domain::memory::Difficulty::new(2.0).unwrap(),
        chrono::Utc::now(),
    )
}

// --- Phrase index fixture ---
//
// The phrase index is a process-wide `OnceLock`: only one index can live in a
// test binary. This loads the same 8-phrase fixture used by `lesson_builder`,
// `journeys/phrase` and `seed_ready_phrases`, so the data is identical no matter
// which module wins the initialization race. Tests that assert on phrase
// selection MUST call this so their assertions exercise the real selection path
// rather than silently matching zero phrases (the failure mode flagged in C1).

static PHRASE_INDEX_INIT: std::sync::Once = std::sync::Once::new();

fn ensure_test_phrase_index() {
    PHRASE_INDEX_INIT.call_once(|| {
        if crate::dictionary::phrase::is_phrases_loaded() {
            return;
        }
        let index_json = r#"{"v":1,"h":"test","total":8,"phrases":[
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HJ","t":["test","hello"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HK","t":["test","bye"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB"]},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HN","t":["test","morning"],"c":0,"g":["01KJ9AVWBGC2BT0DMFPDYYFEWB","01G00000000000000024000000"]},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HM","t":["test","thanks"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HP","t":["test","は"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HQ","t":["hello","extra1"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HR","t":["hello","extra2"],"c":0},
            {"i":"01KPJ5S3N1DRFFD236Z4EZ03HS","t":["alpha","beta"],"c":0}
        ]}"#;
        crate::dictionary::phrase::init_phrase_index(index_json)
            .expect("Failed to init test phrase index");
    });
}

fn phrase_id_hello() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HJ").expect("valid ULID")
}

fn phrase_id_bye() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HK").expect("valid ULID")
}

fn phrase_id_morning() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HN").expect("valid ULID")
}

fn phrase_id_thanks() -> Ulid {
    Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HM").expect("valid ULID")
}

/// Counts phrase study cards present in a lesson, split by core (interleaved)
/// vs tail. Core phrases sit at indices `< core_count`; tail phrases follow.
fn count_phrases(result: &LessonData) -> (usize, usize) {
    let mut core_phrases = 0usize;
    let mut tail_phrases = 0usize;
    for (index, (_, lc)) in result.cards.iter().enumerate() {
        if matches!(lc.card(), Card::Phrase(_)) {
            if index < result.core_count {
                core_phrases += 1;
            } else {
                tail_phrases += 1;
            }
        }
    }
    (core_phrases, tail_phrases)
}

#[test]
fn cards_to_lesson_includes_favorite_cards() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let study_card = knowledge_set.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    knowledge_set.toggle_favorite(card_id).unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );
    assert!(result.contains_key(&card_id));
}

#[test]
fn cards_to_lesson_includes_high_difficulty_cards() {
    let mut knowledge_set = KnowledgeSet::new();

    let card1 = create_vocab_card("猫");
    let card2 = create_vocab_card("犬");

    let study1 = knowledge_set.create_card(card1).unwrap();
    let study2 = knowledge_set.create_card(card2).unwrap();

    knowledge_set
        .rate_card(*study1.card_id(), Rating::Again, RateMode::ShortTerm)
        .unwrap();

    knowledge_set
        .rate_card(*study2.card_id(), Rating::Easy, RateMode::StandardLesson)
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(result.contains_key(study1.card_id()));
}

#[test]
fn handle_favorite_rating_easy_increases_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    assert_eq!(study_card.favorite_easy_streak(), 0);
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);
}

#[test]
fn handle_favorite_rating_good_does_not_change_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.favorite_easy_streak(), 1);
}

#[test]
fn handle_favorite_rating_hard_resets_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);

    study_card.handle_favorite_rating(Rating::Hard);
    assert_eq!(study_card.favorite_easy_streak(), 0);
}

#[test]
fn handle_favorite_rating_again_resets_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);

    study_card.handle_favorite_rating(Rating::Again);
    assert_eq!(study_card.favorite_easy_streak(), 0);
}

#[test]
fn handle_favorite_rating_five_easy_removes_favorite() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    assert!(study_card.is_favorite());

    for _ in 0..4 {
        study_card.handle_favorite_rating(Rating::Easy);
        assert!(study_card.is_favorite());
    }

    study_card.handle_favorite_rating(Rating::Easy);
    assert!(!study_card.is_favorite());
    assert_eq!(study_card.favorite_easy_streak(), 0);
}

#[test]
fn handle_favorite_rating_good_does_not_interrupt_accumulation() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);
    study_card.toggle_favorite();

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.favorite_easy_streak(), 1);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 2);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.favorite_easy_streak(), 2);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 3);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.favorite_easy_streak(), 3);

    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 4);

    study_card.handle_favorite_rating(Rating::Good);
    assert_eq!(study_card.favorite_easy_streak(), 4);

    study_card.handle_favorite_rating(Rating::Easy);
    assert!(!study_card.is_favorite());
}

#[test]
fn handle_favorite_rating_non_favorited_does_nothing() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = create_known_memory_state();
    study_card.apply_review(memory, Rating::Good);

    assert!(!study_card.is_favorite());

    let initial_streak = study_card.favorite_easy_streak();
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), initial_streak);
}

#[test]
fn handle_favorite_rating_new_card_increments_streak() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    study_card.toggle_favorite();

    assert_eq!(study_card.favorite_easy_streak(), 0);
    study_card.handle_favorite_rating(Rating::Easy);
    assert_eq!(study_card.favorite_easy_streak(), 1);
}

#[test]
fn handle_favorite_rating_new_card_auto_unfavorite_after_five_easy() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    assert!(study_card.is_new());
    study_card.toggle_favorite();
    assert!(study_card.is_favorite());

    for _ in 0..5 {
        study_card.handle_favorite_rating(Rating::Easy);
    }

    assert!(!study_card.is_favorite());
    assert_eq!(study_card.favorite_easy_streak(), 0);
}

#[test]
fn handle_favorite_rating_high_difficulty_auto_unfavorite_after_five_easy() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let mut study_card = knowledge_set.create_card(card).unwrap();

    let memory = MemoryState::new(
        crate::domain::memory::Stability::new(5.0).unwrap(),
        crate::domain::memory::Difficulty::new(7.0).unwrap(),
        chrono::Utc::now(),
    );
    study_card.apply_review(memory, Rating::Good);

    study_card.toggle_favorite();
    assert!(study_card.is_favorite());

    for _ in 0..5 {
        study_card.handle_favorite_rating(Rating::Easy);
    }

    assert!(!study_card.is_favorite());
    assert_eq!(study_card.favorite_easy_streak(), 0);
}

#[test]
fn create_card_updates_daily_stats() {
    let mut knowledge_set = KnowledgeSet::new();

    assert!(knowledge_set.lesson_history().is_empty());

    let card1 = create_vocab_card("猫");
    knowledge_set.create_card(card1).unwrap();

    assert_eq!(knowledge_set.lesson_history().len(), 1);
    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.total_words(), 1);
    assert_eq!(history_item.new_words(), 1);
    assert_eq!(history_item.known_words(), 0);
    assert_eq!(history_item.lessons_completed(), 0);

    let card2 = create_vocab_card("犬");
    knowledge_set.create_card(card2).unwrap();

    assert_eq!(knowledge_set.lesson_history().len(), 1);
    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.total_words(), 2);
    assert_eq!(history_item.new_words(), 2);
    assert_eq!(history_item.lessons_completed(), 0);
}

#[test]
fn merge_respects_tombstones() {
    let mut local = KnowledgeSet::new();
    local.create_card(create_vocab_card("猫")).unwrap();
    let study2 = local.create_card(create_vocab_card("犬")).unwrap();
    local.create_card(create_vocab_card("鳥")).unwrap();

    let remote = local.clone();

    let card2_id = *study2.card_id();
    local.delete_card(card2_id).unwrap();

    assert_eq!(local.study_cards().len(), 2);
    assert!(local.deleted_cards().contains(&card2_id));

    local.merge(&remote);

    assert_eq!(
        local.study_cards().len(),
        2,
        "card2 не должна восстановиться"
    );
    assert!(
        local.deleted_cards().contains(&card2_id),
        "tombstone должен сохраниться"
    );
}

#[test]
fn rate_card_increments_lessons_completed() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let study_card = knowledge_set.create_card(card).unwrap();

    knowledge_set
        .rate_card(
            *study_card.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();

    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(history_item.lessons_completed(), 1);
}

#[test]
fn merge_study_cards_updates_existing() {
    let mut local = KnowledgeSet::new();
    let study_card = local.create_card(create_vocab_card("猫")).unwrap();
    let card_id = *study_card.card_id();

    assert!(
        local.get_card(card_id).unwrap().is_new(),
        "карточка должна быть новой до merge"
    );

    let mut remote = local.clone();
    remote
        .rate_card(card_id, Rating::Good, RateMode::StandardLesson)
        .unwrap();

    local.merge(&remote);

    let merged_card = local.get_card(card_id).unwrap();
    assert!(
        !merged_card.is_new(),
        "карточка не должна быть новой после merge"
    );
}

#[test]
fn merge_lessons_completed_takes_max() {
    let mut local = KnowledgeSet::new();
    let card1 = local.create_card(create_vocab_card("猫")).unwrap();
    local
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    local
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    let history_item = &local.lesson_history()[0];
    assert_eq!(history_item.lessons_completed(), 2);

    let mut remote = KnowledgeSet::new();
    let card2 = remote.create_card(create_vocab_card("犬")).unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    remote
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    let remote_history_item = &remote.lesson_history()[0];
    assert_eq!(remote_history_item.lessons_completed(), 3);

    local.merge(&remote);

    let merged_history = &local.lesson_history()[0];
    assert_eq!(
        merged_history.lessons_completed(),
        3,
        "lessons_completed должен использовать max для идемпотентности"
    );
}

#[test]
fn merge_stats_recalculated_from_actual() {
    let mut local = KnowledgeSet::new();
    for i in 0..100 {
        local
            .create_card(create_vocab_card(&format!("word{i}")))
            .unwrap();
    }

    let mut remote = local.clone();

    for i in 0..50 {
        local
            .create_card(create_vocab_card(&format!("known{i}")))
            .unwrap();
    }

    for i in 0..150 {
        remote
            .create_card(create_vocab_card(&format!("remote{i}")))
            .unwrap();
    }

    local.merge(&remote);

    let history_item = &local.lesson_history()[0];
    assert_eq!(history_item.total_words(), 300);
}

#[test]
fn recalculate_daily_stats_preserves_new_cards_on_create_card() {
    let mut knowledge_set = KnowledgeSet::new();

    let mut studied_ids = Vec::new();
    for i in 0..5 {
        let card = create_vocab_card(&format!("word{i}"));
        let study_card = knowledge_set.create_card(card).unwrap();
        studied_ids.push(*study_card.card_id());
    }

    for id in studied_ids {
        knowledge_set
            .rate_card(id, Rating::Good, RateMode::StandardLesson)
            .unwrap();
    }

    assert_eq!(knowledge_set.new_cards_studied_today(), 5);

    knowledge_set
        .create_card(create_vocab_card("extra"))
        .unwrap();

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        5,
        "new_cards_studied_today should be preserved after create_card"
    );

    for i in 0..10 {
        knowledge_set
            .create_card(create_vocab_card(&format!("new{i}")))
            .unwrap();
    }

    let lesson_cards = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );
    let new_in_lesson = lesson_cards
        .iter()
        .filter(|(id, _)| knowledge_set.get_card(**id).unwrap().memory().is_new())
        .count();
    assert!(
        new_in_lesson <= 5,
        "Expected at most 5 new cards in lesson, got {new_in_lesson}"
    );
}

#[test]
fn recalculate_daily_stats_preserves_new_cards_on_delete_card() {
    let mut knowledge_set = KnowledgeSet::new();

    let card1 = knowledge_set.create_card(create_vocab_card("a")).unwrap();
    let card2 = knowledge_set.create_card(create_vocab_card("b")).unwrap();
    knowledge_set.create_card(create_vocab_card("c")).unwrap();

    knowledge_set
        .rate_card(*card1.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();
    knowledge_set
        .rate_card(*card2.card_id(), Rating::Good, RateMode::StandardLesson)
        .unwrap();

    assert_eq!(knowledge_set.new_cards_studied_today(), 2);

    knowledge_set.delete_card(*card1.card_id()).unwrap();

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        2,
        "new_cards_studied_today should be preserved after delete_card"
    );
}

#[test]
fn new_cards_sorted_by_jlpt_level() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["走る".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N3,
        ["挑戦する".to_string()].into_iter().collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_chousen = knowledge_set
        .create_card(create_vocab_card("挑戦する"))
        .unwrap();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();
    let study_hashiru = knowledge_set
        .create_card(create_vocab_card("走る"))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(2),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (N5) should be selected — highest JLPT priority"
    );
    assert!(
        result.contains_key(study_hashiru.card_id()),
        "走る (N4) should be selected — second highest JLPT priority"
    );
    assert!(
        !result.contains_key(study_chousen.card_id()),
        "挑戦する (N3) should not be selected — daily limit reached"
    );
}

#[test]
fn new_cards_unknown_level_go_last() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_michigo = knowledge_set
        .create_card(create_vocab_card("未知語"))
        .unwrap();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(1),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (N5) should be selected — known JLPT level"
    );
    assert!(
        !result.contains_key(study_michigo.card_id()),
        "未知語 (Unknown) should not be selected — unknown level has lowest priority"
    );
}

#[test]
fn new_cards_jlpt_sort_does_not_affect_other_categories() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["食べる".to_string()].into_iter().collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["走る".to_string()].into_iter().collect(),
    );
    jlpt_content
        .kanji_by_level
        .insert(JapaneseLevel::N5, ["日".to_string()].into_iter().collect());

    let mut knowledge_set = KnowledgeSet::new();
    let study_taberu = knowledge_set
        .create_card(create_vocab_card("食べる"))
        .unwrap();
    let study_hashiru = knowledge_set
        .create_card(create_vocab_card("走る"))
        .unwrap();
    let study_nichi = knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("日".to_string())))
        .unwrap();

    knowledge_set
        .rate_card(*study_taberu.card_id(), Rating::Again, RateMode::ShortTerm)
        .unwrap();
    knowledge_set
        .toggle_favorite(*study_nichi.card_id())
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.contains_key(study_taberu.card_id()),
        "食べる (due, high difficulty) should be in lesson"
    );
    assert!(
        result.contains_key(study_hashiru.card_id()),
        "走る (new, N4) should be in lesson"
    );
    assert!(
        result.contains_key(study_nichi.card_id()),
        "日 (favorite) should be in lesson"
    );
}

#[test]
fn new_cards_interleaved_by_type_within_jlpt_level() {
    let grammar_rule_id_1 = Ulid::new();
    let grammar_rule_id_2 = Ulid::new();

    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["w1", "w2", "w3", "w4", "w5", "w6"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        ["k1", "k2"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.grammar_by_level.insert(
        JapaneseLevel::N5,
        [grammar_rule_id_1.to_string(), grammar_rule_id_2.to_string()]
            .into_iter()
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["w1", "w2", "w3", "w4", "w5", "w6"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("k1".to_string())))
        .unwrap();
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("k2".to_string())))
        .unwrap();
    knowledge_set
        .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            grammar_rule_id_1,
        )))
        .unwrap();
    knowledge_set
        .create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(
            grammar_rule_id_2,
        )))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(15),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let distinct_card_ids: HashSet<Ulid> = result.values().map(|lc| lc.card_id()).collect();
    assert_eq!(
        distinct_card_ids.len(),
        10,
        "Should return all 10 underlying cards (6 vocab + 2 kanji + 2 grammar); \
         expansion may add extra showings but not new card ids"
    );

    let has_kanji = result.values().any(|lc| {
        matches!(
            knowledge_set.get_card(lc.card_id()).unwrap().card(),
            Card::Kanji(_)
        )
    });
    let has_grammar = result.values().any(|lc| {
        matches!(
            knowledge_set.get_card(lc.card_id()).unwrap().card(),
            Card::Grammar(_)
        )
    });

    assert!(
        has_kanji,
        "Should contain at least one kanji — interleaving distributes types"
    );
    assert!(
        has_grammar,
        "Should contain at least one grammar — interleaving distributes types"
    );
}

#[test]
fn new_cards_interleave_handles_missing_type() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["w1", "w2", "w3", "w4", "w5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["w1", "w2", "w3", "w4", "w5"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(5),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert_eq!(
        result.len(),
        5,
        "All 5 vocab cards should be included when no kanji/grammar exist"
    );
}

#[test]
fn new_cards_interleave_across_jlpt_levels() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["n5w1", "n5w2", "n5w3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N5,
        ["n5k1"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["n4w1", "n4w2", "n4w3"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );
    jlpt_content.kanji_by_level.insert(
        JapaneseLevel::N4,
        ["n4k1"].into_iter().map(|s| s.to_string()).collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    for word in ["n5w1", "n5w2", "n5w3"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("n5k1".to_string())))
        .unwrap();
    for word in ["n4w1", "n4w2", "n4w3"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }
    knowledge_set
        .create_card(Card::Kanji(KanjiCard::new_test("n4k1".to_string())))
        .unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(4),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let distinct_card_ids: HashSet<Ulid> = result.values().map(|lc| lc.card_id()).collect();
    assert_eq!(
        distinct_card_ids.len(),
        4,
        "Should select exactly 4 underlying cards (limit=4); \
         multi-show expansion may add extra showings of those 4 card ids"
    );

    let n5_words: HashSet<&str> = ["n5w1", "n5w2", "n5w3"].into_iter().collect();
    let n5_kanji: &str = "n5k1";
    let n4_words: HashSet<&str> = ["n4w1", "n4w2", "n4w3"].into_iter().collect();
    let n4_kanji: &str = "n4k1";

    for lc in result.values() {
        let card = knowledge_set.get_card(lc.card_id()).unwrap().card();
        match card {
            Card::Vocabulary(v) => {
                let word = v.word().text();
                assert!(n5_words.contains(word), "{word} should be N5");
                assert!(!n4_words.contains(word), "{word} should not be N4");
            },
            Card::Kanji(k) => {
                let kanji = k.kanji().text();
                assert_eq!(kanji, n5_kanji, "Only N5 kanji should be selected");
                assert_ne!(kanji, n4_kanji, "N4 kanji should not be selected");
            },
            Card::Grammar(_) => panic!("No grammar cards in this test"),
            Card::Phrase(_) => panic!("No phrase cards in this test"),
        }
    }

    let has_n5_kanji = result
        .keys()
        .any(|id| matches!(knowledge_set.get_card(*id).unwrap().card(), Card::Kanji(_)));
    assert!(
        has_n5_kanji,
        "N5 group should include at least one kanji via interleaving"
    );
}

#[test]
fn new_cards_interleave_preserves_jlpt_priority() {
    let mut jlpt_content = JlptContent::new();
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N5,
        ["n5w1"].into_iter().map(|s| s.to_string()).collect(),
    );
    jlpt_content.words_by_level.insert(
        JapaneseLevel::N4,
        ["n4w1", "n4w2", "n4w3", "n4w4", "n4w5"]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
    );

    let mut knowledge_set = KnowledgeSet::new();
    let study_n5 = knowledge_set
        .create_card(create_vocab_card("n5w1"))
        .unwrap();
    for word in ["n4w1", "n4w2", "n4w3", "n4w4", "n4w5"] {
        knowledge_set.create_card(create_vocab_card(word)).unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(3),
        &jlpt_content,
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.contains_key(study_n5.card_id()),
        "N5 card (n5w1) should be selected — highest JLPT priority"
    );

    let n4_count = result
        .keys()
        .filter(|id| {
            let card = knowledge_set.get_card(**id).unwrap().card();
            matches!(card, Card::Vocabulary(v) if v.word().text().starts_with("n4"))
        })
        .count();

    assert_eq!(
        n4_count, 2,
        "Should contain exactly 2 N4 cards (limit=3 minus 1 N5)"
    );
}

#[test]
fn phrase_new_cards_limited() {
    ensure_test_phrase_index();

    // Four known anchor words (each a unique token of one fixture phrase) and
    // the four matching NEW phrase cards. Known anchors enter the core without
    // consuming the daily new-card allowance, so every phrase competes only for
    // the shared new-phrase budget.
    //
    // The known vocabulary pool built from the knowledge_set is exactly
    // {hello, bye, morning, thanks} — it does NOT contain "test", which is the
    // shared first token of every fixture phrase. Therefore `phrase_tail_eligible`
    // rejects all of them (every phrase leans on the unknown "test" token), the
    // tail stays empty, and only the interleaved fallback-to-known path yields
    // phrases — two of them, capped by the budget. That is why both phrases
    // land in the core and `tail_phrases == 0` here.
    let mut knowledge_set = KnowledgeSet::new();
    for word in ["hello", "bye", "morning", "thanks"] {
        let study = knowledge_set.create_card(create_vocab_card(word)).unwrap();
        knowledge_set.mark_card_as_known(*study.card_id()).unwrap();
    }
    for phrase_id in [
        phrase_id_hello(),
        phrase_id_bye(),
        phrase_id_morning(),
        phrase_id_thanks(),
    ] {
        knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
    }

    // DailyBudget::with_daily_cards(1) -> per-lesson phrase cap = 1 * 2.
    // Four phrases are available, so the cap must be the binding constraint.
    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(1),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let (core_phrases, tail_phrases) = count_phrases(&result);
    let total_phrases = core_phrases + tail_phrases;

    assert_eq!(
        total_phrases, 2,
        "New phrase cards must be capped at new_phrases_per_lesson (1 × 2), got {total_phrases}"
    );
    assert_eq!(
        core_phrases, 2,
        "Both budget-bound phrases land in the interleaved core section, got {core_phrases}"
    );
}

#[test]
fn phrase_does_not_increment_new_cards_studied() {
    let mut knowledge_set = KnowledgeSet::new();
    let mut phrase_ids = Vec::new();
    for _ in 0..5 {
        let phrase_id = Ulid::new();
        let study_card = knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
        phrase_ids.push(*study_card.card_id());
    }

    for id in phrase_ids {
        knowledge_set
            .rate_card(id, Rating::Good, RateMode::StandardLesson)
            .unwrap();
    }

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        0,
        "Phrase cards should not increment new_cards_studied_today"
    );
}

#[test]
fn phrase_new_cards_zero_when_limit_below_ratio() {
    let mut knowledge_set = KnowledgeSet::new();
    for _ in 0..20 {
        let phrase_id = Ulid::new();
        knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
    }
    for i in 0..5 {
        knowledge_set
            .create_card(create_vocab_card(&format!("vocab{i}")))
            .unwrap();
    }

    // DailyBudget::with_daily_cards(0) → new_phrases_per_lesson = 0
    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(0),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let phrase_count = result
        .keys()
        .filter(|id| {
            matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Phrase(_)
            )
        })
        .count();

    assert_eq!(
        phrase_count, 0,
        "Phrase cards should be 0 when new_phrases_per_lesson is 0, got {phrase_count}"
    );
}

#[test]
fn phrase_excluded_from_stats() {
    let mut knowledge_set = KnowledgeSet::new();
    let phrase_id = Ulid::new();
    let phrase_study = knowledge_set
        .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
        .unwrap();
    let vocab_study = knowledge_set.create_card(create_vocab_card("猫")).unwrap();

    knowledge_set
        .rate_card(
            *phrase_study.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();
    knowledge_set
        .rate_card(
            *vocab_study.card_id(),
            Rating::Good,
            RateMode::StandardLesson,
        )
        .unwrap();

    let history_item = &knowledge_set.lesson_history()[0];
    assert_eq!(
        history_item.total_words(),
        1,
        "Only vocab should be counted in total_words"
    );
    assert_eq!(
        history_item.lessons_completed(),
        1,
        "Only vocab should increment lessons_completed"
    );
}

#[test]
fn limited_types_still_respect_daily_limit() {
    let mut knowledge_set = KnowledgeSet::new();
    for i in 0..10 {
        knowledge_set
            .create_card(create_vocab_card(&format!("vocab{i}")))
            .unwrap();
    }
    for i in 0..10 {
        knowledge_set
            .create_card(Card::Kanji(KanjiCard::new_test(format!("kanji{i}"))))
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(5),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    // daily_new_limit bounds the count of distinct NEW cards introduced per
    // day. Multi-show expansion (`expand_repeated_views`) may add extra slots
    // showing the same card_id twice — those are not new cards, so the limit
    // must be checked against distinct card_ids, not against `result.len()`.
    // Before the proportional-slot fix distribute returned only Vocabulary
    // (so kanji never reached the multi-show pipeline) and the looser check
    // happened to hold; with the fix kanji enters the lesson and exercises
    // the multi-show path, exposing the original assertion's intent bug.
    let distinct_new_count: usize = result
        .values()
        .map(|lc| lc.card_id())
        .collect::<HashSet<_>>()
        .len();

    assert!(
        distinct_new_count <= 5,
        "Distinct new cards should respect daily limit of 5, got {distinct_new_count}"
    );
}

#[test]
fn lesson_size_respects_max_limit() {
    let mut knowledge_set = KnowledgeSet::new();

    for i in 0..100 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("word{i}")))
            .unwrap();
        knowledge_set
            .rate_card(
                *study_card.card_id(),
                Rating::Easy,
                RateMode::StandardLesson,
            )
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(100),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.len() <= MAX_LESSON_SIZE,
        "Total lesson size should not exceed MAX_LESSON_SIZE, got {}",
        result.len()
    );
}

#[test]
fn high_difficulty_cards_respect_max_lesson_size() {
    let mut knowledge_set = KnowledgeSet::new();

    // 100 карточек — все high-difficulty и due (rated Again + ShortTerm)
    for i in 0..100 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("hard{i}")))
            .unwrap();
        knowledge_set
            .rate_card(*study_card.card_id(), Rating::Again, RateMode::ShortTerm)
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    assert!(
        result.len() <= MAX_LESSON_SIZE,
        "High-difficulty cards should be capped at MAX_LESSON_SIZE, got {}",
        result.len()
    );
}

#[test]
fn phrases_added_after_core_cards_learning() {
    let mut knowledge_set = KnowledgeSet::new();

    for i in 0..20 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("core{i}")))
            .unwrap();
        knowledge_set
            .rate_card(*study_card.card_id(), Rating::Again, RateMode::ShortTerm)
            .unwrap();
    }

    for _ in 0..10 {
        let phrase_id = Ulid::new();
        let study_card = knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
        knowledge_set
            .rate_card(*study_card.card_id(), Rating::Again, RateMode::ShortTerm)
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(5),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let core_count = result
        .keys()
        .filter(|id| {
            !matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Phrase(_)
            )
        })
        .count();

    assert!(
        core_count >= 15,
        "Core cards should reach at least MIN_LESSON_SIZE, got {core_count}"
    );
}

#[test]
fn phrases_added_after_core_cards_review() {
    let mut knowledge_set = KnowledgeSet::new();

    for i in 0..20 {
        let study_card = knowledge_set
            .create_card(create_vocab_card(&format!("core{i}")))
            .unwrap();
        knowledge_set
            .mark_card_as_known(*study_card.card_id())
            .unwrap();
    }

    for _ in 0..10 {
        let phrase_id = Ulid::new();
        let study_card = knowledge_set
            .create_card(Card::Phrase(PhraseCard::new_test_with_id(phrase_id)))
            .unwrap();
        knowledge_set
            .mark_card_as_known(*study_card.card_id())
            .unwrap();
    }

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(5),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let core_count = result
        .keys()
        .filter(|id| {
            !matches!(
                knowledge_set.get_card(**id).unwrap().card(),
                Card::Phrase(_)
            )
        })
        .count();

    assert!(
        core_count >= 15,
        "Core cards should reach at least MIN_LESSON_SIZE, got {core_count}"
    );
}

#[test]
fn onboarding_scoring_does_not_consume_daily_limit() {
    let mut knowledge_set = KnowledgeSet::new();
    let mut all_ids = Vec::new();

    for i in 0..20 {
        let card = create_vocab_card(&format!("word{i}"));
        let study_card = knowledge_set.create_card(card).unwrap();
        all_ids.push(*study_card.card_id());
    }

    for id in &all_ids[..13] {
        knowledge_set
            .rate_card(*id, Rating::Easy, RateMode::OnboardingScoring)
            .unwrap();
    }

    assert_eq!(
        knowledge_set.new_cards_studied_today(),
        0,
        "OnboardingScoring should not increment new_cards_studied_today"
    );

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(15),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let new_in_lesson = result
        .iter()
        .filter(|(id, _)| knowledge_set.get_card(**id).unwrap().memory().is_new())
        .count();

    assert_eq!(
        new_in_lesson, 7,
        "All 7 remaining new cards should be in lesson (not limited by onboarding), got {new_in_lesson}"
    );
}

#[test]
fn favorite_card_appears_once_when_due_high_difficulty() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("猫");
    let study_card = knowledge_set.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    knowledge_set
        .rate_card(card_id, Rating::Again, RateMode::ShortTerm)
        .unwrap();

    knowledge_set.toggle_favorite(card_id).unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let count = result
        .card_ids()
        .iter()
        .filter(|id| **id == card_id)
        .count();
    assert_eq!(
        count, 1,
        "Favorite card should appear exactly once, got {count}"
    );
}

#[test]
fn favorite_card_appears_once_when_new() {
    let mut knowledge_set = KnowledgeSet::new();

    for i in 0..5 {
        knowledge_set
            .create_card(create_vocab_card(&format!("filler{i}")))
            .unwrap();
    }

    let card = create_vocab_card("犬");
    let study_card = knowledge_set.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    knowledge_set.toggle_favorite(card_id).unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let count = result
        .card_ids()
        .iter()
        .filter(|id| **id == card_id)
        .count();
    assert_eq!(
        count, 1,
        "Favorite new card should appear exactly once, got {count}"
    );
}

#[test]
fn favorite_card_appears_once_when_due_known() {
    let mut knowledge_set = KnowledgeSet::new();
    let card = create_vocab_card("鳥");
    let study_card = knowledge_set.create_card(card).unwrap();
    let card_id = *study_card.card_id();

    knowledge_set
        .rate_card(card_id, Rating::Good, RateMode::StandardLesson)
        .unwrap();

    knowledge_set.toggle_favorite(card_id).unwrap();

    let result = knowledge_set.cards_to_lesson(
        DailyBudget::with_daily_cards(10),
        &JlptContent::new(),
        JapaneseLevel::N5,
        NativeLanguage::Russian,
    );

    let count = result
        .card_ids()
        .iter()
        .filter(|id| **id == card_id)
        .count();
    assert_eq!(
        count, 1,
        "Favorite due+known card should appear exactly once, got {count}"
    );
}

mod companion_vocab_cards {
    use super::*;
    use crate::domain::NativeLanguage;
    use crate::use_cases::init_real_dictionaries;

    #[test]
    fn create_companion_vocab_cards_creates_cards_for_known_kanji() {
        init_real_dictionaries();

        let mut knowledge_set = KnowledgeSet::new();
        let created = knowledge_set.create_companion_vocab_cards("日", &NativeLanguage::Russian);

        assert!(
            !created.is_empty(),
            "Should create at least one companion card for 日"
        );
        assert!(
            created.len() <= 3,
            "Should create at most MAX_COMPANION_WORDS (3) cards, got {}",
            created.len()
        );

        for study_card in &created {
            assert!(
                matches!(study_card.card(), Card::Vocabulary(_)),
                "All companion cards should be vocabulary cards"
            );
        }
    }

    #[test]
    fn create_companion_vocab_cards_returns_empty_for_unknown_kanji() {
        init_real_dictionaries();

        let mut knowledge_set = KnowledgeSet::new();
        let created = knowledge_set.create_companion_vocab_cards("∃", &NativeLanguage::Russian);

        assert!(
            created.is_empty(),
            "Should return empty vec for unknown kanji"
        );
    }

    #[test]
    fn create_companion_vocab_cards_skips_duplicates() {
        init_real_dictionaries();

        let mut knowledge_set = KnowledgeSet::new();
        let first = knowledge_set.create_companion_vocab_cards("日", &NativeLanguage::Russian);
        let second = knowledge_set.create_companion_vocab_cards("日", &NativeLanguage::Russian);

        assert!(
            !first.is_empty(),
            "First call should create companion cards"
        );
        assert!(
            second.is_empty(),
            "Second call should return empty vec (all duplicates)"
        );
    }

    #[test]
    fn create_companion_vocab_cards_creates_fewer_than_max() {
        init_real_dictionaries();

        let mut knowledge_set = KnowledgeSet::new();

        let kanji_info = crate::dictionary::kanji::get_kanji_info("一").unwrap();
        let popular_count = kanji_info.popular_words().len();

        let created = knowledge_set.create_companion_vocab_cards("一", &NativeLanguage::Russian);

        assert!(
            created.len() <= popular_count.min(3),
            "Should create at most min(popular_words, MAX_COMPANION_WORDS) cards"
        );
    }
}

mod deleted_companion_words {
    use super::*;
    use crate::domain::{NativeLanguage, RateMode, Rating};
    use crate::use_cases::init_real_dictionaries;

    fn first_companion_word(created: &[StudyCard]) -> String {
        let sc = created.first().expect("at least one companion created");
        match sc.card() {
            Card::Vocabulary(v) => v.word().text().to_string(),
            _ => panic!("companion card must be Vocabulary"),
        }
    }

    fn vocab_word_of(sc: &StudyCard) -> String {
        match sc.card() {
            Card::Vocabulary(v) => v.word().text().to_string(),
            _ => panic!("card must be Vocabulary"),
        }
    }

    #[test]
    fn delete_vocab_card_records_word_in_blocklist() {
        let mut ks = KnowledgeSet::new();
        let sc = ks.create_card(create_vocab_card("猫")).unwrap();

        ks.delete_card(*sc.card_id()).unwrap();

        assert!(
            ks.deleted_companion_words_for_test().contains("猫"),
            "deleted vocab word must be recorded in the dismissed-companion blocklist"
        );
    }

    #[test]
    fn delete_non_vocab_card_does_not_record() {
        let mut ks = KnowledgeSet::new();
        let sc = ks
            .create_card(Card::Kanji(crate::domain::knowledge::KanjiCard::new_test(
                "日".to_string(),
            )))
            .unwrap();

        ks.delete_card(*sc.card_id()).unwrap();

        assert!(
            ks.deleted_companion_words_for_test().is_empty(),
            "deleting a non-Vocabulary card must not populate the blocklist"
        );
    }

    #[test]
    fn dismissed_companion_word_is_not_recreated() {
        init_real_dictionaries();

        let mut ks = KnowledgeSet::new();
        let created = ks.create_companion_vocab_cards("日", &NativeLanguage::Russian);
        assert!(!created.is_empty(), "fixture: 日 should yield companions");

        let dismissed = first_companion_word(&created);
        let dismissed_id = *created.first().unwrap().card_id();
        ks.delete_card(dismissed_id).unwrap();
        assert!(ks.deleted_companion_words_for_test().contains(&dismissed));

        let recreated = ks.create_companion_vocab_cards("日", &NativeLanguage::Russian);
        let dismissed_recreated = recreated.iter().any(|sc| vocab_word_of(sc) == dismissed);
        assert!(
            !dismissed_recreated,
            "a companion word the user dismissed must not be re-created on the next run"
        );
    }

    #[test]
    fn manual_create_card_evicts_blocklist_entry() {
        let mut ks = KnowledgeSet::new();
        let sc = ks.create_card(create_vocab_card("猫")).unwrap();
        ks.delete_card(*sc.card_id()).unwrap();
        assert!(ks.deleted_companion_words_for_test().contains("猫"));

        let recreated = ks.create_card(create_vocab_card("猫"));
        assert!(
            recreated.is_ok(),
            "manual create_card of a blocklisted word must succeed (blocklist only blocks companion auto-creation)"
        );
        assert!(
            !ks.deleted_companion_words_for_test().contains("猫"),
            "manual re-creation must evict the word from the blocklist"
        );
    }

    #[test]
    fn old_user_json_without_field_deserializes_empty() {
        let json = r#"{"study_cards":{},"deleted_cards":[],"lesson_history":[]}"#;

        let ks: KnowledgeSet = serde_json::from_str(json).expect("old JSON must deserialize");

        assert!(
            ks.deleted_companion_words_for_test().is_empty(),
            "Users persisted before the blocklist field must deserialize with an empty set"
        );
        assert!(ks.study_cards().is_empty());
    }

    #[test]
    fn serde_roundtrip_preserves_blocklist_and_stats() {
        let mut ks = KnowledgeSet::new();
        let foo = ks.create_card(create_vocab_card("foo")).unwrap();
        ks.rate_card(*foo.card_id(), Rating::Good, RateMode::StandardLesson)
            .unwrap();
        let bar = ks.create_card(create_vocab_card("bar")).unwrap();
        ks.delete_card(*bar.card_id()).unwrap();

        let json = serde_json::to_string(&ks).unwrap();
        let restored: KnowledgeSet = serde_json::from_str(&json).unwrap();

        assert!(
            restored.deleted_companion_words_for_test().contains("bar"),
            "blocklist must survive the serde round-trip"
        );
        assert!(
            !restored.lesson_history().is_empty(),
            "flattened stats (lesson_history) must survive alongside the blocklist"
        );
        assert_eq!(restored.study_cards().len(), 1);
    }

    #[test]
    fn merge_unions_blocklist_without_evicting_cards() {
        // Device A dismissed word W: blocklist={W}, no W card.
        let mut device_a = KnowledgeSet::new();
        let w_a = device_a.create_card(create_vocab_card("W")).unwrap();
        device_a.delete_card(*w_a.card_id()).unwrap();
        assert!(device_a.deleted_companion_words_for_test().contains("W"));
        assert!(device_a.study_cards().is_empty());

        // Device B legitimately (re)created W under a different ULID: has W card, empty blocklist.
        let mut device_b = KnowledgeSet::new();
        let w_b = device_b.create_card(create_vocab_card("W")).unwrap();
        assert_eq!(vocab_word_of(&w_b), "W");

        device_a.merge(&device_b);

        assert!(
            device_a.deleted_companion_words_for_test().contains("W"),
            "merge must union the blocklist so post-sync auto-creation stays suppressed"
        );
        assert!(
            device_a.study_cards().contains_key(w_b.card_id()),
            "merge must NOT evict a legitimately created card for a blocklisted word"
        );
    }
}
