use std::collections::HashMap;

use crate::dictionary::kanji::KanjiInfo;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::NativeLanguage;
use crate::domain::{Card, CardType, GrammarRuleCard, MemoryHistory};
use rand::Rng;

use super::types::LessonCardView;

mod generation;
#[cfg(test)]
mod tests;
mod transforms;

const QUIZ_OPTIONS_COUNT: usize = 4;

const PROB_NORMAL_VIEW: f32 = 0.15;
const PROB_QUIZ_VIEW: f32 = 0.30;
const PROB_YESNO_VIEW: f32 = 0.50;
const PROB_REVERSED_VIEW: f32 = 0.75;

const PROB_KANJI_NORMAL: f32 = 1.0 / 5.0;
const PROB_KANJI_READING_QUIZ: f32 = 2.0 / 5.0;
const PROB_KANJI_QUIZ: f32 = 3.0 / 5.0;
const PROB_KANJI_YESNO: f32 = 4.0 / 5.0;

const PROB_NEW_KANJI_NORMAL: f32 = 0.33;
const PROB_NEW_KANJI_QUIZ: f32 = 0.66;
const PROB_NEW_VOCAB_NORMAL: f32 = 0.50;

const PROB_NEW_PHRASE_NORMAL: f32 = 0.50;
const PROB_REVIEW_PHRASE_NORMAL: f32 = 0.15;

const PROB_GRAMMAR_QUIZ: f32 = 0.50;

const EASY_REVIEWS_FOR_REVERSED: usize = 2;
const GOOD_REVIEWS_FOR_REVERSED: usize = 4;

pub struct LessonViewGenerator<'a> {
    knowledge_set: &'a KnowledgeSet,
    cards_by_type: HashMap<CardType, Vec<Card>>,
    known_grammars: Vec<GrammarRuleCard>,
    kanji_cache: HashMap<String, &'static KanjiInfo>,
    native_language: NativeLanguage,
}

impl<'a> LessonViewGenerator<'a> {
    pub fn new(knowledge_set: &'a KnowledgeSet, native_language: NativeLanguage) -> Self {
        let cards_by_type = knowledge_set.build_cards_by_type();
        // Мутации идут только по ранее изученным правилам: новая
        // грамматика ещё не прошла руку знакомства — её слайд юзер
        // не видел, и рейтинг мутированного слова не должен уводить её
        // из «новых» в обход знакомства.
        let known_grammars: Vec<_> = knowledge_set
            .study_cards()
            .values()
            .filter(|study_card| !study_card.memory().is_new())
            .filter_map(|x| match x.card() {
                Card::Grammar(grammar_rule_card) => Some(grammar_rule_card.clone()),
                _ => None,
            })
            .collect();

        Self {
            knowledge_set,
            cards_by_type,
            known_grammars,
            kanji_cache: HashMap::new(),
            native_language,
        }
    }

    fn same_type_cards(&self, card_type: &CardType) -> &[Card] {
        self.cards_by_type
            .get(card_type)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn apply_view(
        &mut self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut impl Rng,
    ) -> LessonCardView {
        let card = study_card.card();
        let card_type = CardType::from(card);

        match card_type {
            CardType::Grammar if !is_new => {
                let rand_val = rng.random::<f32>();
                if rand_val < PROB_GRAMMAR_QUIZ {
                    generation::generate_grammar_quiz(
                        card.clone(),
                        self.knowledge_set,
                        &self.native_language,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
                } else {
                    LessonCardView::Normal(card.clone())
                }
            },
            CardType::Grammar => LessonCardView::Normal(card.clone()),
            CardType::Kanji if is_new => {
                let same_type_cards = self.same_type_cards(&card_type);
                self.select_new_kanji_view(card, same_type_cards, rng)
            },
            CardType::Kanji => {
                let same_type_cards: &[Card] = self
                    .cards_by_type
                    .get(&card_type)
                    .map(|v| v.as_slice())
                    .unwrap_or(&[]);
                Self::select_review_kanji_view(
                    card,
                    same_type_cards,
                    &mut self.kanji_cache,
                    rng,
                    self.native_language,
                )
            },
            CardType::Vocabulary if is_new => {
                let same_type_cards = self.same_type_cards(&card_type);
                self.select_new_vocab_view(card, same_type_cards, rng)
            },
            CardType::Vocabulary => {
                let same_type_cards = self.same_type_cards(&card_type);
                self.select_review_vocab_view(card, same_type_cards, study_card.memory(), rng)
            },
            CardType::Phrase => {
                let same_type_cards = self.same_type_cards(&card_type);
                self.select_phrase_view(card, same_type_cards, is_new, rng)
            },
        }
    }

    fn select_new_kanji_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NEW_KANJI_NORMAL {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_NEW_KANJI_QUIZ {
            generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else {
            LessonCardView::Writing(card.clone())
        }
    }

    fn select_review_kanji_view<R: Rng>(
        card: &Card,
        same_type_cards: &[Card],
        kanji_cache: &mut HashMap<String, &'static KanjiInfo>,
        rng: &mut R,
        native_language: NativeLanguage,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_KANJI_NORMAL {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_KANJI_READING_QUIZ {
            generation::generate_kanji_reading_quiz(card.clone(), same_type_cards, kanji_cache)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if rand_val < PROB_KANJI_QUIZ {
            generation::generate_quiz(card.clone(), same_type_cards, &native_language)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if rand_val < PROB_KANJI_YESNO {
            generation::generate_yesno(card.clone(), same_type_cards, &native_language, rng)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else {
            LessonCardView::Writing(card.clone())
        }
    }

    fn select_new_vocab_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NEW_VOCAB_NORMAL {
            LessonCardView::Normal(card.clone())
        } else {
            generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        }
    }

    fn select_review_vocab_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        memory: &MemoryHistory,
        rng: &mut R,
    ) -> LessonCardView {
        let is_high_difficulty = memory.is_high_difficulty();
        let eligible_for_advanced = memory.is_known_card() || memory.is_in_progress();
        let eligible_for_reversed = eligible_for_advanced
            || memory.easy_review_count() > EASY_REVIEWS_FOR_REVERSED
            || memory.good_review_count() >= GOOD_REVIEWS_FOR_REVERSED;
        let rand_val = rng.random::<f32>();
        if rand_val < PROB_NORMAL_VIEW {
            LessonCardView::Normal(card.clone())
        } else if rand_val < PROB_QUIZ_VIEW {
            generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if !is_high_difficulty && rand_val < PROB_YESNO_VIEW {
            generation::generate_yesno(card.clone(), same_type_cards, &self.native_language, rng)
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone()))
        } else if eligible_for_reversed && rand_val < PROB_REVERSED_VIEW {
            transforms::apply_reversed(card, &self.native_language)
        } else if eligible_for_advanced {
            transforms::apply_grammar_mutated(
                card,
                &self.known_grammars,
                rng,
                &self.native_language,
            )
        } else {
            LessonCardView::Normal(card.clone())
        }
    }

    fn select_phrase_view<R: Rng>(
        &self,
        card: &Card,
        same_type_cards: &[Card],
        is_new: bool,
        rng: &mut R,
    ) -> LessonCardView {
        let rand_val = rng.random::<f32>();
        let normal_threshold = if is_new {
            PROB_NEW_PHRASE_NORMAL
        } else {
            PROB_REVIEW_PHRASE_NORMAL
        };

        if rand_val < normal_threshold {
            return LessonCardView::Normal(card.clone());
        }

        generation::generate_phrase_quiz(card.clone(), same_type_cards, &self.native_language)
            .unwrap_or_else(|| LessonCardView::Normal(card.clone()))
    }

    pub(crate) fn candidate_views_for_repeat<R: Rng>(
        &mut self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut R,
    ) -> Vec<LessonCardView> {
        let card = study_card.card();
        match CardType::from(card) {
            CardType::Vocabulary => self.candidate_views_for_vocab_repeat(study_card, is_new, rng),
            CardType::Kanji => self.candidate_views_for_kanji_repeat(study_card, is_new, rng),
            CardType::Grammar => self.candidate_views_for_grammar_repeat(study_card, is_new, rng),
            CardType::Phrase => Vec::new(),
        }
    }

    fn candidate_views_for_vocab_repeat<R: Rng>(
        &self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut R,
    ) -> Vec<LessonCardView> {
        let card = study_card.card();
        let same_type_cards: &[Card] = self
            .cards_by_type
            .get(&CardType::Vocabulary)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        if is_new {
            build_distinct_views(vec![
                LessonCardView::Normal(card.clone()),
                generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
            ])
        } else {
            let memory = study_card.memory();
            let is_high_difficulty = memory.is_high_difficulty();
            let eligible_for_advanced = memory.is_known_card() || memory.is_in_progress();
            let eligible_for_reversed = eligible_for_advanced
                || memory.easy_review_count() > EASY_REVIEWS_FOR_REVERSED
                || memory.good_review_count() >= GOOD_REVIEWS_FOR_REVERSED;

            let mut candidates = vec![
                generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
            ];
            if !is_high_difficulty {
                candidates.push(
                    generation::generate_yesno(
                        card.clone(),
                        same_type_cards,
                        &self.native_language,
                        rng,
                    )
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                );
            }
            if eligible_for_reversed {
                candidates.push(transforms::apply_reversed(card, &self.native_language));
            }
            if eligible_for_advanced {
                candidates.push(transforms::apply_grammar_mutated(
                    card,
                    &self.known_grammars,
                    rng,
                    &self.native_language,
                ));
            }
            candidates.push(LessonCardView::Normal(card.clone()));

            build_distinct_views(candidates)
        }
    }

    fn candidate_views_for_kanji_repeat<R: Rng>(
        &mut self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        rng: &mut R,
    ) -> Vec<LessonCardView> {
        let card = study_card.card();
        let same_type_cards: &[Card] = self
            .cards_by_type
            .get(&CardType::Kanji)
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        if is_new {
            build_distinct_views(vec![
                LessonCardView::Normal(card.clone()),
                generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                LessonCardView::Writing(card.clone()),
            ])
        } else {
            build_distinct_views(vec![
                generation::generate_kanji_reading_quiz(
                    card.clone(),
                    same_type_cards,
                    &mut self.kanji_cache,
                )
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                generation::generate_quiz(card.clone(), same_type_cards, &self.native_language)
                    .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                generation::generate_yesno(
                    card.clone(),
                    same_type_cards,
                    &self.native_language,
                    rng,
                )
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                LessonCardView::Writing(card.clone()),
                LessonCardView::Normal(card.clone()),
            ])
        }
    }

    fn candidate_views_for_grammar_repeat<R: Rng>(
        &self,
        study_card: &crate::domain::StudyCard,
        is_new: bool,
        _rng: &mut R,
    ) -> Vec<LessonCardView> {
        let card = study_card.card();
        if is_new {
            vec![LessonCardView::Normal(card.clone())]
        } else {
            build_distinct_views(vec![
                generation::generate_grammar_quiz(
                    card.clone(),
                    self.knowledge_set,
                    &self.native_language,
                )
                .unwrap_or_else(|_| LessonCardView::Normal(card.clone())),
                LessonCardView::Normal(card.clone()),
            ])
        }
    }
}

fn build_distinct_views(candidates: Vec<LessonCardView>) -> Vec<LessonCardView> {
    use std::collections::HashSet;
    let mut seen: HashSet<std::mem::Discriminant<LessonCardView>> = HashSet::new();
    let mut result = Vec::with_capacity(candidates.len());
    for view in candidates {
        let disc = std::mem::discriminant(&view);
        if seen.insert(disc) {
            result.push(view);
        }
    }
    result
}
