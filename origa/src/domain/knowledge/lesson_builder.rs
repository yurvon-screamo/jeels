use std::cmp::Reverse;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};

use rand::seq::SliceRandom;
use ulid::Ulid;

use super::lesson::{LessonCard, LessonCardView, LessonData, LessonViewGenerator};
use super::{Card, CardType, KnowledgeSet, StudyCard};
use crate::domain::{JapaneseLevel, JlptContent, NativeLanguage};

const MIN_LESSON_SIZE: usize = 15;
pub(crate) const MAX_LESSON_SIZE: usize = 22;
const PHRASE_NEW_RATIO: usize = 2;

/// Приоритет карточек без определённого JLPT уровня — ниже всех известных уровней (N1=1)
const UNKNOWN_JLPT_PRIORITY: u8 = 0;

/// Per-lesson cap on no-anchor (formerly "tail") phrases — phrases whose
/// tokens are all known but none of which anchor to a lesson vocab word. They
/// are MIXED INTO the lesson (no dedicated end zone, see PR #203). Under
/// Option alpha this slot is independent of `phrase_new_budget`: a depleted
/// budget no longer starves the no-anchor section, so up to this many no-anchor
/// phrases (due + new) appear every lesson.
const TAIL_PHRASE_PER_LESSON: usize = 5;

/// Tail phrases only reinforce already-mastered material: no single known word
/// may appear in more than this many tail phrases, otherwise a frequent word
/// (e.g. する) would crowd out the entire tail.
pub(crate) const MAX_PHRASES_PER_WORD_IN_TAIL: usize = 1;

/// How many phrases may be interleaved next to a single anchor word inside the
/// core section. Two cards let the learner meet the word in context shortly
/// after the standalone review without saturating the lesson.
const INTERLEAVED_PHRASES_PER_WORD: usize = 2;

/// Minimum number of cards that must sit between two consecutive showings
/// of the same primary card after multi-show expansion. Prevents
/// back-to-back ratings of the same underlying StudyCard.
const MIN_REPEAT_SPACING: usize = 3;

/// Minimum number of other cards that must sit between an anchor word and its
/// first interleaved phrase, so the phrase does not leak the answer back into
/// the word rating. Degrades to "as late as possible" on short lessons.
const INTERLEAVING_GAP: usize = 2;

/// Веса типов карточек для interleaving: Vocab:Kanji:Grammar ≈ 80:10:10.
/// При добавлении нового варианта в CardType — обновить эту константу.
const CARD_TYPE_WEIGHTS: [(CardType, usize); 3] = [
    (CardType::Vocabulary, 8),
    (CardType::Kanji, 1),
    (CardType::Grammar, 1),
];

/// Remaining new-ANCHORED-phrase allowance for the day. Encapsulates
/// `PHRASE_NEW_RATIO` so callers cannot reach past it. Under Option alpha this
/// budget governs only ANCHORED (interleaved) new phrases; no-anchor phrases
/// are capped per lesson by `TAIL_PHRASE_PER_LESSON` and do not decrement it.
pub(crate) fn compute_phrase_new_budget(daily_new_limit: usize, studied: usize) -> usize {
    (daily_new_limit * PHRASE_NEW_RATIO).saturating_sub(studied)
}

/// Collects the lesson core (favorites, due/new/known cards, padding) WITHOUT
/// any phrases. The core is shuffled here so downstream interleaving sees a
/// stable order. Phrases are attached later by `add_phrases`.
///
/// Returns the lesson data and the set of "primary" card ids (favorites,
/// selected core cards, padding). Companion cards added later are deliberately
/// excluded so the multi-show expansion treats them as exempt.
pub(crate) fn build_lesson_core(
    knowledge_set: &KnowledgeSet,
    daily_new_limit: usize,
    jlpt_content: &JlptContent,
    native_language: NativeLanguage,
) -> (LessonData, HashSet<Ulid>) {
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());

    let favorite_cards: Vec<_> = all_cards
        .iter()
        .filter(|(_, card)| card.is_favorite())
        .copied()
        .collect();

    let favorite_ids: HashSet<Ulid> = favorite_cards.iter().map(|(id, _)| **id).collect();

    let mut selected_cards = collect_core_high_difficulty(&all_cards, &favorite_ids);
    let mut rng = rand::rng();
    collect_core_new_cards(
        &all_cards,
        &mut selected_cards,
        &favorite_ids,
        knowledge_set.new_cards_studied_today(),
        daily_new_limit,
        jlpt_content,
        &mut rng,
    );
    fill_core_due_known(&all_cards, &mut selected_cards, &favorite_ids);

    let all_selected_ids = build_selected_ids(&selected_cards, &favorite_cards);
    let padding_cards = collect_padding(&all_cards, &all_selected_ids);

    let primary_card_ids: HashSet<Ulid> = favorite_ids
        .iter()
        .copied()
        .chain(selected_cards.iter().map(|(id, _)| **id))
        .chain(padding_cards.iter().map(|(id, _)| **id))
        .collect();

    let mut cards = build_core_lesson_cards(
        &favorite_cards,
        &selected_cards,
        &padding_cards,
        knowledge_set,
        native_language,
    );
    cards.shuffle(&mut rand::rng());
    let core_count = cards.len();

    (LessonData { cards, core_count }, primary_card_ids)
}

/// Reorders the core section (`cards[..core_count]`) so kanji and grammar are
/// spread across the lesson instead of clustering at the end. Vocab acts as the
/// separator spine: kanji and grammar are dealt round-robin into the `V+1` gaps
/// between vocab cards, bounding the longest same-type run to
/// `⌈count/(V+1)⌉`. The bound depends only on card counts, not on the shuffled
/// within-type order, so it is deterministic. When the core has no vocab there
/// is no separator to spread with, so the layout is left untouched.
pub(crate) fn interleave_core_by_type(mut lesson_data: LessonData) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count <= 1 {
        return lesson_data;
    }

    let vocab_count = lesson_data.cards[..core_count]
        .iter()
        .filter(|(_, lc)| CardType::from(lc.card()) == CardType::Vocabulary)
        .count();
    if vocab_count == 0 {
        return lesson_data;
    }

    let (mut vocab, mut kanji, mut grammar, mut other) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for card in lesson_data.cards.drain(..core_count) {
        match CardType::from(card.1.card()) {
            CardType::Vocabulary => vocab.push(card),
            CardType::Kanji => kanji.push(card),
            CardType::Grammar => grammar.push(card),
            CardType::Phrase => other.push(card),
        }
    }

    let num_gaps = vocab_count + 1;
    let mut gap_kanji: Vec<Vec<(Ulid, LessonCard)>> = (0..num_gaps).map(|_| Vec::new()).collect();
    for (i, card) in kanji.into_iter().enumerate() {
        gap_kanji[i % num_gaps].push(card);
    }
    let mut gap_grammar: Vec<Vec<(Ulid, LessonCard)>> = (0..num_gaps).map(|_| Vec::new()).collect();
    for (i, card) in grammar.into_iter().enumerate() {
        gap_grammar[i % num_gaps].push(card);
    }

    let mut new_core: Vec<(Ulid, LessonCard)> = Vec::with_capacity(core_count);
    new_core.append(&mut gap_kanji[0]);
    new_core.append(&mut gap_grammar[0]);
    for (i, vcard) in vocab.into_iter().enumerate() {
        new_core.push(vcard);
        new_core.append(&mut gap_kanji[i + 1]);
        new_core.append(&mut gap_grammar[i + 1]);
    }
    new_core.append(&mut other);

    lesson_data.cards.splice(..0, new_core);
    lesson_data
}

/// Core-section eligibility predicate shared by the high-difficulty, new and
/// due-known collectors: a card is a core candidate when it is neither a
/// user favorite (already pinned) nor a phrase (phrases enter the lesson via
/// the interleaved/tail pipelines, not the core).
fn is_core_candidate(id: &Ulid, card: &StudyCard, favorite_ids: &HashSet<Ulid>) -> bool {
    !favorite_ids.contains(id) && !matches!(card.card(), Card::Phrase(_))
}

fn collect_core_high_difficulty<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    favorite_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let limit = MAX_LESSON_SIZE.saturating_sub(favorite_ids.len());
    all_cards
        .iter()
        .filter(|(id, card)| {
            is_core_candidate(id, card, favorite_ids)
                && card.memory().is_due()
                && card.memory().is_high_difficulty()
        })
        .take(limit)
        .copied()
        .collect()
}

fn collect_core_new_cards<'a, R: rand::Rng>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
    new_cards_studied_today: usize,
    daily_new_limit: usize,
    jlpt_content: &JlptContent,
    rng: &mut R,
) {
    // Compute `allowed` BEFORE distribute so the per-type slot allocator
    // knows the actual quota (otherwise it would build an unbounded list
    // and we'd slice the tail off, defeating the proportional split).
    let available = MAX_LESSON_SIZE.saturating_sub(selected_cards.len() + favorite_ids.len());
    let daily_remaining = daily_new_limit
        .saturating_sub(new_cards_studied_today)
        .saturating_sub(
            selected_cards
                .iter()
                .filter(|(_, c)| c.memory().is_new())
                .count(),
        );
    let allowed = daily_remaining.min(available);

    if allowed == 0 {
        return;
    }

    let new_core_cards: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| is_core_candidate(id, card, favorite_ids) && card.memory().is_new())
        .copied()
        .collect();

    if new_core_cards.is_empty() {
        return;
    }

    let distributed = distribute_new_cards(new_core_cards, jlpt_content, allowed, rng);
    selected_cards.extend(distributed);
}

fn fill_core_due_known<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    selected_cards: &mut Vec<(&'a Ulid, &'a StudyCard)>,
    favorite_ids: &HashSet<Ulid>,
) {
    let current_count = selected_cards.len() + favorite_ids.len();
    let remaining = MAX_LESSON_SIZE.saturating_sub(current_count);
    if remaining == 0 {
        return;
    }

    let due_known: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            is_core_candidate(id, card, favorite_ids)
                && card.memory().is_due()
                && (card.memory().is_in_progress() || card.memory().is_known_card())
        })
        .take(remaining)
        .copied()
        .collect();
    selected_cards.extend(due_known);
}

fn collect_padding<'a>(
    all_cards: &[(&'a Ulid, &'a StudyCard)],
    all_selected_ids: &HashSet<Ulid>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    if all_selected_ids.len() >= MIN_LESSON_SIZE {
        return Vec::new();
    }
    let needed = MIN_LESSON_SIZE.saturating_sub(all_selected_ids.len());
    let mut candidates: Vec<_> = all_cards
        .iter()
        .filter(|(id, card)| {
            !all_selected_ids.contains(id)
                && !matches!(card.card(), Card::Phrase(_))
                && card.memory().is_high_difficulty()
        })
        .copied()
        .collect();
    candidates.sort_by_key(|(_, card)| card.memory().next_review_date());
    candidates.into_iter().take(needed).collect()
}

/// A phrase is tail-eligible when every non-particle token it references is
/// part of the known vocabulary pool. Grammatical particles (は, が, を, …)
/// attach to words rather than carrying standalone meaning, so they never
/// block eligibility. The pool spans the ENTIRE knowledge_set (built by
/// `collect_known_vocabulary_words`), not just lesson cards.
fn phrase_tail_eligible(phrase_id: &Ulid, known_pool: &HashSet<String>) -> bool {
    let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_id) else {
        return false;
    };
    entry.tokens().iter().all(|token| {
        crate::domain::grammar::is_grammatical_particle(token) || known_pool.contains(token)
    })
}

/// Bundles the immutable inputs driving no-anchor ("tail") phrase selection.
/// Grouping them keeps `collect_phrase_cards` below the argument-count
/// threshold and makes the selection context explicit at the call site. Under
/// Option alpha the no-anchor section does NOT draw from `phrase_new_budget`
/// (that budget governs only anchored phrases), so it is intentionally absent
/// here — see `TAIL_PHRASE_PER_LESSON` for the per-lesson cap.
struct TailPhraseSelection<'a> {
    all_cards: &'a [(&'a Ulid, &'a StudyCard)],
    excluded_card_ids: &'a HashSet<Ulid>,
    used_phrase_ids: &'a HashSet<Ulid>,
    known_pool: &'a HashSet<String>,
}

fn collect_phrase_cards<'a>(
    selection: &mut TailPhraseSelection<'a>,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    let phrase_eligible = |id: &&Ulid, card: &&StudyCard| {
        let Card::Phrase(phrase_card) = card.card() else {
            return false;
        };
        !selection.excluded_card_ids.contains(id)
            && !selection.used_phrase_ids.contains(phrase_card.phrase_id())
            && phrase_tail_eligible(phrase_card.phrase_id(), selection.known_pool)
    };

    // `all_cards` is sorted by `next_review_date` asc upstream (see
    // `add_phrases`), so filtering preserves the scheduling order without
    // an explicit secondary sort.
    let mut cap = PerWordCap::new(selection.known_pool);
    let mut phrase_cards: Vec<(&'a Ulid, &'a StudyCard)> = Vec::new();

    // Due no-anchor phrases first — free of any budget cost, but they still
    // occupy per-word cap slots so a frequent word cannot crowd out the
    // section through scheduling pressure alone.
    for (id, card) in selection.all_cards.iter().copied() {
        if !phrase_eligible(&id, &card) || !card.memory().is_due() {
            continue;
        }
        if cap.try_admit(card) {
            phrase_cards.push((id, card));
        }
    }

    // New no-anchor phrases are admitted on the per-word cap alone: under
    // Option alpha they do NOT decrement `phrase_new_budget` (reserved for
    // anchored phrases), so a depleted budget can no longer starve the
    // no-anchor section. The total is bounded per lesson by
    // `TAIL_PHRASE_PER_LESSON` via the truncate below.
    for (id, card) in selection.all_cards.iter().copied() {
        if !phrase_eligible(&id, &card) || !card.memory().is_new() {
            continue;
        }
        if cap.try_admit(card) {
            phrase_cards.push((id, card));
        }
    }

    phrase_cards.truncate(TAIL_PHRASE_PER_LESSON);
    phrase_cards
}

/// Streaming enforcer of `MAX_PHRASES_PER_WORD_IN_TAIL`. `try_admit` returns
/// `true` and reserves the phrase's known-word slots when the phrase still
/// fits the cap, `false` when at least one anchored word is already saturated.
/// Phrases are consumed in admission order (due before new) so the most
/// relevant phrase wins a word's slot when contention occurs.
struct PerWordCap<'a> {
    known_pool: &'a HashSet<String>,
    word_count: HashMap<&'static str, usize>,
}

impl<'a> PerWordCap<'a> {
    fn new(known_pool: &'a HashSet<String>) -> Self {
        Self {
            known_pool,
            word_count: HashMap::new(),
        }
    }

    fn try_admit(&mut self, card: &StudyCard) -> bool {
        let Card::Phrase(phrase_card) = card.card() else {
            return false;
        };
        let Some(entry) = crate::dictionary::phrase::get_index_entry(phrase_card.phrase_id())
        else {
            return false;
        };
        let over_cap = entry.tokens().iter().any(|token| {
            !crate::domain::grammar::is_grammatical_particle(token)
                && self.known_pool.contains(token)
                && self.word_count.get(token.as_str()).copied().unwrap_or(0)
                    >= MAX_PHRASES_PER_WORD_IN_TAIL
        });
        if over_cap {
            return false;
        }
        for token in entry.tokens() {
            if !crate::domain::grammar::is_grammatical_particle(token)
                && self.known_pool.contains(token)
            {
                *self.word_count.entry(token.as_str()).or_insert(0) += 1;
            }
        }
        true
    }
}

/// Interleaves phrase cards into the core stream so each phrase appears after
/// its anchor word with at least `gap` other cards between them. The invariant
/// `phrase_position > word_position` is preserved even when the lesson is too
/// short to honour the gap (remaining phrases flush at the end).
fn interleave_with_gap(
    core_cards: Vec<(Ulid, LessonCard)>,
    mut assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>>,
    gap: usize,
) -> Vec<(Ulid, LessonCard)> {
    let pending_phrases: usize = assignments.values().map(|v| v.len()).sum();
    let mut result: Vec<(Ulid, LessonCard)> =
        Vec::with_capacity(core_cards.len() + pending_phrases);
    let mut pending: Vec<(usize, (Ulid, LessonCard))> = Vec::new();

    for card in core_cards {
        let word_id = card.0;
        result.push(card);

        if let Some(phrases) = assignments.remove(&word_id) {
            let word_pos = result.len() - 1;
            for phrase in phrases {
                pending.push((word_pos + gap + 1, phrase));
            }
        }

        let mut deferred = Vec::with_capacity(pending.len());
        for (min_pos, phrase) in pending.drain(..) {
            if result.len() >= min_pos {
                result.push(phrase);
            } else {
                deferred.push((min_pos, phrase));
            }
        }
        pending = deferred;
    }

    for (_, phrase) in pending {
        result.push(phrase);
    }

    // Any assignment left in `assignments` had no matching core card and would
    // be silently dropped. Surface this as a programmer error in debug builds.
    debug_assert!(
        assignments.is_empty(),
        "interleave_with_gap dropped phrase assignments for words not present in core_cards"
    );

    result
}

/// Picks up to `INTERLEAVED_PHRASES_PER_WORD` phrase study cards for a single
/// anchor word. Due phrases win slots for free; new phrases fill the remainder
/// and each consumes one unit of the shared new-phrase budget.
fn collect_interleaved_phrases_for_word<'a>(
    word: &str,
    phrase_cards_by_id: &'a HashMap<Ulid, (&'a Ulid, &'a StudyCard)>,
    in_lesson: &'a HashSet<Ulid>,
    used_phrase_ids: &'a mut HashSet<Ulid>,
    phrase_new_budget: &'a mut usize,
) -> Vec<(Ulid, &'a StudyCard)> {
    let mut picker = InterleavePicker {
        phrase_cards_by_id,
        in_lesson,
        used_phrase_ids,
        phrase_new_budget,
    };
    let entries = crate::dictionary::phrase::get_phrases_by_token(word);
    let mut picked: Vec<(Ulid, &'a StudyCard)> = Vec::new();

    // `MemoryState::is_due` already implies `!is_new`, so due phrases are a
    // strict subset disjoint from the new-phrase pass below.
    picker.fill(&entries, &mut picked, |sc| sc.memory().is_due(), false);
    picker.fill(&entries, &mut picked, |sc| sc.memory().is_new(), true);

    picked
}

/// Shared selection state for the two interleaving passes (due then new) of a
/// single anchor word. Grouping the lookup inputs keeps `fill` below the
/// argument-count threshold and makes the pass context explicit.
struct InterleavePicker<'a> {
    phrase_cards_by_id: &'a HashMap<Ulid, (&'a Ulid, &'a StudyCard)>,
    in_lesson: &'a HashSet<Ulid>,
    used_phrase_ids: &'a mut HashSet<Ulid>,
    phrase_new_budget: &'a mut usize,
}

impl<'a> InterleavePicker<'a> {
    /// Appends eligible phrases to `picked` until `INTERLEAVED_PHRASES_PER_WORD`
    /// is reached or the budget runs out. `consume_budget` ties new-phrase
    /// consumption to the shared allowance (free for due phrases).
    fn fill<F>(
        &mut self,
        entries: &[&'static crate::dictionary::phrase::IndexEntry],
        picked: &mut Vec<(Ulid, &'a StudyCard)>,
        memory_predicate: F,
        consume_budget: bool,
    ) where
        F: Fn(&StudyCard) -> bool,
    {
        for entry in entries {
            if picked.len() >= INTERLEAVED_PHRASES_PER_WORD {
                break;
            }
            if consume_budget && *self.phrase_new_budget == 0 {
                break;
            }
            let pid = entry.id();
            if self.used_phrase_ids.contains(pid) {
                continue;
            }
            let Some(&(card_id, sc)) = self.phrase_cards_by_id.get(pid) else {
                continue;
            };
            if self.in_lesson.contains(card_id) {
                continue;
            }
            if !memory_predicate(sc) {
                continue;
            }
            picked.push((*card_id, sc));
            self.used_phrase_ids.insert(*pid);
            if consume_budget {
                *self.phrase_new_budget -= 1;
            }
        }
    }
}

/// Collects up to `INTERLEAVED_PHRASES_PER_WORD` phrase card ids per anchor
/// word. Due phrases win slots for free; new phrases consume the shared budget.
/// Dedupes by word text so the same word is never processed twice. Returns
/// owned card ids (not references) so the shared `&mut` borrow of the budget
/// set is released between iterations.
fn collect_anchored_phrase_card_ids(
    targets: &[(Ulid, String)],
    phrase_cards_by_id: &HashMap<Ulid, (&Ulid, &StudyCard)>,
    in_lesson: &HashSet<Ulid>,
    used_phrase_ids: &mut HashSet<Ulid>,
    phrase_new_budget: &mut usize,
) -> Vec<Ulid> {
    let mut seen_words: HashSet<&str> = HashSet::new();
    let mut out = Vec::new();
    for (_, word_text) in targets {
        if !seen_words.insert(word_text.as_str()) {
            continue;
        }
        let picked = collect_interleaved_phrases_for_word(
            word_text,
            phrase_cards_by_id,
            in_lesson,
            used_phrase_ids,
            phrase_new_budget,
        );
        for (card_id, _) in picked {
            out.push(card_id);
        }
    }
    out
}

/// Inserts phrases (anchored + no-anchor) into the core so each phrase lands
/// after the first showing of every lesson-vocab word it references,
/// distributing the rest instead of dumping them at the end. All phrases become
/// part of the core (`core_count` grows to the whole lesson), removing the
/// dedicated tail zone. Under Option alpha: `phrase_new_budget` bounds NEW
/// ANCHORED phrases (per-word cap `INTERLEAVED_PHRASES_PER_WORD`, plus the
/// daily budget — there is deliberately no per-lesson TOTAL cap on anchored
/// phrases); NEW no-anchor phrases are admitted independently, capped per
/// lesson by `TAIL_PHRASE_PER_LESSON`, so a depleted budget can no longer
/// starve the no-anchor section.
pub(crate) fn add_phrases(
    mut lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
    phrase_new_budget: &mut usize,
) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count == 0 {
        return lesson_data;
    }

    let phrase_cards_by_id: HashMap<Ulid, (&Ulid, &StudyCard)> = knowledge_set
        .study_cards()
        .iter()
        .filter_map(|(id, sc)| match sc.card() {
            Card::Phrase(pc) => Some((*pc.phrase_id(), (id, sc))),
            _ => None,
        })
        .collect();

    let in_lesson: HashSet<Ulid> = lesson_data.cards.iter().map(|(id, _)| *id).collect();
    let mut used_phrase_ids: HashSet<Ulid> = HashSet::new();

    let core_vocab: Vec<(Ulid, String)> = lesson_data.cards[..core_count]
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect();

    // Anchored interleaving reinforces vocab still being learned; fall back to
    // known vocab only when no non-known anchor yields any phrase.
    let (non_known, known) = core_vocab
        .iter()
        .cloned()
        .partition::<Vec<_>, _>(|(id, _)| {
            knowledge_set
                .get_card(*id)
                .map(|sc| !sc.memory().is_known_card())
                .unwrap_or(false)
        });

    let mut selected_ids: Vec<Ulid> = collect_anchored_phrase_card_ids(
        &non_known,
        &phrase_cards_by_id,
        &in_lesson,
        &mut used_phrase_ids,
        phrase_new_budget,
    );
    if selected_ids.is_empty() && !known.is_empty() {
        selected_ids = collect_anchored_phrase_card_ids(
            &known,
            &phrase_cards_by_id,
            &in_lesson,
            &mut used_phrase_ids,
            phrase_new_budget,
        );
    }

    // No-anchor phrases: whole known-pool eligibility, per-word cap, and a
    // per-lesson count bounded by `TAIL_PHRASE_PER_LESSON`. Under Option alpha
    // they do NOT draw from `phrase_new_budget` (which stays reserved for the
    // anchored pass above), so they survive even a depleted budget.
    let mut all_cards = knowledge_set.study_cards().iter().collect::<Vec<_>>();
    all_cards.sort_by_key(|(_, card)| card.memory().next_review_date());
    let known_pool =
        super::collect_known_vocabulary_words(knowledge_set.study_cards().values(), true);
    let mut tail_selection = TailPhraseSelection {
        all_cards: &all_cards,
        excluded_card_ids: &in_lesson,
        used_phrase_ids: &used_phrase_ids,
        known_pool: &known_pool,
    };
    let tail_cards = collect_phrase_cards(&mut tail_selection);
    selected_ids.extend(tail_cards.iter().map(|(id, _)| **id));

    if selected_ids.is_empty() {
        return lesson_data;
    }

    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);
    let phrase_lessons: Vec<(Ulid, LessonCard)> = selected_ids
        .iter()
        .filter_map(|card_id| {
            let sc = knowledge_set.get_card(*card_id)?;
            let view = generator.apply_view(sc, sc.is_new(), &mut rand::rng());
            Some((*card_id, LessonCard::new(*card_id, view, false)))
        })
        .collect();

    let core_cards = std::mem::take(&mut lesson_data.cards);
    lesson_data.cards =
        place_phrases_constraint_aware(core_cards, phrase_lessons, &core_vocab, INTERLEAVING_GAP);
    lesson_data.core_count = lesson_data.cards.len();
    lesson_data
}

/// Places `phrases` among `core_cards` honouring the contamination constraint:
/// a phrase that references a lesson-vocab word must appear AFTER the first
/// showing of every such word. Each phrase is released at the latest
/// first-occurrence among its anchor words (so it follows all of them), then
/// handed to `interleave_with_gap` which keeps it `INTERLEAVING_GAP` cards past
/// the release point. Phrases with no anchor in the lesson are distributed at
/// even intervals so they no longer pile up at the end.
fn place_phrases_constraint_aware(
    core_cards: Vec<(Ulid, LessonCard)>,
    phrases: Vec<(Ulid, LessonCard)>,
    core_vocab: &[(Ulid, String)],
    gap: usize,
) -> Vec<(Ulid, LessonCard)> {
    let n = core_cards.len();

    let mut first_pos: HashMap<Ulid, usize> = HashMap::with_capacity(n);
    for (i, (id, _)) in core_cards.iter().enumerate() {
        first_pos.entry(*id).or_insert(i);
    }

    let word_to_card: HashMap<String, Ulid> = core_vocab
        .iter()
        .map(|(id, word)| (word.clone(), *id))
        .collect();

    let mut releases: Vec<(usize, (Ulid, LessonCard))> = Vec::with_capacity(phrases.len());
    let mut no_anchor: Vec<(Ulid, LessonCard)> = Vec::new();

    for phrase in phrases {
        let phrase_id = match phrase.1.card() {
            Card::Phrase(p) => *p.phrase_id(),
            _ => {
                no_anchor.push(phrase);
                continue;
            },
        };
        let tokens: &[String] = crate::dictionary::phrase::get_index_entry(&phrase_id)
            .map(|e| e.tokens())
            .unwrap_or(&[]);
        let anchors: Vec<usize> = tokens
            .iter()
            .filter_map(|t| word_to_card.get(t.as_str()).copied())
            .filter_map(|card_id| first_pos.get(&card_id).copied())
            .collect();
        if anchors.is_empty() {
            no_anchor.push(phrase);
        } else {
            let max_pos = *anchors.iter().max().expect("anchors non-empty");
            releases.push((max_pos, phrase));
        }
    }

    let no_anchor_count = no_anchor.len();
    for (i, phrase) in no_anchor.into_iter().enumerate() {
        let target_idx = if n == 0 {
            0
        } else {
            (((i + 1) * n) / (no_anchor_count + 1)).min(n - 1)
        };
        releases.push((target_idx, phrase));
    }

    let mut assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = HashMap::new();
    for (idx, phrase) in releases {
        if let Some((release_id, _)) = core_cards.get(idx) {
            assignments.entry(*release_id).or_default().push(phrase);
        }
    }

    interleave_with_gap(core_cards, assignments, gap)
}

/// Per-card target number of showings, derived from the FSRS memory state.
/// Hard and in-progress/new cards both get a light drill of 2 showings; known
/// cards keep their original single showing.
fn target_showings(study_card: &StudyCard) -> usize {
    let memory = study_card.memory();
    if memory.is_high_difficulty() || memory.is_new() || memory.is_in_progress() {
        2
    } else {
        1
    }
}

/// Decides whether a primary card slot should be expanded into multiple
/// showings, and if so returns the COPY views (the extra showings beyond the
/// primary). The primary itself is the probabilistic view `apply_view` already
/// assigned upstream (`primary_view`); copies are drawn from
/// `candidate_views_for_repeat` with the primary's variant removed, so every
/// showing of the card uses a distinct `LessonCardView` variant. Returns an
/// empty vector when the card is exempt (not primary, not a multi-show type,
/// target is 1, or no distinct copy variant is available).
fn compute_expansion_views(
    generator: &mut LessonViewGenerator,
    knowledge_set: &KnowledgeSet,
    primary_card_ids: &HashSet<Ulid>,
    card_id: Ulid,
    card_type: CardType,
    primary_view: &LessonCardView,
) -> Vec<LessonCardView> {
    if !primary_card_ids.contains(&card_id) {
        return Vec::new();
    }
    if !matches!(
        card_type,
        CardType::Vocabulary | CardType::Kanji | CardType::Grammar
    ) {
        return Vec::new();
    }

    let Some(study_card) = knowledge_set.get_card(card_id) else {
        return Vec::new();
    };

    let target = target_showings(study_card);
    if target <= 1 {
        return Vec::new();
    }

    let primary_disc = std::mem::discriminant(primary_view);
    let mut candidates =
        generator.candidate_views_for_repeat(study_card, study_card.is_new(), &mut rand::rng());
    candidates.retain(|view| std::mem::discriminant(view) != primary_disc);
    candidates.truncate(target.saturating_sub(1));

    candidates
}

/// Multiplies primary (non-phrase) cards across multiple distinct views when
/// their FSRS state demands it. Each copy occupies its own slot (unique slot
/// id) but shares the underlying StudyCard id (`card_id`), so every showing is
/// rated independently. Companion cards, phrases, and primary cards with a
/// single-show target (or whose guards clamp to a single distinct view) keep
/// their original slot unchanged.
///
/// Runs last in the pipeline so every upstream step still operates by slot id.
/// `core_count` grows by the number of copies added so the tail-vs-core
/// boundary stays contiguous.
pub(crate) fn expand_repeated_views(
    lesson_data: LessonData,
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
    primary_card_ids: &HashSet<Ulid>,
) -> LessonData {
    let original_cards = lesson_data.cards;
    let core_count_before = lesson_data.core_count;

    let (core_cards, tail_cards) = original_cards.split_at(core_count_before);

    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);
    let mut new_core: Vec<(Ulid, LessonCard)> = Vec::with_capacity(core_cards.len() * 2);
    let mut pending: Vec<(usize, Ulid, LessonCardView, bool)> = Vec::new();

    for (slot_id, lc) in core_cards.iter() {
        let card_id = lc.card_id();
        let card_type = CardType::from(lc.card());
        let is_short_term = lc.is_short_term();
        let primary_view = lc.view().clone();

        let copy_views = compute_expansion_views(
            &mut generator,
            knowledge_set,
            primary_card_ids,
            card_id,
            card_type,
            &primary_view,
        );

        if copy_views.is_empty() {
            new_core.push((*slot_id, lc.clone()));
            drain_pending(&mut new_core, &mut pending);
            continue;
        }

        new_core.push((
            *slot_id,
            LessonCard::new(card_id, primary_view, is_short_term),
        ));

        // Each extra showing of the same card_id must land MIN_REPEAT_SPACING
        // cards after the PREVIOUS showing of that card_id. Translating
        // "N cards between" into index deltas: positions differ by N+1. The
        // anchor sits at index `new_core.len() - 1`, so the first extra view
        // targets `new_core.len() + MIN_REPEAT_SPACING` (== anchor_idx +
        // MIN_REPEAT_SPACING + 1). Subsequent views step by the same delta so
        // every gap honours the same invariant.
        let mut next_min_pos = new_core.len() + MIN_REPEAT_SPACING;
        for view in copy_views {
            pending.push((next_min_pos, card_id, view, is_short_term));
            next_min_pos += MIN_REPEAT_SPACING + 1;
        }

        drain_pending(&mut new_core, &mut pending);
    }

    new_core = distribute_pending_with_spacing(new_core, pending);
    let added = new_core.len() - core_cards.len();

    let mut final_cards = new_core;
    final_cards.extend(tail_cards.iter().cloned());

    LessonData {
        cards: final_cards,
        core_count: core_count_before + added,
    }
}

/// Flushes any pending expansion copies whose minimum position has been
/// reached. Remaining copies stay in `pending` for a future iteration.
fn drain_pending(
    new_core: &mut Vec<(Ulid, LessonCard)>,
    pending: &mut Vec<(usize, Ulid, LessonCardView, bool)>,
) {
    let mut deferred = Vec::new();
    for (min_pos, p_card_id, p_view, p_short) in pending.drain(..) {
        if new_core.len() >= min_pos {
            new_core.push((Ulid::new(), LessonCard::new(p_card_id, p_view, p_short)));
        } else {
            deferred.push((min_pos, p_card_id, p_view, p_short));
        }
    }
    *pending = deferred;
}

/// Distributes expansion copies that did not fit during the main loop.
/// Each copy is inserted at the earliest position that keeps at least
/// `MIN_REPEAT_SPACING` cards between it and the previous showing of the
/// same `card_id`, instead of blindly appending the leftovers back-to-back
/// (which would make consecutive showings of one card land adjacent and
/// defeat the purpose of the spacing rule).
///
/// Contract: spacing is guaranteed only when the assembled core is large
/// enough to absorb every pending copy at its required gap. On a
/// too-short lesson — anchor near the end of the core, or a single-card
/// core whose target forces more copies than the buffer can hold — the
/// target index is clamped to `core.len()` and copies cluster at the end
/// of the core section. This is the only mathematically unreachable case
/// (the main loop already spaces copies whenever the core has room, see
/// `expand_enforces_min_spacing_between_consecutive_showings`). The
/// "past `core_count` is phrase-only" invariant is preserved because
/// copies are inserted strictly inside the core section; tail phrases
/// remain at the very end of the lesson.
fn distribute_pending_with_spacing(
    mut core: Vec<(Ulid, LessonCard)>,
    mut pending: Vec<(usize, Ulid, LessonCardView, bool)>,
) -> Vec<(Ulid, LessonCard)> {
    if pending.is_empty() {
        return core;
    }
    pending.sort_by_key(|(min_pos, _, _, _)| *min_pos);

    for (min_pos, card_id, view, is_short_term) in pending {
        let last_pos = core
            .iter()
            .enumerate()
            .rev()
            .find(|(_, (_, lc))| lc.card_id() == card_id)
            .map(|(idx, _)| idx);
        let spacing_target = last_pos
            .map(|prev| prev + MIN_REPEAT_SPACING + 1)
            .unwrap_or(min_pos);
        let target = spacing_target.max(min_pos).min(core.len());
        core.insert(
            target,
            (Ulid::new(), LessonCard::new(card_id, view, is_short_term)),
        );
    }
    core
}

/// Reorders `content` so consecutive showings of the same `card_id` stay at
/// least `MIN_REPEAT_SPACING` apart whenever the slot count allows it, using
/// the Task-Scheduler greedy: at each output position the slot of the
/// available group with the highest remaining count is emitted ("available" =
/// the group's previous showing is more than `MIN_REPEAT_SPACING` positions
/// back). This is the canonical min-distance construction — it spreads a LONE
/// multi-show card across the whole core (its copies land exactly
/// `MIN_REPEAT_SPACING + 1` apart when it is the bottleneck) while still
/// interleaving many multi-show cards. When the core is structurally too small
/// to honour every gap (the counts cannot fit), the greedy degrades to
/// best-effort by emitting the most-loaded group anyway — the only case where
/// `MIN_REPEAT_SPACING` may be violated, matching the upstream best-effort
/// contract. Within-group view order (`[primary, copy1, copy2]`) and the
/// "primary is the first showing" invariant are preserved: a group's queue is
/// always drained front-to-back.
fn deal_by_card_id(content: Vec<(Ulid, LessonCard)>) -> Vec<(Ulid, LessonCard)> {
    let n = content.len();
    if n <= 1 {
        return content;
    }

    // (queue of slots in original view order, first_original_index)
    let mut queues: Vec<(VecDeque<(Ulid, LessonCard)>, usize)> = Vec::new();
    let mut index_by_card: HashMap<Ulid, usize> = HashMap::new();
    for (i, slot) in content.into_iter().enumerate() {
        let card_id = slot.1.card_id();
        match index_by_card.get(&card_id) {
            Some(&qi) => queues[qi].0.push_back(slot),
            None => {
                index_by_card.insert(card_id, queues.len());
                queues.push((VecDeque::from([slot]), i));
            },
        }
    }

    let m = queues.len();
    let mut last_pos: Vec<Option<usize>> = vec![None; m];
    let mut result: Vec<(Ulid, LessonCard)> = Vec::with_capacity(n);

    for p in 0..n {
        // Spacing-respecting pass first; fall back to the most-loaded group
        // (ignoring cooldown) only when nothing is available — the sole path
        // that can violate MIN_REPEAT_SPACING, reachable on a structurally
        // overloaded core.
        let chosen = pick_group(&queues, &last_pos, p, false)
            .or_else(|| pick_group(&queues, &last_pos, p, true));
        let Some(qi) = chosen else {
            break;
        };
        if let Some(slot) = queues[qi].0.pop_front() {
            last_pos[qi] = Some(p);
            result.push(slot);
        }
    }
    result
}

/// Selects the next group to emit at output position `p`. With `force` false
/// only groups whose previous showing is more than `MIN_REPEAT_SPACING`
/// positions back qualify (the spacing-respecting pass); with `force` true the
/// cooldown is ignored (the best-effort fallback). The highest remaining count
/// wins; ties are broken by earliest first-occurrence so the pick is
/// deterministic regardless of the random slot/card ULIDs.
fn pick_group(
    queues: &[(VecDeque<(Ulid, LessonCard)>, usize)],
    last_pos: &[Option<usize>],
    p: usize,
    force: bool,
) -> Option<usize> {
    let mut best: Option<(usize, usize, usize)> = None;
    for (qi, (queue, first_idx)) in queues.iter().enumerate() {
        let rem = queue.len();
        if rem == 0 {
            continue;
        }
        let cooled = last_pos[qi].map_or(true, |lp| p - lp > MIN_REPEAT_SPACING);
        if !force && !cooled {
            continue;
        }
        match best {
            None => best = Some((qi, rem, *first_idx)),
            Some((_, b_rem, b_first)) => {
                if rem > b_rem || (rem == b_rem && *first_idx < b_first) {
                    best = Some((qi, rem, *first_idx));
                }
            },
        }
    }
    best.map(|(qi, _, _)| qi)
}

/// Final layout pass: reorders the core section so consecutive showings of the
/// same `card_id` are maximally separated, removing the back-to-back
/// clustering that `expand_repeated_views` leaves when a multi-show anchor
/// sits near the end of a short core. Content (vocab/kanji/grammar, including
/// multi-show copies) is re-dealt via `deal_by_card_id`; phrases are then
/// re-placed through the existing constraint-aware placer so the
/// phrase-after-word invariant (PR #203) is re-derived from the new content
/// order. `core_count` and the tail section are preserved: the dealt core has
/// the same length, and inserting phrases between content cards only widens
/// the gaps between them.
pub(crate) fn redistribute_core_for_spacing(mut lesson_data: LessonData) -> LessonData {
    let core_count = lesson_data.core_count;
    if core_count <= 1 {
        return lesson_data;
    }

    let core: Vec<(Ulid, LessonCard)> = lesson_data.cards.drain(..core_count).collect();
    let (content, phrases): (Vec<_>, Vec<_>) = core
        .into_iter()
        .partition(|(_, lc)| !matches!(lc.card(), Card::Phrase(_)));

    let core_vocab: Vec<(Ulid, String)> = content
        .iter()
        .filter_map(|(id, lc)| match lc.card() {
            Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
            _ => None,
        })
        .collect();

    let dealt = deal_by_card_id(content);
    let reordered = if phrases.is_empty() {
        dealt
    } else if dealt.is_empty() {
        // Defensive: a core made only of phrases has no content to anchor or
        // reorder against — keep the phrases in place rather than dropping them
        // (unreachable in the real pipeline, where the core always holds at
        // least one non-phrase card).
        phrases
    } else {
        place_phrases_constraint_aware(dealt, phrases, &core_vocab, INTERLEAVING_GAP)
    };

    lesson_data.cards.splice(..0, reordered);
    lesson_data
}

fn build_selected_ids(
    selected_cards: &[(&Ulid, &StudyCard)],
    favorite_cards: &[(&Ulid, &StudyCard)],
) -> HashSet<Ulid> {
    let selected_ids: HashSet<_> = selected_cards.iter().map(|(id, _)| **id).collect();
    let favorite_ids: HashSet<_> = favorite_cards.iter().map(|(id, _)| **id).collect();
    selected_ids.union(&favorite_ids).copied().collect()
}

fn build_core_lesson_cards(
    favorite_cards: &[(&Ulid, &StudyCard)],
    selected_cards: &[(&Ulid, &StudyCard)],
    padding_cards: &[(&Ulid, &StudyCard)],
    knowledge_set: &KnowledgeSet,
    native_language: NativeLanguage,
) -> Vec<(Ulid, LessonCard)> {
    let padding_ids: HashSet<_> = padding_cards.iter().map(|(id, _)| **id).collect();
    let mut generator = LessonViewGenerator::new(knowledge_set, native_language);

    let favorite_lessons: Vec<_> = favorite_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(**card_id, view, is_short_term))
        })
        .collect();

    let selected_lessons: Vec<_> = selected_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            let is_short_term = padding_ids.contains(card_id);
            (**card_id, LessonCard::new(**card_id, view, is_short_term))
        })
        .collect();

    let padding_lessons: Vec<_> = padding_cards
        .iter()
        .map(|(card_id, study_card)| {
            let view = generator.apply_view(study_card, study_card.is_new(), &mut rand::rng());
            (**card_id, LessonCard::new(**card_id, view, true))
        })
        .collect();

    let mut result = favorite_lessons;
    result.extend(selected_lessons);
    result.extend(padding_lessons);
    result
}

fn resolve_jlpt_level(card: &Card, jlpt_content: &JlptContent) -> Option<JapaneseLevel> {
    jlpt_content.find_level(&card.content_key(), CardType::from(card))
}

/// Distributes `allowed` slots across card types proportionally to
/// `CARD_TYPE_WEIGHTS`, treating the weights as **percentages** (not as a
/// round-robin pattern). Uses the largest-remainder method with a
/// minor-priority rule for leftover slots:
///
/// 1. `raw_t = allowed * w_t / sum_w` (where `sum_w` is **renormalized** over
///    types actually present in `available_by_type`).
/// 2. `floor_t = min(floor(raw_t), available_t)`.
/// 3. `leftover = allowed - sum(floor_t)`.
/// 4. Phase 1 (minor types, non-Vocabulary): candidates with `floor_t <
///    available_t` AND `floor_t < ceil(raw_t)`; sorted by `fract(raw_t)` desc
///    with a random tie-break.
/// 5. Phase 2 (fallback): Vocabulary first, then any remaining type, until
///    leftover is exhausted or availability runs out.
///
/// This fixes the systematic grammar starvation at `daily_load ≤ 9` where the
/// old round-robin pattern (`8V + 1K + 0G`) gave grammar 0 slots every day.
fn compute_type_slots<R: rand::Rng>(
    allowed: usize,
    available_by_type: &HashMap<CardType, usize>,
    rng: &mut R,
) -> HashMap<CardType, usize> {
    if allowed == 0 {
        return HashMap::new();
    }

    // Active types: CARD_TYPE_WEIGHTS entries with available > 0.
    // If a type is missing from the pool, sum_w is renormalized without it.
    let active: Vec<(CardType, usize)> = CARD_TYPE_WEIGHTS
        .iter()
        .copied()
        .filter(|(t, _)| available_by_type.get(t).copied().unwrap_or(0) > 0)
        .collect();
    if active.is_empty() {
        return HashMap::new();
    }

    let sum_w: usize = active.iter().map(|(_, w)| w).sum();

    // raw and floor (with availability cap).
    let mut slots: HashMap<CardType, usize> = HashMap::new();
    let mut raw_map: HashMap<CardType, f64> = HashMap::new();
    let mut sum_floor = 0usize;
    for (t, w) in &active {
        let raw = (allowed as f64) * (*w as f64) / (sum_w as f64);
        raw_map.insert(*t, raw);
        let avail = available_by_type.get(t).copied().unwrap_or(0);
        let floor = (raw.floor() as usize).min(avail);
        slots.insert(*t, floor);
        sum_floor += floor;
    }

    let mut leftover = allowed.saturating_sub(sum_floor);
    if leftover == 0 {
        debug_assert!(slots.values().sum::<usize>() <= allowed);
        return slots;
    }

    // Phase 1: minor (non-Vocabulary) types first.
    // Candidate must still have headroom: floor < available AND floor < ceil(raw).
    let mut minor_candidates: Vec<CardType> = active
        .iter()
        .filter(|(t, _)| *t != CardType::Vocabulary)
        .map(|(t, _)| *t)
        .filter(|t| {
            let cur = slots.get(t).copied().unwrap_or(0);
            let avail = available_by_type.get(t).copied().unwrap_or(0);
            let raw = raw_map.get(t).copied().unwrap_or(0.0);
            cur < avail && cur < raw.ceil() as usize
        })
        .collect();

    // Shuffle then stable-sort by remainder desc: ties keep the random order
    // from the shuffle, giving the random tie-break required by the contract.
    minor_candidates.shuffle(rng);
    minor_candidates.sort_by(|a, b| {
        let rem_a = raw_map.get(a).copied().unwrap_or(0.0).fract();
        let rem_b = raw_map.get(b).copied().unwrap_or(0.0).fract();
        rem_b
            .partial_cmp(&rem_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for t in minor_candidates.iter() {
        if leftover == 0 {
            break;
        }
        let cur = slots.get(t).copied().unwrap_or(0);
        let avail = available_by_type.get(t).copied().unwrap_or(0);
        if cur < avail {
            slots.insert(*t, cur + 1);
            leftover -= 1;
        }
    }

    // Phase 2: Vocabulary first, then any remaining type with headroom.
    if leftover > 0 {
        let phase2_order: Vec<CardType> = {
            let mut v_first: Vec<CardType> = active
                .iter()
                .filter(|(t, _)| *t == CardType::Vocabulary)
                .map(|(t, _)| *t)
                .collect();
            let mut rest: Vec<CardType> = active
                .iter()
                .filter(|(t, _)| *t != CardType::Vocabulary)
                .map(|(t, _)| *t)
                .collect();
            v_first.append(&mut rest);
            v_first
        };
        for t in phase2_order {
            if leftover == 0 {
                break;
            }
            let avail = available_by_type.get(&t).copied().unwrap_or(0);
            while leftover > 0 {
                let cur = slots.get(&t).copied().unwrap_or(0);
                if cur >= avail {
                    break;
                }
                slots.insert(t, cur + 1);
                leftover -= 1;
            }
        }
    }

    debug_assert!(slots.values().sum::<usize>() <= allowed);
    slots
}

/// New cards grouped first by JLPT priority (N5 highest) then by CardType.
/// Used by [`distribute_new_cards`] to walk groups in priority order while
/// keeping per-type queues for the proportional slot allocator.
type GroupedNewCards<'a> =
    BTreeMap<Reverse<u8>, HashMap<CardType, VecDeque<(&'a Ulid, &'a StudyCard)>>>;

/// Picks the new cards for the lesson respecting both the JLPT-level priority
/// (N5 first, then N4, …) and the per-type proportional quota derived from
/// `CARD_TYPE_WEIGHTS`. `allowed` bounds the total returned; each JLPT group
/// consumes up to `min(remaining, group_size)` slots, allocated across types
/// via `compute_type_slots`. Output order: Vocabulary as spine, then Kanji,
/// then Grammar (matches the historical lesson layout — `interleave_core_by_type`
/// reshuffles it later anyway).
fn distribute_new_cards<'a, R: rand::Rng>(
    new_cards: Vec<(&'a Ulid, &'a StudyCard)>,
    jlpt_content: &JlptContent,
    allowed: usize,
    rng: &mut R,
) -> Vec<(&'a Ulid, &'a StudyCard)> {
    debug_assert!(
        new_cards
            .iter()
            .all(|(_, sc)| !matches!(sc.card(), Card::Phrase(_))),
        "phrases must be filtered out before distribute_new_cards (is_core_candidate)"
    );

    if allowed == 0 || new_cards.is_empty() {
        return Vec::new();
    }

    // Reverse: N5(5) → highest priority → first key in BTreeMap.
    let mut groups: GroupedNewCards = BTreeMap::new();
    for card in new_cards {
        let priority = resolve_jlpt_level(card.1.card(), jlpt_content)
            .map(|l| l.as_number())
            .unwrap_or(UNKNOWN_JLPT_PRIORITY);
        groups
            .entry(Reverse(priority))
            .or_default()
            .entry(CardType::from(card.1.card()))
            .or_default()
            .push_back(card);
    }

    let mut result: Vec<(&Ulid, &StudyCard)> = Vec::with_capacity(allowed);
    let mut remaining = allowed;
    for (_, by_type) in groups {
        if remaining == 0 {
            break;
        }
        let group_total: usize = by_type.values().map(|q| q.len()).sum();
        let take = remaining.min(group_total);
        if take == 0 {
            continue;
        }

        let available: HashMap<CardType, usize> =
            by_type.iter().map(|(t, q)| (*t, q.len())).collect();
        let slots = compute_type_slots(take, &available, rng);
        // `take` is clamped to `group_total` above, so `compute_type_slots`
        // must allocate exactly `take` slots. Guards against a future
        // allocator change that would silently shorten `result`.
        debug_assert_eq!(
            slots.values().sum::<usize>(),
            take,
            "compute_type_slots must allocate exactly `take` slots"
        );

        for card_type in [CardType::Vocabulary, CardType::Kanji, CardType::Grammar] {
            if let Some(queue) = by_type.get(&card_type) {
                let n = slots.get(&card_type).copied().unwrap_or(0);
                for card in queue.iter().take(n) {
                    result.push(*card);
                }
            }
        }

        remaining -= take;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::knowledge::LessonCardView;
    use crate::domain::knowledge::{GrammarRuleCard, KanjiCard, PhraseCard, VocabularyCard};
    use crate::domain::value_objects::Question;
    use crate::domain::{RateMode, Rating};
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use rstest::rstest;

    fn vocab_card(word: &str) -> Card {
        Card::Vocabulary(VocabularyCard::new(
            Question::new(word.to_string()).unwrap(),
        ))
    }

    fn phrase_card(phrase_id: Ulid) -> Card {
        Card::Phrase(PhraseCard::new_test_with_id(phrase_id))
    }

    fn make_study_card(card: Card) -> (Ulid, StudyCard) {
        (Ulid::new(), StudyCard::new(card))
    }

    // --- Tail phrase selection (Slice-3) ---
    //
    // The phrase index is a process-wide `OnceLock`; only one index can live in
    // a test binary. These tests reuse the exact 8-phrase fixture also used by
    // `journeys/phrase.rs` and `seed_ready_phrases.rs` so they hold regardless
    // of which module wins the initialization race.

    static PHRASE_INDEX_INIT: std::sync::OnceLock<()> = std::sync::OnceLock::new();

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

    fn phrase_id_particle() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HP").expect("valid ULID")
    }

    fn phrase_id_extra1() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HQ").expect("valid ULID")
    }

    fn phrase_id_extra2() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HR").expect("valid ULID")
    }

    fn ensure_test_phrase_index() {
        PHRASE_INDEX_INIT.get_or_init(|| {
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

    fn full_known_set() -> HashSet<String> {
        ["test", "hello", "bye", "morning", "thanks"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn phrase_tail_eligible_true_when_all_tokens_known() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(phrase_tail_eligible(&phrase_id_hello(), &known));
        assert!(phrase_tail_eligible(&phrase_id_bye(), &known));
    }

    #[test]
    fn phrase_tail_eligible_false_when_token_missing() {
        ensure_test_phrase_index();

        let partial: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
        assert!(!phrase_tail_eligible(&phrase_id_bye(), &partial));
    }

    #[test]
    fn phrase_tail_eligible_false_missing_index_entry() {
        ensure_test_phrase_index();

        let known = full_known_set();
        assert!(!phrase_tail_eligible(&Ulid::new(), &known));
    }

    // --- Prove-It test suite (Slice-1, RED) ---
    //
    // These tests pin the new tail-eligibility and interleaving contracts
    // introduced to fix the post-PR-#188 regression. They drive the Slice-2
    // fix and must remain green afterwards. The contract:
    //   * Tail eligibility ignores grammatical particles and draws the
    //     known-pool from the ENTIRE knowledge_set (not just lesson cards).
    //   * Tail per-word cap is 1 (down from 2).
    //   * Interleaved anchor set is `!is_known_card()` — high-difficulty
    //     vocab is intentionally included.
    //   * There is no global interleaved cap (only per-word cap).

    #[test]
    fn phrase_tail_eligible_ignores_grammatical_particle() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["test"].iter().map(|s| s.to_string()).collect();
        assert!(
            phrase_tail_eligible(&phrase_id_particle(), &known),
            "particle token は must be ignored when judging tail eligibility"
        );
    }

    #[test]
    fn phrase_tail_eligible_rejects_unknown_non_particle_token() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["hello"].iter().map(|s| s.to_string()).collect();
        assert!(
            !phrase_tail_eligible(&phrase_id_extra1(), &known),
            "unknown non-particle token must disqualify the phrase"
        );
    }

    #[test]
    fn phrase_tail_eligible_all_known_vocab_eligible() {
        ensure_test_phrase_index();

        let known: HashSet<String> = ["test", "hello"].iter().map(|s| s.to_string()).collect();
        assert!(
            phrase_tail_eligible(&phrase_id_hello(), &known),
            "phrase whose tokens are all known must be eligible"
        );
    }

    /// Tail eligibility draws its known-pool from the ENTIRE knowledge_set, not
    /// just the lesson cards. A phrase anchored to vocab that lives in the
    /// knowledge_set but is NOT in the current lesson's core must still be
    /// tail-eligible. This is the core regression fix.
    #[test]
    fn phrase_tail_eligible_uses_entire_knowledge_set() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        ks.mark_card_as_known(*test_sc.card_id())
            .expect("mark test known");
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello known");
        ks.create_card(Card::Phrase(
            PhraseCard::new_test_with_id(phrase_id_hello()),
        ))
        .expect("create phrase");

        let lesson = LessonData {
            cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
            core_count: 1,
        };

        let mut budget = 5;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "phrase anchored to known vocab outside the lesson core must still enter the lesson"
        );
    }

    #[test]
    fn tail_phrases_cap_one_per_word() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ]);
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let mut selection = tail_selection(&all_cards, &known, &empty_used);
        let result = collect_phrase_cards(&mut selection);

        assert_eq!(
            result.len(),
            1,
            "Tail phrases sharing a word should be capped at 1 (MAX_PHRASES_PER_WORD_IN_TAIL=1), got {}",
            result.len()
        );
    }

    /// Under Option alpha the no-anchor section ignores `phrase_new_budget`:
    /// a depleted budget must NOT starve no-anchor phrases, whose count is
    /// bounded per lesson by `TAIL_PHRASE_PER_LESSON` (plus the per-word cap)
    /// instead. With `budget = 0` the no-anchor phrases still appear.
    #[test]
    fn no_anchor_phrases_ignore_new_phrase_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        for word in ["test", "hello", "bye", "morning", "thanks"] {
            let sc = ks.create_card(vocab_card(word)).expect("create vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(Card::Phrase(PhraseCard::new_test_with_id(pid)))
                .expect("create phrase");
        }

        // Lesson core holds a word no phrase anchors to, so every selected
        // phrase is no-anchor. Per-word cap (`MAX_PHRASES_PER_WORD_IN_TAIL=1`)
        // admits exactly one (all four share the "test" token).
        let lesson = LessonData {
            cards: vec![(Ulid::new(), lesson_card_for(vocab_card("猫")))],
            core_count: 1,
        };

        let mut zero_budget = 0;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut zero_budget);
        let count = lesson_phrase_ids(&result).len();

        assert_eq!(
            zero_budget, 0,
            "Option alpha: no-anchor phrases must not decrement phrase_new_budget"
        );
        assert!(
            count >= 1,
            "no-anchor phrases must appear even with budget=0, got {count}"
        );
        assert!(
            count <= TAIL_PHRASE_PER_LESSON,
            "no-anchor phrases must respect TAIL_PHRASE_PER_LESSON={TAIL_PHRASE_PER_LESSON}, got {count}"
        );
    }

    // Note: `tail_phrases_preserve_due_then_new_order` previously pinned that
    // due phrases preceded new phrases inside the dedicated tail zone. That zone
    // is gone (Slice 3 merges all phrases into constraint-aware placement), so
    // the due/new ordering no longer exists as a separate contract — phrase
    // order is now driven by the anchor-first-showing constraint, not by due/new
    // scheduling. The test was removed together with the tail zone.

    /// High-difficulty vocab is now a valid interleaving target (the original
    /// purpose of interleaving): a phrase anchored to an HD word must appear.
    #[test]
    fn interleaved_phrases_target_high_difficulty() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let bye_sc = ks.create_card(vocab_card("bye")).expect("create bye");
        for word in ["test", "hello"] {
            ks.create_card(vocab_card(word))
                .expect("create known vocab");
        }
        ks.create_card(phrase_card(phrase_id_bye()))
            .expect("create phrase");

        for _ in 0..3 {
            ks.rate_card(*bye_sc.card_id(), Rating::Again, RateMode::ShortTerm)
                .expect("rate bye");
        }

        let bye_id = *bye_sc.card_id();
        assert!(
            ks.get_card(bye_id).unwrap().memory().is_high_difficulty(),
            "fixture sanity: bye should be high difficulty"
        );

        let lesson = LessonData {
            cards: vec![(bye_id, lesson_card_for(vocab_card("bye")))],
            core_count: 1,
        };

        let mut budget = 5;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_bye()),
            "high-difficulty anchor must receive an interleaved phrase"
        );
    }

    /// No global interleaved cap exists: two anchor words each yield
    /// `INTERLEAVED_PHRASES_PER_WORD` phrases, summing to 4 — proving the
    /// absence of a hidden total ceiling.
    #[test]
    fn interleaved_phrases_no_total_cap_multi_anchor() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_extra1(),
            phrase_id_extra2(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let test_id = *test_sc.card_id();
        let hello_id = *hello_sc.card_id();
        let lesson = LessonData {
            cards: vec![
                (test_id, lesson_card_for(vocab_card("test"))),
                (hello_id, lesson_card_for(vocab_card("hello"))),
            ],
            core_count: 2,
        };

        let mut budget = 10;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.len() >= INTERLEAVED_PHRASES_PER_WORD * 2,
            "two anchors × {} phrases each (no total cap) should yield ≥ {}, got {}",
            INTERLEAVED_PHRASES_PER_WORD,
            INTERLEAVED_PHRASES_PER_WORD * 2,
            phrases.len()
        );
    }

    fn make_phrase_study_cards(phrase_ids: &[Ulid]) -> Vec<(Ulid, StudyCard)> {
        phrase_ids
            .iter()
            .map(|pid| make_study_card(phrase_card(*pid)))
            .collect()
    }

    fn selected_phrase_ids(cards: &[(&Ulid, &StudyCard)]) -> Vec<Ulid> {
        cards
            .iter()
            .filter_map(|(_, sc)| match sc.card() {
                Card::Phrase(p) => Some(*p.phrase_id()),
                _ => None,
            })
            .collect()
    }

    fn empty_ulid_set() -> &'static HashSet<Ulid> {
        static EMPTY: std::sync::OnceLock<HashSet<Ulid>> = std::sync::OnceLock::new();
        EMPTY.get_or_init(HashSet::new)
    }

    fn tail_selection<'a>(
        all_cards: &'a [(&'a Ulid, &'a StudyCard)],
        known: &'a HashSet<String>,
        used: &'a HashSet<Ulid>,
    ) -> TailPhraseSelection<'a> {
        TailPhraseSelection {
            all_cards,
            excluded_card_ids: empty_ulid_set(),
            used_phrase_ids: used,
            known_pool: known,
        }
    }

    #[test]
    fn tail_phrases_contain_only_known_words() {
        ensure_test_phrase_index();

        // One eligible phrase plus one unknown phrase id. The eligible phrase
        // has all-known tokens; the unknown phrase id is absent from the index,
        // so `phrase_tail_eligible` rejects it via `get_index_entry` returning
        // None.
        let owned = make_phrase_study_cards(&[phrase_id_hello(), Ulid::new()]);
        let unknown_phrase = match owned[1].1.card() {
            Card::Phrase(p) => *p.phrase_id(),
            _ => unreachable!("second card is a phrase card"),
        };
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let empty_used = HashSet::new();
        let mut selection = tail_selection(&all_cards, &known, &empty_used);
        let result = collect_phrase_cards(&mut selection);

        let selected = selected_phrase_ids(&result);
        assert!(
            selected.contains(&phrase_id_hello()),
            "phrase with all-known tokens should be selected"
        );
        assert!(
            !selected.contains(&unknown_phrase),
            "phrase whose phrase_id is absent from the index must be excluded"
        );
    }

    #[test]
    fn tail_phrases_exclude_used_phrase_ids() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[phrase_id_hello(), phrase_id_bye()]);
        let all_cards: Vec<(&Ulid, &StudyCard)> = owned.iter().map(|(id, sc)| (id, sc)).collect();

        let known = full_known_set();
        let mut used = HashSet::new();
        used.insert(phrase_id_hello());

        let mut selection = tail_selection(&all_cards, &known, &used);
        let result = collect_phrase_cards(&mut selection);

        let selected = selected_phrase_ids(&result);
        assert!(
            !selected.contains(&phrase_id_hello()),
            "phrase already used by interleaving must not reappear in the tail"
        );
        assert!(selected.contains(&phrase_id_bye()));
    }

    #[test]
    fn init_phrase_index_loads_entries() {
        // The CDN loader (`init_phrase_index_from_cdn`) is process-wide and
        // mutually exclusive with the 8-phrase fixture used across the lib test
        // binary, so loading the real 156k-entry index here would win the
        // OnceLock race and break fixture-based tests. We instead verify the
        // helper is exported and callable, and assert the index contract
        // (entries become retrievable) via the safe fixture path.
        let _helper: fn() = crate::use_cases::init_phrase_index_from_cdn;

        ensure_test_phrase_index();

        assert!(
            crate::dictionary::phrase::is_phrases_loaded(),
            "phrase index should be loaded after init"
        );
        assert!(
            crate::dictionary::phrase::iter_index_entries().is_some(),
            "iter_index_entries should yield entries once the index is loaded"
        );
        assert!(
            crate::dictionary::phrase::get_phrases_by_token("test")
                .iter()
                .any(|e| !e.tokens().is_empty()),
            "get_phrases_by_token should resolve known fixture tokens to entries"
        );
    }

    // --- Interleaving algorithm (H1) ---
    //
    // These tests pin the invariants of the phrase-interleaving pipeline. They
    // intentionally span three levels: the pure layout primitive
    // `interleave_with_gap`, the mid-level orchestrator `add_phrases`
    // / `collect_interleaved_phrases_for_word`, and the full `cards_to_lesson`
    // pipeline. Lower levels isolate a single invariant; the pipeline tests
    // guard the integration of the shared budget.

    fn lesson_card_for(card: Card) -> LessonCard {
        LessonCard::new(Ulid::new(), LessonCardView::Normal(card), false)
    }

    // --- interleave_core_by_type (Slice 2) ---
    //
    // Vocab acts as the separator spine; kanji/grammar are dealt round-robin
    // into the V+1 gaps between vocab cards. The longest kanji-only run is
    // therefore bounded by ⌈K/(V+1)⌉ deterministically (counts only).

    fn slot(card: Card) -> (Ulid, LessonCard) {
        let id = Ulid::new();
        (id, LessonCard::new(id, LessonCardView::Normal(card), false))
    }

    fn build_core_with(vocab_n: usize, kanji_n: usize, grammar_n: usize) -> LessonData {
        let mut cards: Vec<(Ulid, LessonCard)> = Vec::new();
        for i in 0..vocab_n {
            cards.push(slot(vocab_card(&format!("v{i}"))));
        }
        for i in 0..kanji_n {
            cards.push(slot(Card::Kanji(KanjiCard::new_test(format!("k{i}")))));
        }
        for _ in 0..grammar_n {
            cards.push(slot(Card::Grammar(GrammarRuleCard::new_test())));
        }
        let core_count = cards.len();
        LessonData { cards, core_count }
    }

    fn longest_type_run(cards: &[(Ulid, LessonCard)], target: CardType) -> usize {
        let mut best = 0usize;
        let mut current = 0usize;
        for (_, lc) in cards {
            if CardType::from(lc.card()) == target {
                current += 1;
                best = best.max(current);
            } else {
                current = 0;
            }
        }
        best
    }

    #[rstest]
    #[case::vocab_dominant(8, 5, 0)]
    #[case::kanji_heavy(5, 10, 0)]
    #[case::with_grammar(6, 7, 3)]
    #[case::single_kanji(10, 1, 0)]
    #[case::kanji_slightly_more_than_vocab(4, 6, 0)]
    fn interleave_core_bounds_kanji_run_within_ceiling(
        #[case] vocab_n: usize,
        #[case] kanji_n: usize,
        #[case] grammar_n: usize,
    ) {
        let lesson = build_core_with(vocab_n, kanji_n, grammar_n);
        let result = interleave_core_by_type(lesson);

        let ceiling = (kanji_n + vocab_n) / (vocab_n + 1);
        let run = longest_type_run(&result.cards, CardType::Kanji);
        assert!(
            run <= ceiling,
            "V={vocab_n},K={kanji_n},G={grammar_n}: max kanji run {run} exceeds ceiling ⌈K/(V+1)⌉={ceiling}"
        );
    }

    #[test]
    fn interleave_core_grammar_run_also_bounded() {
        let lesson = build_core_with(6, 2, 9);
        let result = interleave_core_by_type(lesson);

        let ceiling = (9 + 6) / (6 + 1);
        let run = longest_type_run(&result.cards, CardType::Grammar);
        assert!(
            run <= ceiling,
            "grammar run {run} exceeds ceiling ⌈G/(V+1)⌉={ceiling}"
        );
    }

    #[test]
    fn interleave_core_preserves_card_set() {
        let lesson = build_core_with(5, 4, 2);
        let before: HashSet<Ulid> = lesson.cards.iter().map(|(id, _)| *id).collect();
        let result = interleave_core_by_type(lesson);
        let after: HashSet<Ulid> = result.cards.iter().map(|(id, _)| *id).collect();

        assert_eq!(
            before, after,
            "interleaving must not add, drop or duplicate cards"
        );
        assert_eq!(result.core_count, before.len());
    }

    #[test]
    fn interleave_core_empty_is_noop() {
        let lesson = LessonData {
            cards: vec![],
            core_count: 0,
        };
        let result = interleave_core_by_type(lesson);
        assert!(result.cards.is_empty());
        assert_eq!(result.core_count, 0);
    }

    #[test]
    fn interleave_core_single_card_is_noop() {
        let lesson = build_core_with(1, 0, 0);
        let only_id = lesson.cards[0].0;
        let result = interleave_core_by_type(lesson);
        assert_eq!(result.cards.len(), 1);
        assert_eq!(result.cards[0].0, only_id);
    }

    #[test]
    fn interleave_core_no_vocab_leaves_layout_untouched() {
        // With no separator spine the core cannot be spread; the original
        // order must be preserved byte-for-byte.
        let lesson = build_core_with(0, 6, 0);
        let original_order: Vec<Ulid> = lesson.cards.iter().map(|(id, _)| *id).collect();
        let result = interleave_core_by_type(lesson);
        let result_order: Vec<Ulid> = result.cards.iter().map(|(id, _)| *id).collect();

        assert_eq!(
            original_order, result_order,
            "an all-kanji core must be left untouched (no separator available)"
        );
    }

    fn lesson_phrase_id(lc: &LessonCard) -> Option<Ulid> {
        match lc.card() {
            Card::Phrase(p) => Some(*p.phrase_id()),
            _ => None,
        }
    }

    fn phrase_cards_map(owned: &[(Ulid, StudyCard)]) -> HashMap<Ulid, (&Ulid, &StudyCard)> {
        owned
            .iter()
            .filter_map(|(id, sc)| match sc.card() {
                Card::Phrase(p) => Some((*p.phrase_id(), (id, sc))),
                _ => None,
            })
            .collect()
    }

    fn lesson_phrase_ids(lesson: &LessonData) -> HashSet<Ulid> {
        lesson
            .cards
            .iter()
            .filter_map(|(_, lc)| lesson_phrase_id(lc))
            .collect()
    }

    /// A phrase anchored to a word must land strictly after it, with at least
    /// `gap` intervening cards when the core is long enough to honour the gap.
    #[test]
    fn interleaved_phrases_placed_after_word_with_gap() {
        let word_id = Ulid::new();
        let phrase_card_id = Ulid::new();

        let mut core_cards: Vec<(Ulid, LessonCard)> = (0..5)
            .map(|_| (Ulid::new(), lesson_card_for(vocab_card("filler"))))
            .collect();
        core_cards[2] = (word_id, lesson_card_for(vocab_card("anchor")));

        let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
            word_id,
            vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
        )]
        .into_iter()
        .collect();

        let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

        let word_pos = result
            .iter()
            .position(|(id, _)| *id == word_id)
            .expect("anchor word present in output");
        let phrase_pos = result
            .iter()
            .position(|(id, _)| *id == phrase_card_id)
            .expect("interleaved phrase present in output");

        assert!(
            phrase_pos > word_pos,
            "phrase must follow its anchor word: {phrase_pos} <= {word_pos}"
        );
        assert!(
            phrase_pos > word_pos + INTERLEAVING_GAP,
            "core had room for the gap: phrase_pos {phrase_pos} should be > {}",
            word_pos + INTERLEAVING_GAP
        );
    }

    /// No single anchor word may pull more than `INTERLEAVED_PHRASES_PER_WORD`
    /// phrases into the core, even when many eligible phrases share its token.
    #[test]
    fn interleaved_phrases_max_two_per_word() {
        ensure_test_phrase_index();

        let owned = make_phrase_study_cards(&[
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ]);
        let map = phrase_cards_map(&owned);

        let in_lesson = HashSet::new();
        let mut used = HashSet::new();
        let mut budget = 10;

        let picked =
            collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

        assert!(
            picked.len() <= INTERLEAVED_PHRASES_PER_WORD,
            "per-word cap violated: {} > {INTERLEAVED_PHRASES_PER_WORD}",
            picked.len()
        );
        assert_eq!(
            picked.len(),
            INTERLEAVED_PHRASES_PER_WORD,
            "four eligible phrases with a deep budget should fill the cap exactly"
        );
    }

    /// Interleaving falls back to known vocab ONLY when no non-known (new /
    /// in-progress / high-difficulty) anchor yields a phrase. The fallback
    /// exists to keep phrases flowing on a fully-mastered lesson — the target
    /// filter `!is_known_card()` covers new, in-progress and high-difficulty
    /// vocab alike (see `interleaved_phrases_target_high_difficulty`), so this
    /// test exercises the residual branch where every core vocab is known.
    #[test]
    fn interleaved_phrases_fallback_to_known() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        for pid in [phrase_id_hello(), phrase_id_bye()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello known");
        ks.mark_card_as_known(*test_sc.card_id())
            .expect("mark test known");

        let hello_id = *hello_sc.card_id();
        let test_id = *test_sc.card_id();
        assert!(ks.get_card(hello_id).unwrap().memory().is_known_card());
        assert!(ks.get_card(test_id).unwrap().memory().is_known_card());

        let lesson = LessonData {
            cards: vec![
                (hello_id, lesson_card_for(vocab_card("hello"))),
                (test_id, lesson_card_for(vocab_card("test"))),
            ],
            core_count: 2,
        };

        let mut budget = 5;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_hello()),
            "fallback should anchor a phrase to a known word when no target vocab yields phrases"
        );
    }

    /// A phrase selected for the lesson must never appear more than once: the
    /// shared `used_phrase_ids` set dedupes the former interleaved/tail pools,
    /// which are now merged into a single placement pass.
    #[test]
    fn phrases_never_duplicated_in_lesson() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let lesson = ks.cards_to_lesson(
            1,
            &JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );

        let mut all_phrase_ids: Vec<Ulid> = lesson
            .cards
            .iter()
            .filter_map(|(_, lc)| lesson_phrase_id(lc))
            .collect();
        let total = all_phrase_ids.len();
        all_phrase_ids.sort();
        all_phrase_ids.dedup();
        assert!(
            !all_phrase_ids.is_empty(),
            "scenario should place at least one phrase"
        );
        assert_eq!(
            all_phrase_ids.len(),
            total,
            "no phrase id may appear more than once in the lesson"
        );
    }

    /// After merging, every phrase sits inside the core section and there is no
    /// dedicated tail zone: `core_count` equals the lesson length.
    #[test]
    fn phrases_merged_into_core_no_tail_zone() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let lesson = ks.cards_to_lesson(
            1,
            &JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );

        let phrases_in_lesson: usize = lesson
            .cards
            .iter()
            .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
            .count();
        assert!(
            phrases_in_lesson >= 1,
            "scenario should place at least one phrase"
        );
        assert_eq!(
            lesson.core_count,
            lesson.cards.len(),
            "all cards (phrases included) must be part of the core — no dedicated tail zone"
        );
    }

    /// On a lesson too short to honour the gap, the phrase is flushed at the
    /// end. The only inviolable ordering rule — phrase after its anchor — holds.
    #[test]
    fn interleaved_phrases_small_lesson_degrades_gap() {
        let word_id = Ulid::new();
        let phrase_card_id = Ulid::new();

        let core_cards: Vec<(Ulid, LessonCard)> =
            [(word_id, lesson_card_for(vocab_card("solo")))].to_vec();
        let assignments: HashMap<Ulid, Vec<(Ulid, LessonCard)>> = [(
            word_id,
            vec![(phrase_card_id, lesson_card_for(phrase_card(Ulid::new())))],
        )]
        .into_iter()
        .collect();

        let result = interleave_with_gap(core_cards, assignments, INTERLEAVING_GAP);

        let word_pos = result
            .iter()
            .position(|(id, _)| *id == word_id)
            .expect("anchor present");
        let phrase_pos = result
            .iter()
            .position(|(id, _)| *id == phrase_card_id)
            .expect("flushed phrase present");

        assert_eq!(result.len(), 2);
        assert!(phrase_pos > word_pos);
        assert!(
            phrase_pos < word_pos + INTERLEAVING_GAP + 1,
            "gap should degrade on a tiny core, but ordering must survive"
        );
    }

    /// Under Option alpha the new-phrase budget bounds only NEW ANCHORED
    /// phrases; NEW no-anchor phrases are additive (capped per lesson by
    /// `TAIL_PHRASE_PER_LESSON`). The combined new-phrase count therefore may
    /// reach `budget + TAIL_PHRASE_PER_LESSON` but must not exceed it. Due
    /// phrases are free on both sides.
    #[test]
    fn new_phrases_respect_option_alpha_combined_ceiling() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(vocab_card("hello")).expect("create hello");
        for word in ["test", "bye", "morning", "thanks"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let daily_new_limit = 1;
        let budget = compute_phrase_new_budget(daily_new_limit, 0);
        assert_eq!(budget, 2, "fixture sanity: PHRASE_NEW_RATIO=2");

        let lesson = ks.cards_to_lesson(
            daily_new_limit,
            &JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );

        let new_phrases_in_lesson = lesson
            .cards
            .iter()
            .filter(|(_, lc)| lesson_phrase_id(lc).is_some())
            .filter(|(id, _)| ks.get_card(*id).is_some_and(|sc| sc.memory().is_new()))
            .count();

        let combined_ceiling = budget + TAIL_PHRASE_PER_LESSON;
        assert!(
            new_phrases_in_lesson <= combined_ceiling,
            "new phrases (anchored ≤ budget + no-anchor ≤ TAIL_PHRASE_PER_LESSON) must respect \
             the Option-alpha combined ceiling: {new_phrases_in_lesson} > {combined_ceiling}"
        );
        assert!(
            new_phrases_in_lesson >= 1,
            "scenario should place at least one new phrase"
        );
    }

    /// Due phrases enter the interleaved section for free: they do not decrement
    /// `phrase_new_budget`, which stays reserved for new phrases.
    #[test]
    fn interleaved_phrases_due_do_not_consume_new_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        let hello_sc = ks
            .create_card(phrase_card(phrase_id_hello()))
            .expect("create hello");
        for pid in [phrase_id_bye(), phrase_id_morning(), phrase_id_thanks()] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        // mark_card_as_known schedules next_review in the past, so the phrase
        // becomes due (and not new) without going through a timed review cycle.
        ks.mark_card_as_known(*hello_sc.card_id())
            .expect("mark hello due");
        let hello_card = ks.get_card(*hello_sc.card_id()).unwrap();
        assert!(hello_card.memory().is_due());
        assert!(!hello_card.memory().is_new());

        let owned: Vec<(Ulid, StudyCard)> = ks
            .study_cards()
            .iter()
            .map(|(id, sc)| (*id, sc.clone()))
            .collect();
        let map = phrase_cards_map(&owned);

        let in_lesson = HashSet::new();
        let mut used = HashSet::new();
        let mut budget = 2;
        let initial_budget = budget;

        let picked =
            collect_interleaved_phrases_for_word("test", &map, &in_lesson, &mut used, &mut budget);

        let picked_due_hello = picked.iter().any(|(id, _)| *id == *hello_sc.card_id());
        assert!(
            picked_due_hello,
            "due phrase must be picked for free by the due pass"
        );

        let new_picked = picked.iter().filter(|(_, sc)| sc.memory().is_new()).count();
        assert_eq!(
            budget,
            initial_budget - new_picked,
            "only new phrases consume budget; the due phrase must be free"
        );
    }

    // --- Constraint-aware placement (Slice 3) ---
    //
    // A phrase that references a lesson-vocab word must appear AFTER the first
    // showing of every such word, otherwise it leaks the answer into the word's
    // standalone FSRS rating. These tests pin that invariant end-to-end.

    fn first_showing_positions(lesson: &LessonData) -> HashMap<Ulid, usize> {
        let mut positions: HashMap<Ulid, usize> = HashMap::new();
        for (i, (slot_id, _)) in lesson.cards.iter().enumerate() {
            positions.entry(*slot_id).or_insert(i);
        }
        positions
    }

    /// Asserts every phrase in `lesson` follows the first showing of each of its
    /// anchor words that are present in `anchor_vocab` (word -> slot id).
    fn assert_phrases_follow_anchors(
        lesson: &LessonData,
        anchor_vocab: &[(&Ulid, &str)],
        positions: &HashMap<Ulid, usize>,
    ) {
        let word_to_slot: HashMap<&str, Ulid> =
            anchor_vocab.iter().map(|(id, w)| (*w, **id)).collect();
        for (phrase_idx, (_, lc)) in lesson.cards.iter().enumerate() {
            let Some(phrase_id) = lesson_phrase_id(lc) else {
                continue;
            };
            let tokens: &[String] = crate::dictionary::phrase::get_index_entry(&phrase_id)
                .map(|e| e.tokens())
                .unwrap_or(&[]);
            for token in tokens {
                if let Some(anchor_slot) = word_to_slot.get(token.as_str()) {
                    let anchor_pos = positions.get(anchor_slot).copied().unwrap_or(usize::MAX);
                    assert!(
                        phrase_idx > anchor_pos,
                        "phrase {phrase_id} at index {phrase_idx} must follow anchor '{token}' \
                         (slot {anchor_slot}) first shown at {anchor_pos}"
                    );
                }
            }
        }
    }

    fn build_vocab_core(words: &[&str]) -> Vec<(Ulid, LessonCard)> {
        words
            .iter()
            .map(|w| {
                let slot_id = Ulid::new();
                (slot_id, lesson_card_for(vocab_card(w)))
            })
            .collect()
    }

    fn vocab_slots_of(cards: &[(Ulid, LessonCard)]) -> Vec<(Ulid, String)> {
        cards
            .iter()
            .filter_map(|(id, lc)| match lc.card() {
                Card::Vocabulary(v) => Some((*id, v.word().text().to_string())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn phrase_appears_after_its_anchor_word_first_showing() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        ks.create_card(phrase_card(phrase_id_hello()))
            .expect("create phrase hello"); // tokens [test, hello]
        ks.create_card(phrase_card(phrase_id_bye()))
            .expect("create phrase bye"); // tokens [test, bye]

        let core_cards = build_vocab_core(&["test", "fill1", "hello", "fill2"]);
        let core_vocab = vocab_slots_of(&core_cards);
        let lesson = LessonData {
            cards: core_cards,
            core_count: 4,
        };

        let mut budget = 10;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

        let phrase_ids = lesson_phrase_ids(&result);
        assert!(
            phrase_ids.contains(&phrase_id_hello()),
            "fixture sanity: phrase hello should be selected"
        );

        let positions = first_showing_positions(&result);
        let anchor_vocab: Vec<(&Ulid, &str)> =
            core_vocab.iter().map(|(id, w)| (id, w.as_str())).collect();
        assert_phrases_follow_anchors(&result, &anchor_vocab, &positions);
    }

    #[test]
    fn constraint_holds_across_many_anchors_and_phrases() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
            phrase_id_extra1(),
            phrase_id_extra2(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let core_cards = build_vocab_core(&[
            "test", "hello", "bye", "morning", "thanks", "fill1", "fill2",
        ]);
        let core_vocab = vocab_slots_of(&core_cards);
        let lesson = LessonData {
            cards: core_cards,
            core_count: 7,
        };

        let mut budget = 20;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

        let positions = first_showing_positions(&result);
        let anchor_vocab: Vec<(&Ulid, &str)> =
            core_vocab.iter().map(|(id, w)| (id, w.as_str())).collect();
        assert_phrases_follow_anchors(&result, &anchor_vocab, &positions);
    }

    #[test]
    fn anchorless_phrases_are_distributed_not_clustered_at_end() {
        ensure_test_phrase_index();

        // ULID of the fixture phrase whose tokens are ["alpha", "beta"] — both
        // deliberately absent from the lesson core, making the phrase anchorless.
        let phrase_independent =
            Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HS").expect("valid ULID");

        let mut ks = KnowledgeSet::new();
        // "alpha"/"beta" are NOT in the lesson core → this phrase has no anchor
        // and must be distributed rather than dumped at the end.
        ks.create_card(phrase_card(phrase_independent))
            .expect("create independent phrase");
        // a second anchorless phrase for a stronger distribution check
        ks.create_card(phrase_card(phrase_id_extra1()))
            .expect("create extra1 phrase");

        // Mark all tokens known so tail eligibility admits them.
        for word in ["alpha", "beta", "hello", "extra1"] {
            let sc = ks.create_card(vocab_card(word)).expect("create vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }

        // Lesson core deliberately contains none of the phrase tokens.
        let core_cards = build_vocab_core(&["v0", "v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8"]);
        let core_len = core_cards.len();
        let lesson = LessonData {
            cards: core_cards,
            core_count: core_len,
        };

        let mut budget = 10;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);

        let phrase_indices: Vec<usize> = result
            .cards
            .iter()
            .enumerate()
            .filter(|(_, (_, lc))| lesson_phrase_id(lc).is_some())
            .map(|(i, _)| i)
            .collect();
        assert!(!phrase_indices.is_empty(), "at least one phrase expected");

        // The last third of the lesson must not contain ALL phrases — that would
        // reproduce the dedicated end zone the merge set out to remove.
        let tail_start = result.cards.len() * 2 / 3;
        let phrases_before_tail = phrase_indices.iter().filter(|&&i| i < tail_start).count();
        assert!(
            phrases_before_tail > 0,
            "anchorless phrases must be distributed: found {phrases_before_tail} before the last third, indices {phrase_indices:?}"
        );
    }

    #[test]
    fn constraint_survives_multi_show_expansion() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        // Anchor word rated hard → multi-show expansion will repeat it. The
        // phrase anchored to it must still land after the FIRST showing.
        let anchor_sc = ks.create_card(vocab_card("test")).expect("create anchor");
        for _ in 0..3 {
            ks.rate_card(*anchor_sc.card_id(), Rating::Again, RateMode::ShortTerm)
                .expect("rate anchor hard");
        }
        for w in ["hello", "bye", "fill1", "fill2", "fill3"] {
            ks.create_card(vocab_card(w)).expect("create vocab");
        }
        ks.create_card(phrase_card(phrase_id_hello()))
            .expect("create phrase"); // tokens [test, hello]

        let lesson = ks.cards_to_lesson(
            5,
            &JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );

        let anchor_id = *anchor_sc.card_id();
        let anchor_showings: Vec<usize> = lesson
            .cards
            .iter()
            .enumerate()
            .filter(|(_, (_, lc))| lc.card_id() == anchor_id)
            .map(|(i, _)| i)
            .collect();
        let first_anchor = anchor_showings.first().copied();
        let phrase_pos = lesson
            .cards
            .iter()
            .enumerate()
            .find(|(_, (_, lc))| lesson_phrase_id(lc) == Some(phrase_id_hello()))
            .map(|(i, _)| i);

        if let (Some(first), Some(php)) = (first_anchor, phrase_pos) {
            assert!(
                php > first,
                "phrase must follow the first showing of its anchor even after expansion: \
                 first={first}, phrase={php}"
            );
        }
    }

    // --- Multi-show expansion ---
    //
    // These tests pin the contract that a primary card is shown multiple
    // times (in distinct views) when its FSRS state demands it, while
    // companions, phrases and known cards keep a single showing.

    use crate::domain::memory::{Difficulty, MemoryState, ReviewLog, Stability};
    use chrono::{Duration, Utc};

    fn init_test_dict() {
        crate::use_cases::init_real_dictionaries();
    }

    fn rate_into_state(
        ks: &mut KnowledgeSet,
        card_id: Ulid,
        stability: f64,
        difficulty: f64,
        interval_days: i64,
        rating: Rating,
    ) {
        let memory = MemoryState::new(
            Stability::new(stability).unwrap(),
            Difficulty::new(difficulty).unwrap(),
            Utc::now() - Duration::days(interval_days),
        );
        let study_card = ks.study_cards_mut_for_test().get_mut(&card_id).unwrap();
        study_card.add_review(memory, ReviewLog::new(rating, Duration::days(1)));
    }

    fn seed_distractor_vocab(ks: &mut KnowledgeSet, words: &[&str]) {
        for word in words {
            let _ = ks.create_card(vocab_card(word));
        }
    }

    fn build_lesson_with_one_primary_vocab(ks: &KnowledgeSet, primary_id: Ulid) -> LessonData {
        let lesson_card = lesson_card_for(vocab_card("anchor"));
        let lesson = LessonData {
            cards: vec![(
                primary_id,
                LessonCard::new(primary_id, lesson_card.into_view(), false),
            )],
            core_count: 1,
        };
        let primary_set: HashSet<Ulid> = [primary_id].into_iter().collect();
        expand_repeated_views(lesson, ks, NativeLanguage::Russian, &primary_set)
    }

    #[test]
    fn expand_hard_primary_vocab_yields_multiple_showings() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).unwrap();
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 3.0, 8.0, 1, Rating::Hard);
        assert!(ks.get_card(card_id).unwrap().memory().is_high_difficulty());

        let result = build_lesson_with_one_primary_vocab(&ks, card_id);
        let showings = result.find_by_card_id(card_id);
        assert!(
            showings.len() >= 2,
            "HD primary vocab should produce at least 2 showings, got {}",
            showings.len()
        );
    }

    #[test]
    fn expand_in_progress_primary_vocab_yields_multiple_showings() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).unwrap();
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 10.0, 4.0, 1, Rating::Good);
        assert!(ks.get_card(card_id).unwrap().memory().is_in_progress());

        let result = build_lesson_with_one_primary_vocab(&ks, card_id);
        let showings = result.find_by_card_id(card_id);
        assert!(
            showings.len() >= 2,
            "in-progress primary vocab should produce at least 2 showings, got {}",
            showings.len()
        );
    }

    #[test]
    fn expand_known_primary_vocab_preserves_single_showing() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).unwrap();
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 30.0, 3.0, 1, Rating::Easy);
        assert!(ks.get_card(card_id).unwrap().memory().is_known_card());

        let result = build_lesson_with_one_primary_vocab(&ks, card_id);
        let showings = result.find_by_card_id(card_id);
        assert_eq!(
            showings.len(),
            1,
            "known primary vocab should keep a single showing"
        );
    }

    #[test]
    fn expand_companion_vocab_keeps_single_showing() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let primary_sc = ks.create_card(vocab_card("猫")).unwrap();
        let primary_id = *primary_sc.card_id();
        rate_into_state(&mut ks, primary_id, 3.0, 8.0, 1, Rating::Hard);

        let companion_sc = ks.create_card(vocab_card("虎")).unwrap();
        let companion_id = *companion_sc.card_id();
        rate_into_state(&mut ks, companion_id, 3.0, 8.0, 1, Rating::Hard);
        assert!(
            ks.get_card(companion_id)
                .unwrap()
                .memory()
                .is_high_difficulty()
        );

        let primary_view = LessonCardView::Normal(vocab_card("猫"));
        let companion_view = LessonCardView::Normal(vocab_card("虎"));
        let lesson = LessonData {
            cards: vec![
                (primary_id, LessonCard::new(primary_id, primary_view, false)),
                (
                    companion_id,
                    LessonCard::new(companion_id, companion_view, false),
                ),
            ],
            core_count: 2,
        };
        let primary_set: HashSet<Ulid> = [primary_id].into_iter().collect();
        let result = expand_repeated_views(lesson, &ks, NativeLanguage::Russian, &primary_set);

        let companion_showings = result.find_by_card_id(companion_id);
        assert_eq!(
            companion_showings.len(),
            1,
            "companion card (not in primary set) must not be expanded even when HD"
        );
    }

    #[test]
    fn expand_phrase_slot_keeps_single_showing() {
        ensure_test_phrase_index();
        let mut ks = KnowledgeSet::new();
        let phrase_sc = ks.create_card(phrase_card(phrase_id_hello())).unwrap();
        let phrase_id = *phrase_sc.card_id();

        let phrase_view = LessonCardView::Normal(phrase_card(phrase_id_hello()));
        let lesson = LessonData {
            cards: vec![(phrase_id, LessonCard::new(phrase_id, phrase_view, false))],
            core_count: 1,
        };
        let primary_set: HashSet<Ulid> = [phrase_id].into_iter().collect();
        let result = expand_repeated_views(lesson, &ks, NativeLanguage::Russian, &primary_set);

        let showings = result.find_by_card_id(phrase_id);
        assert_eq!(
            showings.len(),
            1,
            "phrase slot must never be expanded even if listed in primary_card_ids"
        );
    }

    #[test]
    fn expand_each_showing_uses_distinct_view_type() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).unwrap();
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 3.0, 8.0, 1, Rating::Hard);

        let result = build_lesson_with_one_primary_vocab(&ks, card_id);
        let showings = result.find_by_card_id(card_id);
        let discriminants: HashSet<std::mem::Discriminant<LessonCardView>> = showings
            .iter()
            .map(|lc| std::mem::discriminant(lc.view()))
            .collect();
        assert_eq!(
            discriminants.len(),
            showings.len(),
            "every showing of a multi-show card must use a distinct LessonCardView variant"
        );
    }

    #[test]
    fn expand_preserves_review_card_view_variety() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛", "虎", "狼", "鹿"]);

        let words = ["猫", "狗", "猪", "豚"];
        let mut cards: Vec<(Ulid, LessonCard)> = Vec::new();
        let mut primary_set: HashSet<Ulid> = HashSet::new();
        for word in words {
            let sc = ks.create_card(vocab_card(word)).expect("seed primary card");
            let card_id = *sc.card_id();
            rate_into_state(&mut ks, card_id, 10.0, 4.0, 1, Rating::Good);
            assert!(
                ks.get_card(card_id)
                    .expect("card exists")
                    .memory()
                    .is_in_progress(),
                "fixture card must be expandable review vocab"
            );
            cards.push((
                card_id,
                LessonCard::new(card_id, LessonCardView::Normal(vocab_card(word)), false),
            ));
            primary_set.insert(card_id);
        }
        let lesson = LessonData {
            cards,
            core_count: words.len(),
        };

        let result = expand_repeated_views(lesson, &ks, NativeLanguage::Russian, &primary_set);

        let has_normal = result
            .cards
            .iter()
            .any(|(_, lc)| matches!(lc.view(), LessonCardView::Normal(_)));
        let has_non_normal = result
            .cards
            .iter()
            .any(|(_, lc)| !matches!(lc.view(), LessonCardView::Normal(_)));
        assert!(
            has_normal,
            "lesson must keep the Normal primary (apply_view result) for review vocab, \
             not force-convert every card to the same quiz variant"
        );
        assert!(
            has_non_normal,
            "lesson must keep at least one non-Normal multi-show variant"
        );
    }

    #[test]
    fn expand_increments_core_count_by_added_copies() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).unwrap();
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 3.0, 8.0, 1, Rating::Hard);

        let original_view = LessonCardView::Normal(vocab_card("猫"));
        let lesson = LessonData {
            cards: vec![(card_id, LessonCard::new(card_id, original_view, false))],
            core_count: 1,
        };
        let primary_set: HashSet<Ulid> = [card_id].into_iter().collect();
        let result = expand_repeated_views(lesson, &ks, NativeLanguage::Russian, &primary_set);

        let added = result.find_by_card_id(card_id).len() - 1;
        assert_eq!(
            result.core_count,
            1 + added,
            "core_count must equal original core_count + added copies: {} vs 1 + {}",
            result.core_count,
            added
        );
    }

    #[test]
    fn expand_preserves_tail_phrases_at_end() {
        ensure_test_phrase_index();
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let hd_sc = ks.create_card(vocab_card("猫")).unwrap();
        let hd_id = *hd_sc.card_id();
        rate_into_state(&mut ks, hd_id, 3.0, 8.0, 1, Rating::Hard);

        let phrase_sc = ks.create_card(phrase_card(phrase_id_hello())).unwrap();
        let phrase_slot = *phrase_sc.card_id();

        let original = LessonData {
            cards: vec![
                (
                    hd_id,
                    LessonCard::new(hd_id, LessonCardView::Normal(vocab_card("猫")), false),
                ),
                (
                    phrase_slot,
                    LessonCard::new(
                        phrase_slot,
                        LessonCardView::Normal(phrase_card(phrase_id_hello())),
                        false,
                    ),
                ),
            ],
            core_count: 1,
        };
        let primary_set: HashSet<Ulid> = [hd_id].into_iter().collect();
        let result = expand_repeated_views(original, &ks, NativeLanguage::Russian, &primary_set);

        for (_, lc) in result.cards[result.core_count..].iter() {
            assert!(
                matches!(lc.card(), Card::Phrase(_)),
                "everything past core_count must be a (tail) phrase card"
            );
        }
        let hd_showings_in_core = result.cards[..result.core_count]
            .iter()
            .filter(|(_, lc)| lc.card_id() == hd_id)
            .count();
        assert!(
            hd_showings_in_core >= 2,
            "HD primary must be expanded inside the core section"
        );
    }

    #[test]
    fn expand_enforces_min_spacing_between_consecutive_showings() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛", "虎", "狼", "鹿"]);
        let hd_sc = ks.create_card(vocab_card("猫")).unwrap();
        let hd_id = *hd_sc.card_id();
        rate_into_state(&mut ks, hd_id, 3.0, 8.0, 1, Rating::Hard);

        let distractor_ids: Vec<Ulid> = ks
            .study_cards()
            .iter()
            .filter(|(id, _)| **id != hd_id)
            .map(|(id, _)| *id)
            .collect();
        let mut cards: Vec<(Ulid, LessonCard)> = vec![(
            hd_id,
            LessonCard::new(hd_id, LessonCardView::Normal(vocab_card("猫")), false),
        )];
        for id in &distractor_ids {
            cards.push((
                *id,
                LessonCard::new(*id, LessonCardView::Normal(vocab_card("filler")), false),
            ));
        }
        let core_count = cards.len();
        let original = LessonData { cards, core_count };
        let primary_set: HashSet<Ulid> = [hd_id].into_iter().collect();
        let result = expand_repeated_views(original, &ks, NativeLanguage::Russian, &primary_set);

        let positions: Vec<usize> = result
            .cards
            .iter()
            .enumerate()
            .filter(|(_, (_, lc))| lc.card_id() == hd_id)
            .map(|(pos, _)| pos)
            .collect();
        assert!(
            positions.len() >= 2,
            "expected at least 2 showings of HD card, got {}",
            positions.len()
        );

        for adjacent in positions.windows(2) {
            let positions_apart = adjacent[1] - adjacent[0];
            let cards_between = positions_apart - 1;
            assert!(
                cards_between >= MIN_REPEAT_SPACING,
                "consecutive showings of the same card_id must have at least {} cards between them, got {} (positions apart = {})",
                MIN_REPEAT_SPACING,
                cards_between,
                positions_apart
            );
        }
    }

    // --- Flush-path spacing (Common-1) ---
    //
    // The main-loop drain already honours MIN_REPEAT_SPACING whenever the
    // core has enough buffer cards after the anchor (covered by
    // `expand_enforces_min_spacing_between_consecutive_showings`). The
    // flush path distributes the leftover copies; these tests pin its
    // contract: spacing is guaranteed when the lesson can absorb the
    // copies, and degrades to best-effort on a structurally too-short
    // lesson (anchor at the very end of a small core, or copies
    // outnumbering buffer cards).

    /// Builds a lesson whose anchor sits at the LAST core slot, so every
    /// extra view falls through to the flush path. Combined with a deep
    /// distractor block placed BEFORE the anchor, the flush path still
    /// has zero cards after the anchor to use as buffer — the only
    /// layout it cannot space. Used to assert the best-effort fallback.
    fn build_lesson_with_anchor_last(ks: &KnowledgeSet, anchor_id: Ulid) -> LessonData {
        let distractor_ids: Vec<Ulid> = ks
            .study_cards()
            .iter()
            .filter(|(id, _)| **id != anchor_id)
            .map(|(id, _)| *id)
            .collect();
        let mut cards: Vec<(Ulid, LessonCard)> = Vec::new();
        for id in &distractor_ids {
            cards.push((
                *id,
                LessonCard::new(*id, LessonCardView::Normal(vocab_card("filler")), false),
            ));
        }
        cards.push((
            anchor_id,
            LessonCard::new(anchor_id, LessonCardView::Normal(vocab_card("猫")), false),
        ));
        let core_count = cards.len();
        let original = LessonData { cards, core_count };
        let primary_set: HashSet<Ulid> = [anchor_id].into_iter().collect();
        expand_repeated_views(original, ks, NativeLanguage::Russian, &primary_set)
    }

    fn showing_positions(result: &LessonData, card_id: Ulid) -> Vec<usize> {
        result
            .cards
            .iter()
            .enumerate()
            .filter(|(_, (_, lc))| lc.card_id() == card_id)
            .map(|(pos, _)| pos)
            .collect()
    }

    /// Best-effort fallback: a single-card core whose HD target forces 2
    /// showings cannot honour MIN_REPEAT_SPACING by construction (need
    /// 1 + MIN_REPEAT_SPACING + 1 slots, have 1). The contract degrades
    /// gracefully: copies are still emitted so the learner drills the
    /// card, every copy follows the anchor, and the anchor keeps the
    /// first slot.
    #[rstest]
    fn expand_short_lesson_best_effort_when_lesson_too_small() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        // Distractors live in the knowledge_set so the view generator
        // can produce the distinct candidate views needed for multi-show
        // expansion, but they are deliberately NOT part of the lesson
        // core — this exercises the degenerate single-slot-core path.
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let sc = ks.create_card(vocab_card("猫")).expect("create anchor");
        let card_id = *sc.card_id();
        rate_into_state(&mut ks, card_id, 3.0, 8.0, 1, Rating::Hard);
        assert!(
            ks.get_card(card_id)
                .is_some_and(|sc| sc.memory().is_high_difficulty()),
            "fixture sanity: anchor must be HD"
        );

        let result = build_lesson_with_one_primary_vocab(&ks, card_id);
        let positions = showing_positions(&result, card_id);

        assert!(
            positions.len() >= 2,
            "expected ≥2 showings of HD anchor, got {}",
            positions.len()
        );
        assert_eq!(positions[0], 0, "primary anchor must occupy the first slot");
        for &pos in &positions[1..] {
            assert!(
                pos > positions[0],
                "every copy must follow the anchor: copy at {pos} <= {}",
                positions[0]
            );
        }
    }

    /// Anchor placed at the LAST core slot of a deep distractor block:
    /// there are no cards after the anchor, so spacing is structurally
    /// unreachable. This pins the best-effort fallback on a realistic
    /// multi-distractor lesson (mirrors the Common-1 edge case in the
    /// review: anchor at end of core).
    #[rstest]
    fn expand_anchor_at_core_end_falls_back_to_best_effort() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛", "虎", "狼", "鹿"]);
        let sc = ks.create_card(vocab_card("猫")).expect("create anchor");
        let anchor_id = *sc.card_id();
        rate_into_state(&mut ks, anchor_id, 3.0, 8.0, 1, Rating::Hard);
        assert!(
            ks.get_card(anchor_id)
                .is_some_and(|sc| sc.memory().is_high_difficulty()),
            "fixture sanity: anchor must be HD"
        );

        let result = build_lesson_with_anchor_last(&ks, anchor_id);
        let positions = showing_positions(&result, anchor_id);

        assert!(
            positions.len() >= 2,
            "expected ≥2 showings of HD anchor, got {}",
            positions.len()
        );
        let last_core_idx = result.cards.len() - 1;
        assert!(
            positions[0] >= last_core_idx.saturating_sub(positions.len()),
            "anchor must sit at the end of the core section (positions = {positions:?}, lesson len = {})",
            result.cards.len(),
        );
        for adjacent in positions.windows(2) {
            let positions_apart = adjacent[1] - adjacent[0];
            assert!(
                positions_apart >= 1,
                "every copy must strictly follow the previous showing (got adjacent delta {positions_apart})"
            );
        }
    }

    /// Positive spacing contract on the flush path: when several cards
    /// leave copies to the flush path, the distributor interleaves them
    /// (instead of blindly appending all copies of one card before the
    /// next). Two HD anchors placed at the end of the core leave no
    /// buffer after them, so full MIN_REPEAT_SPACING is mathematically
    /// unreachable — but consecutive copies of the SAME card_id still
    /// never land back-to-back, because copies of the other anchor
    /// sit between them. This is the structural improvement over the
    /// naive append loop: same-card showings are separated by other
    /// cards whenever the flush path holds more than one card_id.
    #[rstest]
    fn expand_flush_path_interleaves_copies_of_distinct_cards() {
        init_test_dict();
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛"]);
        let a_sc = ks.create_card(vocab_card("猫")).expect("create anchor A");
        let b_sc = ks.create_card(vocab_card("虎")).expect("create anchor B");
        let a_id = *a_sc.card_id();
        let b_id = *b_sc.card_id();
        rate_into_state(&mut ks, a_id, 3.0, 8.0, 1, Rating::Hard);
        rate_into_state(&mut ks, b_id, 3.0, 8.0, 1, Rating::Hard);
        assert!(
            ks.get_card(a_id)
                .is_some_and(|sc| sc.memory().is_high_difficulty())
                && ks
                    .get_card(b_id)
                    .is_some_and(|sc| sc.memory().is_high_difficulty()),
            "fixture sanity: both anchors must be HD"
        );

        let distractor_ids: Vec<Ulid> = ks
            .study_cards()
            .iter()
            .filter(|(id, _)| **id != a_id && **id != b_id)
            .map(|(id, _)| *id)
            .collect();
        let mut cards: Vec<(Ulid, LessonCard)> = Vec::new();
        for id in &distractor_ids {
            cards.push((
                *id,
                LessonCard::new(*id, LessonCardView::Normal(vocab_card("filler")), false),
            ));
        }
        cards.push((
            a_id,
            LessonCard::new(a_id, LessonCardView::Normal(vocab_card("猫")), false),
        ));
        cards.push((
            b_id,
            LessonCard::new(b_id, LessonCardView::Normal(vocab_card("虎")), false),
        ));
        let core_count = cards.len();
        let original = LessonData { cards, core_count };
        let primary_set: HashSet<Ulid> = [a_id, b_id].into_iter().collect();
        let result = expand_repeated_views(original, &ks, NativeLanguage::Russian, &primary_set);

        for card_id in [a_id, b_id] {
            let positions = showing_positions(&result, card_id);
            assert!(
                positions.len() >= 2,
                "expected ≥2 showings of HD anchor {card_id}, got {}",
                positions.len()
            );
            for adjacent in positions.windows(2) {
                assert!(
                    adjacent[1] - adjacent[0] > 1,
                    "flush path must interleave copies of distinct cards: same-card showings of {card_id} must not be back-to-back (positions = {positions:?})"
                );
            }
        }
    }

    // --- deal_by_card_id unit contract (Symptom 1, layout primitive) ---
    //
    // Pins the spacing invariant of the layout primitive directly, at the
    // content level (no phrases masking it), including the lone-multi-show-card
    // review-lesson shape that defeated the earlier round-robin deal.

    /// Builds content slots from a `(card_id, showing_count)` spec, in view
    /// order (primary first). Each showing uses a distinct `LessonCardView`
    /// variant so view-order preservation is observable.
    fn build_content_spec(spec: &[(Ulid, usize)]) -> Vec<(Ulid, LessonCard)> {
        let mut out = Vec::new();
        for (card_id, count) in spec {
            for j in 0..*count {
                let card = vocab_card(&format!("c{card_id}{j}"));
                let view = if j == 0 {
                    LessonCardView::Normal(card)
                } else {
                    LessonCardView::Reversed(card)
                };
                out.push((Ulid::new(), LessonCard::new(*card_id, view, false)));
            }
        }
        out
    }

    fn content_min_gap(content: &[(Ulid, LessonCard)], card_id: Ulid) -> Option<usize> {
        let positions: Vec<usize> = content
            .iter()
            .enumerate()
            .filter(|(_, (_, lc))| lc.card_id() == card_id)
            .map(|(i, _)| i)
            .collect();
        if positions.len() < 2 {
            return None;
        }
        positions.windows(2).map(|w| w[1] - w[0] - 1).min()
    }

    /// The case that beat round-robin: a single multi-show card (3 showings)
    /// surrounded by single-show fillers — a normal review lesson with one
    /// hard card. Its copies must still stay `MIN_REPEAT_SPACING` apart.
    #[test]
    fn deal_spreads_lone_multishow_card() {
        let hard = Ulid::new();
        let mut spec: Vec<(Ulid, usize)> = vec![(hard, 3)];
        for _ in 0..14 {
            spec.push((Ulid::new(), 1));
        }
        let content = build_content_spec(&spec);

        let dealt = deal_by_card_id(content);

        let gap = content_min_gap(&dealt, hard).expect("hard has 3 showings");
        assert!(
            gap >= MIN_REPEAT_SPACING,
            "lone multi-show card must stay >= {MIN_REPEAT_SPACING} apart, got gap={gap}"
        );
    }

    /// Many multi-show cards (kanji-heavy mix): every card_id with >= 2
    /// showings keeps `MIN_REPEAT_SPACING` between consecutive showings.
    #[test]
    fn deal_keeps_min_spacing_across_many_multishow() {
        let mut spec: Vec<(Ulid, usize)> = Vec::new();
        let mut tracked: Vec<Ulid> = Vec::new();
        for _ in 0..5 {
            let id = Ulid::new();
            tracked.push(id);
            spec.push((id, 3));
        }
        for _ in 0..8 {
            let id = Ulid::new();
            tracked.push(id);
            spec.push((id, 2));
        }
        for _ in 0..4 {
            spec.push((Ulid::new(), 1));
        }
        let content = build_content_spec(&spec);
        let dealt = deal_by_card_id(content);

        assert_eq!(dealt.len(), 5 * 3 + 8 * 2 + 4, "deal must place every slot");
        for id in &tracked {
            let gap = content_min_gap(&dealt, *id).expect("tracked card is multi-show");
            assert!(
                gap >= MIN_REPEAT_SPACING,
                "card {id} has gap={gap} < {MIN_REPEAT_SPACING}"
            );
        }
    }

    /// Within a card the showings keep their original view order: the primary
    /// (Normal) variant precedes every copy (Reversed) in the dealt output.
    #[test]
    fn deal_preserves_within_card_view_order() {
        let a = Ulid::new();
        let b = Ulid::new();
        let content = build_content_spec(&[(a, 3), (b, 2), (Ulid::new(), 1)]);
        let dealt = deal_by_card_id(content);

        for id in [a, b] {
            let views: Vec<&LessonCardView> = dealt
                .iter()
                .filter(|(_, lc)| lc.card_id() == id)
                .map(|(_, lc)| lc.view())
                .collect();
            assert!(views.len() >= 2, "card {id} should have >= 2 showings");
            assert!(
                matches!(views[0], LessonCardView::Normal(_)),
                "primary (Normal) must be the first showing of card {id}"
            );
            assert!(
                views[1..]
                    .iter()
                    .all(|v| matches!(v, LessonCardView::Reversed(_))),
                "copies must follow the primary in view order for card {id}"
            );
        }
    }

    /// On a structurally infeasible core (one card with 3 showings and only 2
    /// filler slots — needs 1+3+1+3+1=9 slots, has 5), the deal degrades to
    /// best-effort without panicking and still emits every slot.
    #[test]
    fn deal_best_effort_on_infeasible_core() {
        let hard = Ulid::new();
        let content = build_content_spec(&[(hard, 3), (Ulid::new(), 1), (Ulid::new(), 1)]);
        let dealt = deal_by_card_id(content);
        assert_eq!(
            dealt.len(),
            5,
            "best-effort deal must still emit every slot"
        );
    }

    // --- Multi-show density regression (Symptom 1) ---
    //
    // Reproduces the user-reported "high density" symptom: repetitions of the
    // same word/kanji land nearly consecutively. Runs the FULL cards_to_lesson
    // pipeline and asserts every multi-show card_id keeps at least
    // MIN_REPEAT_SPACING other cards between consecutive showings, across a
    // matrix of realistic lesson shapes (vocab-heavy, kanji-heavy, mixed).

    fn build_multishow_scenario(vocab_n: usize, kanji_n: usize) -> KnowledgeSet {
        let mut ks = KnowledgeSet::new();
        seed_distractor_vocab(&mut ks, &["犬", "鳥", "魚", "馬", "牛", "虎", "狼", "鹿"]);
        for i in 0..vocab_n {
            ks.create_card(vocab_card(&format!("vv{i}")))
                .expect("create new vocab");
        }
        for i in 0..kanji_n {
            let sc = ks
                .create_card(Card::Kanji(KanjiCard::new_test(format!("kk{i}"))))
                .expect("create kanji");
            rate_into_state(&mut ks, *sc.card_id(), 3.0, 8.0, 1, Rating::Hard);
        }
        ks
    }

    fn min_gap_for_multishow_cards(lesson: &LessonData) -> Vec<(Ulid, usize, Vec<usize>)> {
        let mut by_card: HashMap<Ulid, Vec<usize>> = HashMap::new();
        for (i, (_, lc)) in lesson.cards.iter().enumerate() {
            by_card.entry(lc.card_id()).or_default().push(i);
        }
        let mut out = Vec::new();
        for (card_id, positions) in by_card {
            if positions.len() < 2 {
                continue;
            }
            let min_gap = positions
                .windows(2)
                .map(|w| w[1] - w[0] - 1)
                .min()
                .unwrap_or(usize::MAX);
            out.push((card_id, min_gap, positions));
        }
        out.sort_by_key(|(_, g, _)| *g);
        out
    }

    #[rstest]
    #[case::vocab5_kanji3(5, 3)]
    #[case::vocab4_kanji4(4, 4)]
    #[case::vocab2_kanji5(2, 5)]
    #[case::vocab6_kanji2(6, 2)]
    #[case::vocab3_kanji5(3, 5)]
    fn multishow_density_honours_min_spacing(#[case] vocab_n: usize, #[case] kanji_n: usize) {
        init_test_dict();
        ensure_test_phrase_index();
        let ks = build_multishow_scenario(vocab_n, kanji_n);

        let lesson = ks.cards_to_lesson(
            30,
            &JlptContent::new(),
            JapaneseLevel::N5,
            NativeLanguage::Russian,
        );

        let gaps = min_gap_for_multishow_cards(&lesson);
        assert!(
            !gaps.is_empty(),
            "scenario vocab={vocab_n} kanji={kanji_n} should produce at least one multi-show card"
        );
        for (card_id, min_gap, positions) in &gaps {
            assert!(
                *min_gap >= MIN_REPEAT_SPACING,
                "vocab={vocab_n} kanji={kanji_n}: card {card_id} has min_gap={min_gap} \
                 (< {MIN_REPEAT_SPACING}) at positions={positions:?}"
            );
        }
    }

    // --- Phrase no-starvation regression (Symptom 2, Option alpha) ---
    //
    // NEW no-anchor phrases must still appear when anchored phrases exhaust the
    // shared new-phrase budget. Under Option alpha the no-anchor slot
    // (TAIL_PHRASE_PER_LESSON) is independent of phrase_new_budget, so a
    // tail-eligible NEW phrase anchored to no lesson word is admitted even with
    // a depleted budget.

    fn phrase_id_independent() -> Ulid {
        Ulid::from_string("01KPJ5S3N1DRFFD236Z4EZ03HS").expect("valid ULID")
    }

    #[test]
    fn new_no_anchor_phrase_appears_when_anchored_exhausts_budget() {
        ensure_test_phrase_index();

        let mut ks = KnowledgeSet::new();
        // Lesson-core anchors: "test" and "hello" are NEW (not known) so they
        // drive anchored phrase selection and consume the new-phrase budget.
        let test_sc = ks.create_card(vocab_card("test")).expect("create test");
        let hello_sc = ks.create_card(vocab_card("hello")).expect("create hello");

        // Known words that are NOT in the lesson core: they make the
        // [alpha, beta] phrase tail-eligible without turning it into an
        // anchored phrase.
        for word in ["alpha", "beta"] {
            let sc = ks
                .create_card(vocab_card(word))
                .expect("create known vocab");
            ks.mark_card_as_known(*sc.card_id()).expect("mark known");
        }

        // Anchored phrases (token "test"/"hello") + the independent phrase.
        for pid in [
            phrase_id_hello(),
            phrase_id_bye(),
            phrase_id_morning(),
            phrase_id_thanks(),
            phrase_id_extra1(),
            phrase_id_extra2(),
            phrase_id_independent(),
        ] {
            ks.create_card(phrase_card(pid)).expect("create phrase");
        }

        let lesson = LessonData {
            cards: vec![
                (*test_sc.card_id(), lesson_card_for(vocab_card("test"))),
                (*hello_sc.card_id(), lesson_card_for(vocab_card("hello"))),
            ],
            core_count: 2,
        };

        // Budget is deliberately tiny: two anchored NEW phrases (one per anchor
        // word, INTERLEAVED_PHRASES_PER_WORD=2) drain it to zero before the
        // no-anchor pass runs.
        let mut budget = 2;
        let result = add_phrases(lesson, &ks, NativeLanguage::Russian, &mut budget);
        let phrases = lesson_phrase_ids(&result);

        assert!(
            phrases.contains(&phrase_id_independent()),
            "NEW no-anchor phrase must appear even when anchored phrases exhaust the budget: \
             got {phrases:?}"
        );
    }

    // --- Proportional type-slot allocation (grammar-starvation fix) ---
    //
    // CARD_TYPE_WEIGHTS was originally meant as a PERCENTAGE split (V:K:G ≈
    // 80:10:10) but the legacy `weighted_interleave_by_type` treated it as a
    // round-robin pattern and sliced the first N positions off the result.
    // At `daily_new_limit ≤ 9` this gave Grammar 0 slots every day. The new
    // `compute_type_slots` uses largest-remainder + minor-priority tie-break
    // so each type gets its proportional share even at small limits.

    fn infinite_pool() -> HashMap<CardType, usize> {
        let large = 1_000;
        [
            (CardType::Vocabulary, large),
            (CardType::Kanji, large),
            (CardType::Grammar, large),
        ]
        .into_iter()
        .collect()
    }

    fn slots_count(slots: &HashMap<CardType, usize>, t: CardType) -> u32 {
        slots.get(&t).copied().unwrap_or(0) as u32
    }

    /// Expected per-type slot split for a single `allowed` value. Deterministic
    /// cases must hold for every RNG seed; random cases must reach BOTH
    /// branches across enough seeds.
    enum SlotExpectation {
        Deterministic(u32, u32, u32),
        Random {
            a: (u32, u32, u32),
            b: (u32, u32, u32),
        },
    }

    #[rstest]
    #[case::minimal(3, SlotExpectation::Random { a: (2, 1, 0), b: (2, 0, 1) })]
    #[case::light(6, SlotExpectation::Deterministic(4, 1, 1))]
    #[case::medium(9, SlotExpectation::Deterministic(7, 1, 1))]
    #[case::hard(15, SlotExpectation::Random { a: (12, 2, 1), b: (12, 1, 2) })]
    #[case::heavy(21, SlotExpectation::Random { a: (16, 3, 2), b: (16, 2, 3) })]
    #[case::maximum_unit(30, SlotExpectation::Deterministic(24, 3, 3))]
    fn compute_type_slots_reproduces_load_matrix(
        #[case] allowed: usize,
        #[case] expected: SlotExpectation,
    ) {
        let available = infinite_pool();
        match expected {
            SlotExpectation::Deterministic(v, k, g) => {
                for seed in [0u64, 1, 42, 99, 1234] {
                    let mut rng = StdRng::seed_from_u64(seed);
                    let slots = compute_type_slots(allowed, &available, &mut rng);
                    assert_eq!(
                        slots_count(&slots, CardType::Vocabulary),
                        v,
                        "V wrong, allowed={allowed}, seed={seed}"
                    );
                    assert_eq!(
                        slots_count(&slots, CardType::Kanji),
                        k,
                        "K wrong, allowed={allowed}, seed={seed}"
                    );
                    assert_eq!(
                        slots_count(&slots, CardType::Grammar),
                        g,
                        "G wrong, allowed={allowed}, seed={seed}"
                    );
                }
            },
            SlotExpectation::Random { a, b } => {
                let (mut found_a, mut found_b) = (false, false);
                for seed in 0..200u64 {
                    let mut rng = StdRng::seed_from_u64(seed);
                    let slots = compute_type_slots(allowed, &available, &mut rng);
                    let triple = (
                        slots_count(&slots, CardType::Vocabulary),
                        slots_count(&slots, CardType::Kanji),
                        slots_count(&slots, CardType::Grammar),
                    );
                    assert!(
                        triple == a || triple == b,
                        "allowed={allowed}, seed={seed}: unexpected triple {triple:?}"
                    );
                    if triple == a {
                        found_a = true;
                    }
                    if triple == b {
                        found_b = true;
                    }
                }
                assert!(found_a, "case A never observed for allowed={allowed}");
                assert!(found_b, "case B never observed for allowed={allowed}");
            },
        }
    }

    #[test]
    fn compute_type_slots_returns_empty_when_allowed_zero() {
        let available = infinite_pool();
        let mut rng = StdRng::seed_from_u64(0);
        let slots = compute_type_slots(0, &available, &mut rng);
        assert!(slots.is_empty());
    }

    #[test]
    fn compute_type_slots_returns_empty_when_pool_empty() {
        let available: HashMap<CardType, usize> = HashMap::new();
        let mut rng = StdRng::seed_from_u64(0);
        let slots = compute_type_slots(9, &available, &mut rng);
        assert!(slots.is_empty());
    }

    #[test]
    fn compute_type_slots_normalizes_when_main_type_absent() {
        // V absent from pool → renormalize over K:G = 1:1, sum_w=2.
        let mut available = HashMap::new();
        available.insert(CardType::Kanji, 100);
        available.insert(CardType::Grammar, 100);
        for seed in [0u64, 1, 42] {
            let mut rng = StdRng::seed_from_u64(seed);
            let slots = compute_type_slots(10, &available, &mut rng);
            assert_eq!(slots_count(&slots, CardType::Vocabulary), 0, "seed={seed}");
            assert_eq!(slots_count(&slots, CardType::Kanji), 5, "seed={seed}");
            assert_eq!(slots_count(&slots, CardType::Grammar), 5, "seed={seed}");
        }
    }

    #[test]
    fn compute_type_slots_phase2_fallback_to_vocabulary_when_grammar_capped() {
        // V=∞, K=∞, G=1, allowed=30 → raw G=3 capped to 1, leftover=2 falls to
        // V via Phase 2 (the cap-binding fallback path).
        let mut available = HashMap::new();
        available.insert(CardType::Vocabulary, 100);
        available.insert(CardType::Kanji, 100);
        available.insert(CardType::Grammar, 1);
        for seed in [0u64, 1, 42, 99] {
            let mut rng = StdRng::seed_from_u64(seed);
            let slots = compute_type_slots(30, &available, &mut rng);
            assert_eq!(
                slots_count(&slots, CardType::Vocabulary),
                26,
                "V should absorb the cap-induced leftover (seed={seed})"
            );
            assert_eq!(slots_count(&slots, CardType::Kanji), 3, "seed={seed}");
            assert_eq!(
                slots_count(&slots, CardType::Grammar),
                1,
                "G must be capped at available=1 (seed={seed})"
            );
        }
    }

    /// Direct unit test for `distribute_new_cards` multi-group overflow:
    /// when allowed exceeds the first JLPT group's size, the leftover must
    /// roll over to the next group. N5(3V only) is tiny, so the proportional
    /// split for N4(∞) yields the expected 7V+1K+1G there.
    #[test]
    fn distribute_new_cards_overflows_leftover_to_next_jlpt_group() {
        let mut jlpt_content = JlptContent::new();
        jlpt_content.words_by_level.insert(
            JapaneseLevel::N5,
            ["n5w1", "n5w2", "n5w3"]
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
        );
        jlpt_content.words_by_level.insert(
            JapaneseLevel::N4,
            (0..30).map(|i| format!("n4w{i}")).collect::<HashSet<_>>(),
        );
        jlpt_content.kanji_by_level.insert(
            JapaneseLevel::N4,
            (0..10).map(|i| format!("n4k{i}")).collect::<HashSet<_>>(),
        );
        let gids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
        jlpt_content.grammar_by_level.insert(
            JapaneseLevel::N4,
            gids.iter().map(|u| u.to_string()).collect::<HashSet<_>>(),
        );

        let mut ks = KnowledgeSet::new();
        // N5 group: 3 vocab only.
        for w in ["n5w1", "n5w2", "n5w3"] {
            ks.create_card(vocab_card(w)).unwrap();
        }
        // N4 group: 30V + 10K + 5G.
        for i in 0..30 {
            ks.create_card(vocab_card(&format!("n4w{i}"))).unwrap();
        }
        for i in 0..10 {
            ks.create_card(Card::Kanji(KanjiCard::new_test(format!("n4k{i}"))))
                .unwrap();
        }
        for gid in &gids {
            ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
                .unwrap();
        }

        let all_cards: Vec<(&Ulid, &StudyCard)> = ks.study_cards().iter().collect::<Vec<_>>();
        let mut rng = StdRng::seed_from_u64(0);
        let distributed = distribute_new_cards(all_cards, &jlpt_content, 9, &mut rng);
        assert_eq!(distributed.len(), 9, "must return exactly `allowed` cards");

        // Tally per type using the underlying StudyCard.
        let mut vocab = 0;
        let mut kanji = 0;
        let mut grammar = 0;
        for (_, sc) in &distributed {
            match sc.card() {
                Card::Vocabulary(_) => vocab += 1,
                Card::Kanji(_) => kanji += 1,
                Card::Grammar(_) => grammar += 1,
                _ => {},
            }
        }
        // N5 contributes 3V (its entire group), N4 contributes 4V+1K+1G
        // (matrix row "light" for take=6).
        assert_eq!(vocab, 7, "N5 3V + N4 4V = 7");
        assert_eq!(kanji, 1);
        assert_eq!(grammar, 1);
    }

    /// Direct unit test for `distribute_new_cards` JLPT priority: when the
    /// first group alone can fill `allowed`, the second group is untouched.
    #[test]
    fn distribute_new_cards_consumes_first_jlpt_group_when_sufficient() {
        let mut jlpt_content = JlptContent::new();
        jlpt_content.words_by_level.insert(
            JapaneseLevel::N5,
            (0..30).map(|i| format!("n5w{i}")).collect::<HashSet<_>>(),
        );
        jlpt_content.kanji_by_level.insert(
            JapaneseLevel::N5,
            (0..10).map(|i| format!("n5k{i}")).collect::<HashSet<_>>(),
        );
        let gids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
        jlpt_content.grammar_by_level.insert(
            JapaneseLevel::N5,
            gids.iter().map(|u| u.to_string()).collect::<HashSet<_>>(),
        );
        jlpt_content.words_by_level.insert(
            JapaneseLevel::N4,
            ["n4w1"].into_iter().map(|s| s.to_string()).collect(),
        );

        let mut ks = KnowledgeSet::new();
        for i in 0..30 {
            ks.create_card(vocab_card(&format!("n5w{i}"))).unwrap();
        }
        for i in 0..10 {
            ks.create_card(Card::Kanji(KanjiCard::new_test(format!("n5k{i}"))))
                .unwrap();
        }
        for gid in &gids {
            ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
                .unwrap();
        }
        let n4_sc = ks.create_card(vocab_card("n4w1")).unwrap();

        let all_cards: Vec<(&Ulid, &StudyCard)> = ks.study_cards().iter().collect::<Vec<_>>();
        let mut rng = StdRng::seed_from_u64(0);
        let distributed = distribute_new_cards(all_cards, &jlpt_content, 9, &mut rng);
        assert_eq!(distributed.len(), 9);

        // N4 card must NOT appear — N5 alone satisfied `allowed`.
        assert!(
            !distributed.iter().any(|(id, _)| **id == *n4_sc.card_id()),
            "N4 card leaked while N5 group had enough cards"
        );
    }

    #[test]
    fn grammar_appears_in_lesson_at_medium_load() {
        // End-to-end: 30V + 10K + 5G all N5, daily_new_limit=9 (Medium).
        // Before the fix this returned 8V + 1K + 0G (grammar starved).
        // After: 7V + 1K + 1G exactly (deterministic per matrix row "medium").
        let mut ks = KnowledgeSet::new();
        let mut jlpt_content = JlptContent::new();
        jlpt_content.words_by_level.insert(
            JapaneseLevel::N5,
            (0..30).map(|i| format!("vocab{i}")).collect::<HashSet<_>>(),
        );
        jlpt_content.kanji_by_level.insert(
            JapaneseLevel::N5,
            (0..10).map(|i| format!("kanji{i}")).collect::<HashSet<_>>(),
        );
        let grammar_ids: Vec<Ulid> = (0..5).map(|_| Ulid::new()).collect();
        jlpt_content.grammar_by_level.insert(
            JapaneseLevel::N5,
            grammar_ids
                .iter()
                .map(|u| u.to_string())
                .collect::<HashSet<_>>(),
        );

        for i in 0..30 {
            ks.create_card(vocab_card(&format!("vocab{i}"))).unwrap();
        }
        for i in 0..10 {
            ks.create_card(Card::Kanji(KanjiCard::new_test(format!("kanji{i}"))))
                .unwrap();
        }
        for gid in &grammar_ids {
            ks.create_card(Card::Grammar(GrammarRuleCard::new_test_with_id(*gid)))
                .unwrap();
        }

        let lesson =
            ks.cards_to_lesson(9, &jlpt_content, JapaneseLevel::N5, NativeLanguage::Russian);

        // Multi-show expansion may add an extra slot for the same grammar
        // card_id, so we count DISTINCT grammar card_ids, not raw slots.
        let grammar_card_ids: HashSet<Ulid> = lesson
            .values()
            .filter_map(|lc| {
                let card = ks.get_card(lc.card_id())?.card();
                matches!(card, Card::Grammar(_)).then_some(lc.card_id())
            })
            .collect();

        assert_eq!(
            grammar_card_ids.len(),
            1,
            "Medium=9 must include exactly 1 distinct grammar card, got {}",
            grammar_card_ids.len()
        );
    }
}
