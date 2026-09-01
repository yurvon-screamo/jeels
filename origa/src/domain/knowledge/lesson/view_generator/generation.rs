use crate::dictionary::grammar::get_rule_by_id;
use crate::dictionary::kanji::{KanjiInfo, get_kanji_info};
use crate::domain::grammar::apply_format_actions;
use crate::domain::knowledge::KnowledgeSet;
use crate::domain::value_objects::{CardAnswer, NativeLanguage};
use crate::domain::{
    Card, OrigaError, PartOfSpeech, find_known_vocab_words_for_pos, generate_grammar_distractors,
};
use rand::{Rng, prelude::IndexedRandom, seq::SliceRandom};
use std::collections::HashMap;

use super::super::types::{
    GrammarInfo, GrammarQuizCard, LessonCardView, QuizCard, QuizMode, QuizOption, YesNoCard,
};
use super::QUIZ_OPTIONS_COUNT;

fn answer_display_text(answer: &CardAnswer) -> String {
    answer.text_projection()
}

pub(crate) fn generate_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
) -> Result<LessonCardView, OrigaError> {
    match &original_card {
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => {},
    }

    let correct_answer = original_card.answer(lang)?;
    let correct_text = answer_display_text(&correct_answer);

    let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
    let mut rng = rand::rng();
    let selected_distractors = collect_pos_filtered_distractors(
        &original_card,
        same_type_cards,
        &correct_text,
        lang,
        needed_distractors,
        &mut rng,
    );

    if selected_distractors.len() < needed_distractors {
        return Ok(LessonCardView::Normal(original_card));
    }

    let mut options: Vec<QuizOption> = selected_distractors
        .into_iter()
        .map(|text| QuizOption::new_simple(text, false))
        .collect();

    options.push(QuizOption::new_simple(correct_text, true));
    options.shuffle(&mut rand::rng());

    let quiz = QuizCard::new(original_card, options, QuizMode::Single);
    Ok(LessonCardView::Quiz(quiz))
}

fn collect_pos_filtered_distractors(
    original_card: &Card,
    same_type_cards: &[Card],
    correct_answer_text: &str,
    lang: &NativeLanguage,
    needed: usize,
    rng: &mut impl Rng,
) -> Vec<String> {
    let target_pos: Option<PartOfSpeech> = match original_card {
        Card::Vocabulary(vc) => vc.part_of_speech().ok(),
        _ => None,
    };

    let (mut same_pos, mut other): (Vec<String>, Vec<String>) = (Vec::new(), Vec::new());

    for card in same_type_cards {
        let answer_text = match card.answer(lang).ok() {
            Some(a) if answer_display_text(&a) != correct_answer_text => answer_display_text(&a),
            _ => continue,
        };

        match target_pos {
            Some(ref pos) => {
                let card_pos = match card {
                    Card::Vocabulary(vc) => vc.part_of_speech().ok(),
                    _ => None,
                };
                if card_pos.as_ref() == Some(pos) {
                    same_pos.push(answer_text);
                } else {
                    other.push(answer_text);
                }
            },
            None => same_pos.push(answer_text),
        }
    }

    same_pos.shuffle(rng);
    other.shuffle(rng);

    let mut result: Vec<String> = same_pos.into_iter().take(needed).collect();
    if result.len() < needed {
        result.extend(other.into_iter().take(needed - result.len()));
    }

    result
}

pub(crate) fn generate_yesno(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
    rng: &mut impl Rng,
) -> Result<LessonCardView, OrigaError> {
    match &original_card {
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) | Card::Phrase(_) => {},
    }

    let question = original_card.question(lang)?;
    let correct_answer = original_card.answer(lang)?;
    let correct_text = answer_display_text(&correct_answer);

    let is_correct = rng.random_bool(0.5);

    let statement_answer = if is_correct {
        correct_text
    } else {
        let distractors = collect_pos_filtered_distractors(
            &original_card,
            same_type_cards,
            &correct_text,
            lang,
            1,
            rng,
        );
        let Some(statement_answer) = distractors.into_iter().next() else {
            return Ok(LessonCardView::Normal(original_card));
        };
        statement_answer
    };

    let word = question.text().to_string();

    Ok(LessonCardView::YesNo(YesNoCard::new(
        original_card,
        word,
        statement_answer,
        is_correct,
    )))
}

pub(crate) fn generate_phrase_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    lang: &NativeLanguage,
) -> Option<LessonCardView> {
    let phrase_card = match &original_card {
        Card::Phrase(pc) => pc,
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => return None,
    };

    let audio_file = format!("{}.opus", phrase_card.phrase_id());
    let correct_text = answer_display_text(&original_card.answer(lang).ok()?);

    let mut distractors: Vec<String> = same_type_cards
        .iter()
        .filter_map(|c| match c {
            Card::Phrase(_) => c
                .answer(lang)
                .ok()
                .map(|a| answer_display_text(&a))
                .filter(|text| text != &correct_text),
            Card::Vocabulary(_) | Card::Kanji(_) | Card::Grammar(_) => None,
        })
        .collect();

    if distractors.is_empty() {
        return None;
    }

    distractors.shuffle(&mut rand::rng());

    let max_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
    let selected: Vec<String> = distractors.into_iter().take(max_distractors).collect();

    let mut options: Vec<QuizOption> = selected
        .into_iter()
        .map(|text| QuizOption::new_simple(text, false))
        .collect();

    options.push(QuizOption::new_simple(correct_text, true));
    options.shuffle(&mut rand::rng());

    Some(LessonCardView::PhraseListen {
        card: original_card,
        audio_file,
        options,
    })
}

pub(crate) fn generate_kanji_reading_quiz(
    original_card: Card,
    same_type_cards: &[Card],
    kanji_cache: &mut HashMap<String, &'static KanjiInfo>,
) -> Result<LessonCardView, OrigaError> {
    let kanji_card = match &original_card {
        Card::Kanji(kc) => kc,
        Card::Vocabulary(_) | Card::Grammar(_) | Card::Phrase(_) => {
            return Ok(LessonCardView::Normal(original_card));
        },
    };

    let kanji_str = kanji_card.kanji().text();
    let info = match cached_kanji_info(kanji_str, kanji_cache) {
        Ok(info) => info,
        Err(_) => return Ok(LessonCardView::Normal(original_card)),
    };

    // Filter out rare readings — they pay back little learning effort and
    // clutter the quiz. If filtering leaves nothing, fall back to all
    // readings (the kanji has no signal of rarity, so every reading matters).
    let all_on: Vec<String> = info.on_readings().to_vec();
    let all_kun: Vec<String> = info.kun_readings().to_vec();
    let total_readings = all_on.len() + all_kun.len();
    if total_readings == 0 {
        return Ok(LessonCardView::Normal(original_card));
    }

    let non_rare_on: Vec<String> = all_on
        .iter()
        .filter(|r| !info.is_rare_reading(r))
        .cloned()
        .collect();
    let non_rare_kun: Vec<String> = all_kun
        .iter()
        .filter(|r| !info.is_rare_reading(r))
        .cloned()
        .collect();

    let (target_on, target_kun) = if non_rare_on.is_empty() && non_rare_kun.is_empty() {
        (all_on, all_kun)
    } else {
        (non_rare_on, non_rare_kun)
    };

    let target_readings_count = target_on.len() + target_kun.len();
    if target_readings_count > 6 {
        return Ok(LessonCardView::Normal(original_card));
    }

    let target_readings: Vec<String> = target_on.iter().chain(target_kun.iter()).cloned().collect();

    let mut options: Vec<QuizOption> = target_on
        .iter()
        .map(|r| QuizOption::new(r.clone(), true, Some("ON".to_string())))
        .chain(
            target_kun
                .iter()
                .map(|r| QuizOption::new(r.clone(), true, Some("KUN".to_string()))),
        )
        .collect();

    let correct_count = options.len();

    let mut distractors: Vec<String> =
        collect_distractors(same_type_cards, &target_readings, kanji_cache);

    if distractors.len() < 2 {
        return Ok(LessonCardView::Normal(original_card));
    }

    let max_distractors = 8usize.saturating_sub(correct_count);
    distractors.truncate(max_distractors);

    let distractor_options: Vec<QuizOption> = distractors
        .into_iter()
        .map(|text| QuizOption::new_simple(text, false))
        .collect();

    options.extend(distractor_options);
    options.shuffle(&mut rand::rng());

    let quiz = QuizCard::new(original_card, options, QuizMode::Multi);
    Ok(LessonCardView::KanjiReadingQuiz(quiz))
}

fn collect_distractors(
    same_type_cards: &[Card],
    target_readings: &[String],
    kanji_cache: &mut HashMap<String, &'static KanjiInfo>,
) -> Vec<String> {
    let mut distractors: Vec<String> = same_type_cards
        .iter()
        .filter_map(|c| match c {
            Card::Kanji(kc) => {
                let other_info = cached_kanji_info(kc.kanji().text(), kanji_cache).ok()?;
                // Drop rare readings of OTHER kanji too — offering a rare
                // reading as a wrong-answer distractor rewards students for
                // memorising low-value material.
                let mut readings = other_info
                    .on_readings()
                    .iter()
                    .filter(|r| !other_info.is_rare_reading(r))
                    .cloned()
                    .collect::<Vec<_>>();
                readings.extend(
                    other_info
                        .kun_readings()
                        .iter()
                        .filter(|r| !other_info.is_rare_reading(r))
                        .cloned(),
                );
                Some(readings)
            },
            Card::Vocabulary(_) | Card::Grammar(_) | Card::Phrase(_) => None,
        })
        .flatten()
        .filter(|r| !target_readings.contains(r))
        .collect();

    let mut seen = std::collections::HashSet::new();
    distractors.retain(|r| seen.insert(r.clone()));
    distractors.shuffle(&mut rand::rng());

    distractors
}

pub(crate) fn cached_kanji_info<'a>(
    kanji: &str,
    cache: &'a mut HashMap<String, &'static KanjiInfo>,
) -> Result<&'a KanjiInfo, OrigaError> {
    if !cache.contains_key(kanji) {
        let info = get_kanji_info(kanji)?;
        cache.insert(kanji.to_string(), info);
    }
    Ok(cache[kanji])
}

pub(crate) fn generate_grammar_quiz(
    original_card: Card,
    knowledge_set: &KnowledgeSet,
    lang: &NativeLanguage,
) -> Result<LessonCardView, OrigaError> {
    let grammar_rule_card = match &original_card {
        Card::Grammar(grc) => grc,
        Card::Vocabulary(_) | Card::Kanji(_) | Card::Phrase(_) => {
            return Ok(LessonCardView::Normal(original_card));
        },
    };

    let rule = get_rule_by_id(grammar_rule_card.rule_id()).ok_or_else(|| {
        OrigaError::GrammarRuleNotFound {
            rule_id: *grammar_rule_card.rule_id(),
        }
    })?;

    let Some(format_map) = rule.format_map() else {
        return Ok(LessonCardView::Normal(original_card));
    };

    let supported_pos: Vec<&PartOfSpeech> = format_map.keys().collect();
    let applicable_pos = grammar_rule_card
        .apply_to()
        .iter()
        .find(|pos| supported_pos.contains(pos))
        .cloned()
        .ok_or_else(|| OrigaError::GrammarFormatError {
            reason: "No supported part of speech".to_string(),
        })?;

    let matching_vocab = find_known_vocab_words_for_pos(knowledge_set, &applicable_pos);
    if matching_vocab.is_empty() {
        return Ok(LessonCardView::Normal(original_card));
    }

    let mut rng = rand::rng();
    let word_text = matching_vocab
        .choose(&mut rng)
        .unwrap_or(&matching_vocab[0])
        .clone();

    let Some(rules) = format_map.get(&applicable_pos) else {
        return Ok(LessonCardView::Normal(original_card));
    };

    let correct_text = apply_format_actions(&word_text, rules, &applicable_pos)?;

    let needed_distractors = QUIZ_OPTIONS_COUNT.saturating_sub(1);
    let distractors = generate_grammar_distractors(
        rules,
        &word_text,
        &applicable_pos,
        &correct_text,
        needed_distractors,
        &mut rng,
    );

    if distractors.len() < needed_distractors {
        return Ok(LessonCardView::Normal(original_card));
    }

    let mut options: Vec<QuizOption> = distractors
        .into_iter()
        .map(|text| QuizOption::new_simple(text, false))
        .collect();
    options.push(QuizOption::new_simple(correct_text, true));
    options.shuffle(&mut rng);

    let quiz = QuizCard::new(original_card.clone(), options, QuizMode::Single);

    let grammar_title = grammar_rule_card
        .title(lang)
        .map(|q| q.text().to_string())
        .unwrap_or_else(|_| grammar_rule_card.rule_id().to_string());
    let grammar_desc = grammar_rule_card
        .description(lang)
        .map(|a| answer_display_text(&a))
        .unwrap_or_default();

    let grammar_info = GrammarInfo::new(
        Some(*grammar_rule_card.rule_id()),
        grammar_title,
        grammar_desc,
    );

    let grammar_quiz = GrammarQuizCard::new(original_card, grammar_info, word_text, quiz);

    Ok(LessonCardView::GrammarQuiz(grammar_quiz))
}
