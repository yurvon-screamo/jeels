use chrono::{DateTime, Utc};

use super::{CardType, KnowledgeSet};
use crate::domain::{DailyBudget, DailyLoad};

/// Why a lesson selection came out empty, distilled from the whole
/// [`KnowledgeSet`]. Drives the lesson empty state: each flag renders an
/// actionable block, and all applicable blocks are shown together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LessonEmptyDiagnosis {
    /// No NEW non-phrase cards exist at all — the deck is exhausted and
    /// importing content (sets / Anki) is the remedy. Phrases are excluded:
    /// they are auto-seeded reinforcement, not deck content.
    pub deck_exhausted: bool,
    /// New non-phrase cards remain, but today's new-card allowance is spent.
    /// Increasing the daily load (profile) unlocks them today.
    pub daily_limit_reached: bool,
    /// Earliest FUTURE scheduled review among non-phrase cards. Due cards
    /// are excluded (they would have formed a lesson); new cards have no
    /// schedule; phrases are excluded because `add_phrases` cannot anchor a
    /// phrase into an empty core, so a phrase-only review would never
    /// materialize a lesson.
    pub next_review: Option<DateTime<Utc>>,
}

/// Classifies the reasons a [`KnowledgeSet`] yields an empty lesson for a
/// user with the given [`DailyLoad`]. Pure: reads card memory states and the
/// daily history counters only.
pub fn diagnose_empty_lesson(
    knowledge_set: &KnowledgeSet,
    load: DailyLoad,
) -> LessonEmptyDiagnosis {
    let now = Utc::now();
    let budget = DailyBudget::from_load(load);

    let mut new_content_cards = 0usize;
    let mut earliest_future_review: Option<DateTime<Utc>> = None;

    for study_card in knowledge_set.study_cards().values() {
        if matches!(CardType::from(study_card.card()), CardType::Phrase) {
            continue;
        }
        if study_card.memory().is_new() {
            new_content_cards += 1;
        } else if let Some(next) = study_card.memory().next_review_date() {
            if *next > now {
                earliest_future_review = Some(match earliest_future_review {
                    Some(earliest) if earliest <= *next => earliest,
                    _ => *next,
                });
            }
        }
    }

    LessonEmptyDiagnosis {
        deck_exhausted: new_content_cards == 0,
        // The limit hint is only honest while new cards still remain —
        // otherwise the real problem is deck exhaustion.
        daily_limit_reached: new_content_cards > 0
            && knowledge_set.new_cards_studied_today() >= budget.new_cards_per_day(),
        next_review: earliest_future_review,
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::domain::knowledge::PhraseCard;
    use crate::domain::memory::Rating;
    use crate::domain::value_objects::Question;
    use crate::domain::{Card, KnowledgeSet, RateMode, VocabularyCard};

    fn vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn add_vocab(ks: &mut KnowledgeSet, word: &str) {
        ks.create_card(vocab_card(word)).expect("create vocab card");
    }

    fn rate_all_new_as_good(ks: &mut KnowledgeSet, limit: usize) {
        let card_ids: Vec<ulid::Ulid> = ks
            .study_cards()
            .iter()
            .filter(|(_, sc)| sc.memory().is_new())
            .map(|(id, _)| *id)
            .take(limit)
            .collect();
        for id in card_ids {
            ks.rate_card(id, Rating::Good, RateMode::StandardLesson)
                .expect("rate card");
        }
    }

    #[rstest]
    #[case::minimal(DailyLoad::Minimal)]
    #[case::medium(DailyLoad::Medium)]
    #[case::maximum(DailyLoad::Maximum)]
    fn empty_knowledge_set_reports_deck_exhausted_only(#[case] load: DailyLoad) {
        let ks = KnowledgeSet::new();

        let diagnosis = diagnose_empty_lesson(&ks, load);

        assert!(diagnosis.deck_exhausted, "no cards at all → deck exhausted");
        assert!(
            !diagnosis.daily_limit_reached,
            "no cards remain, so the daily-limit hint would be misleading"
        );
        assert_eq!(diagnosis.next_review, None, "nothing was ever reviewed");
    }

    #[test]
    fn new_cards_below_limit_do_not_flag_anything() {
        let mut ks = KnowledgeSet::new();
        add_vocab(&mut ks, "hello");
        add_vocab(&mut ks, "bye");

        // Medium = 9/day, nothing studied today.
        let diagnosis = diagnose_empty_lesson(&ks, DailyLoad::Medium);

        assert!(!diagnosis.deck_exhausted, "two new cards remain");
        assert!(!diagnosis.daily_limit_reached, "0 studied < 9 allowance");
        assert_eq!(diagnosis.next_review, None, "no card has a schedule yet");
    }

    #[test]
    fn spent_limit_with_remaining_new_cards_flags_daily_limit() {
        let mut ks = KnowledgeSet::new();
        for word in ["a", "b", "c", "d"] {
            add_vocab(&mut ks, word);
        }
        // Minimal = 3/day: study three of the four new cards.
        rate_all_new_as_good(&mut ks, 3);
        assert_eq!(ks.new_cards_studied_today(), 3);

        let diagnosis = diagnose_empty_lesson(&ks, DailyLoad::Minimal);

        assert!(!diagnosis.deck_exhausted, "one new card still remains");
        assert!(
            diagnosis.daily_limit_reached,
            "studied 3 ≥ Minimal allowance 3 while new cards remain"
        );
        assert!(
            diagnosis.next_review.is_some(),
            "rated cards schedule future reviews"
        );
    }

    #[test]
    fn fully_studied_deck_reports_exhaustion_and_next_review() {
        let mut ks = KnowledgeSet::new();
        add_vocab(&mut ks, "hello");
        rate_all_new_as_good(&mut ks, 1);

        let diagnosis = diagnose_empty_lesson(&ks, DailyLoad::Minimal);

        assert!(diagnosis.deck_exhausted, "no new cards remain");
        assert!(
            !diagnosis.daily_limit_reached,
            "1 studied < 3 allowance; exhaustion, not the limit, is the cause"
        );
        let next = diagnosis
            .next_review
            .expect("Good on a new card schedules a future review");
        assert!(
            next > Utc::now(),
            "the scheduled review must lie in the future"
        );
    }

    #[test]
    fn next_review_ignores_due_cards_and_phrases() {
        let mut ks = KnowledgeSet::new();
        // mark_card_as_known schedules the review in the PAST (due) — a due
        // card would have formed a lesson, so it must not be reported.
        let known = ks.create_card(vocab_card("known")).expect("create");
        ks.mark_card_as_known(*known.card_id()).expect("mark known");
        // A new phrase card is not deck content and has no schedule.
        ks.create_card(Card::Phrase(
            PhraseCard::new_test_with_id(ulid::Ulid::new()),
        ))
        .expect("create phrase");

        let diagnosis = diagnose_empty_lesson(&ks, DailyLoad::Medium);

        assert_eq!(diagnosis.next_review, None, "due card excluded");
        assert!(
            diagnosis.deck_exhausted,
            "the new phrase must not count as deck content"
        );
    }
}
