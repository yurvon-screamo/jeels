use super::lesson_state::LessonContext;
use crate::core::haptics;
use crate::hooks::phrase_checker;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::{Card, CardType, LessonCard, RateMode, Rating};
use origa::traits::UserRepository;
use origa::use_cases::{CreatePhraseCardUseCase, RateCardWithSideEffectsUseCase};
use tracing::warn;
use ulid::Ulid;

fn determine_rate_mode(card: &LessonCard) -> RateMode {
    if CardType::from(card.card()) == CardType::Phrase {
        return RateMode::PhraseReview;
    }
    if card.is_short_term() {
        return RateMode::ShortTerm;
    }
    match CardType::from(card.card()) {
        CardType::Grammar => RateMode::GrammarReview,
        CardType::Kanji => RateMode::KanjiReview,
        // Phrase cards are intercepted by the early return above, so the only
        // remaining type reaching this arm is Vocabulary.
        _ => RateMode::StandardLesson,
    }
}

fn extract_grammar_rule_id(card: &LessonCard) -> Option<Ulid> {
    match card.view() {
        origa::domain::LessonCardView::GrammarMutated { grammar_info, .. } => {
            grammar_info.rule_id()
        },
        origa::domain::LessonCardView::GrammarQuiz(gq) => gq.grammar_info().rule_id(),
        _ => None,
    }
}

async fn check_and_create_ready_phrases<R: UserRepository>(
    card_id: Ulid,
    card_type: CardType,
    repo: &R,
    rating: Rating,
) {
    // Early exit before the repo round-trip: phrase creation only applies to
    // vocabulary cards rated Good/Easy. Skipping the get_current_user() call
    // for Kanji/Grammar/Phrase cards eliminates a redundant IndexedDB read on
    // every rating, which was the primary source of the "slow kanji advance"
    // symptom — the async chain waited on this fetch before advancing.
    if rating == Rating::Again || card_type != CardType::Vocabulary {
        return;
    }

    let Some(user) = repo.get_current_user().await.ok().flatten() else {
        return;
    };

    let Some(sc) = user.knowledge_set().study_cards().get(&card_id) else {
        return;
    };

    let Card::Vocabulary(vocab) = sc.card() else {
        return;
    };

    let word = vocab.word().text().to_string();
    let ready_phrases =
        phrase_checker::find_ready_phrases(&word, user.knowledge_set().study_cards());

    if !ready_phrases.is_empty() {
        let create_phrase_use_case = CreatePhraseCardUseCase::new(repo);
        if let Err(e) = create_phrase_use_case.execute(ready_phrases).await {
            warn!(error = ?e, "Failed to create phrase cards");
        }
    }
}

fn advance_lesson_state(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    is_completed: RwSignal<bool>,
) {
    lesson_state.update(|state| {
        let next_index = state.current_index + 1;
        let total = state.card_ids.len();

        state.review_count += 1;
        state.waiting_for_next = false;
        state.pending_rating = None;

        if next_index >= total {
            is_completed.set(true);
        } else {
            state.current_index = next_index;
            state.showing_answer = false;
            state.selected_quiz_option = None;
            state.selected_yesno_answer = None;
            state.dont_know_selected = false;
            state.selected_quiz_options.clear();
            state.multi_quiz_submitted = false;
            state.multi_result = None;
        }
    });
}

pub fn create_on_rate_callback(
    lesson_state: RwSignal<super::lesson_state::LessonState>,
    lesson_ctx: LessonContext,
    is_rating: RwSignal<Option<Ulid>>,
) -> Callback<Rating> {
    let Some(is_disposed) = use_context::<StoredValue<()>>() else {
        return Callback::new(move |_: Rating| {});
    };

    Callback::new(move |rating: Rating| {
        // Fire haptic feedback immediately on the synchronous call stack so
        // the user feels it at the moment of contact, before the async rating
        // persistence begins. No-op on desktop and in plain browsers.
        haptics::rating_feedback(rating);

        let state = lesson_state.get_untracked();

        let Some(slot_id) = state.card_ids.get(state.current_index) else {
            return;
        };

        let slot_id = *slot_id;
        is_rating.set(Some(slot_id));

        let lesson_card = state.cards.get(&slot_id);
        let real_card_id = lesson_card.map(|lc| lc.card_id());
        let rate_mode = lesson_card
            .map(determine_rate_mode)
            .unwrap_or(RateMode::StandardLesson);
        let grammar_rule_id = lesson_card.and_then(extract_grammar_rule_id);
        let card_type = lesson_card
            .map(|lc| CardType::from(lc.card()))
            .unwrap_or(CardType::Vocabulary);

        let Some(real_card_id) = real_card_id else {
            return;
        };

        let repo = lesson_ctx.repository.clone();
        let lesson_state = lesson_state;
        let is_completed = lesson_ctx.is_completed;
        let is_rating = is_rating;

        spawn_local(async move {
            let use_case = RateCardWithSideEffectsUseCase::new(&repo);

            if let Err(e) = use_case
                .execute(real_card_id, rate_mode, rating, grammar_rule_id)
                .await
            {
                warn!(error = ?e, "Failed to rate card");
            }

            check_and_create_ready_phrases(real_card_id, card_type, &repo, rating).await;

            if is_disposed.is_disposed() {
                return;
            }

            advance_lesson_state(lesson_state, is_completed);
            is_rating.set(None);
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use origa::domain::{LessonCardView, PhraseCard};

    fn phrase_lesson_card(is_short_term: bool) -> LessonCard {
        let card = Card::Phrase(PhraseCard::new(Ulid::new()));
        LessonCard::new(Ulid::new(), LessonCardView::Normal(card), is_short_term)
    }

    // `VocabularyCard::new` is `#[cfg(test)] pub(crate)` in the `origa` crate,
    // so it cannot be reached from `origa_ui` tests (different crate, even with
    // `cfg(test)`). We therefore round-trip a vocab card through its public
    // serde representation. This is intentionally fragile: any change to the
    // `VocabularyCard` serialization shape will break this helper loudly, which
    // is preferable to a silent mismatch between the test fixture and prod data.
    fn vocab_lesson_card(is_short_term: bool) -> LessonCard {
        let card: Card = serde_json::from_str(
            r#"{"Vocabulary":{"word":{"text":"test"},"reverse_side":null,"pos":null}}"#,
        )
        .expect("deserialize vocab card");
        LessonCard::new(Ulid::new(), LessonCardView::Normal(card), is_short_term)
    }

    // A phrase card placed inside the core section (index < core_count) must
    // still be reviewed as a phrase now that the mode is derived from card
    // type rather than the positional threshold.
    #[test]
    fn determine_rate_mode_phrase_at_core_position() {
        let card = phrase_lesson_card(false);
        assert_eq!(determine_rate_mode(&card), RateMode::PhraseReview);
    }

    // A vocab card placed past the core boundary must NOT be treated as a
    // phrase review; it falls through to the standard / short-term branches.
    #[test]
    fn determine_rate_mode_vocab_at_phrase_position() {
        let card = vocab_lesson_card(false);
        assert_ne!(determine_rate_mode(&card), RateMode::PhraseReview);
        assert_eq!(determine_rate_mode(&card), RateMode::StandardLesson);
    }

    #[test]
    fn determine_rate_mode_short_term_vocab_uses_short_term() {
        let card = vocab_lesson_card(true);
        assert_eq!(determine_rate_mode(&card), RateMode::ShortTerm);
    }
}
