use crate::i18n::{t, use_i18n};
use crate::ui_components::{AudioButtons, Tag, TagVariant};
use leptos::prelude::*;
use origa::domain::{Card, GrammarInfo, PartOfSpeech};

use super::card_type::CardType;
use super::grammar_info_badge::GrammarInfoBadge;
use super::pos_label::part_of_speech_label;
use super::quiz_card::QuizVariant;

fn quiz_variant_matches_card_type(quiz_variant: QuizVariant, card_type: CardType) -> bool {
    matches!(
        (quiz_variant, card_type),
        (QuizVariant::Grammar, CardType::Grammar)
    )
}

/// Unified header rendered ABOVE the lesson card for ALL card types:
/// tags on the left (card type, POS, quiz variant, grammar badge),
/// audio button on the right. Both stay outside the Card border so
/// they never compete with each other for horizontal space inside it.
#[component]
pub fn LessonCardTags(
    card_type: CardType,
    grammar_info: Option<GrammarInfo>,
    show_answer: Signal<bool>,
    card: Card,
) -> impl IntoView {
    let i18n = use_i18n();
    let grammar_info = StoredValue::new(grammar_info);
    let pos_label = StoredValue::new(
        card.vocabulary_part_of_speech()
            .map(|p| part_of_speech_label(p, &i18n)),
    );
    view! {
        <div class="flex items-center gap-2 flex-wrap min-w-0 mb-2 px-1">
            <Tag variant=Signal::derive(move || card_type.tag_variant())>
                {card_type.label(&i18n)}
            </Tag>
            <Show when=move || pos_label.get_value().is_some()>
                {move || {
                    pos_label
                        .get_value()
                        .map(|label| {
                            view! {
                                <Tag>
                                    {label}
                                </Tag>
                            }
                        })
                }}
            </Show>
            <Show when=move || show_answer.get() && grammar_info.get_value().is_some()>
                {move || {
                    grammar_info
                        .get_value()
                        .map(|info| {
                            view! {
                                <GrammarInfoBadge title=info.title().to_string() />
                            }
                        })
                }}
            </Show>
        </div>
    }
}

/// Tags row for quiz-type cards (quiz, yesno, phrase) — includes the
/// quiz variant tag (Meaning/Reading/Grammar).
#[component]
pub fn LessonCardTagsQuiz(
    card_type: CardType,
    #[prop(optional)] quiz_variant: QuizVariant,
    #[prop(default = None)] part_of_speech: Option<PartOfSpeech>,
) -> impl IntoView {
    let i18n = use_i18n();
    let pos_label = StoredValue::new(part_of_speech.map(|p| part_of_speech_label(p, &i18n)));

    view! {
        <div class="flex items-center gap-2 flex-wrap min-w-0 mb-2 px-1">
            <Tag variant=Signal::derive(move || card_type.tag_variant())>
                {card_type.label(&i18n)}
            </Tag>
            <Show when=move || pos_label.get_value().is_some()>
                {move || {
                    pos_label
                        .get_value()
                        .map(|label| {
                            view! { <Tag>{label}</Tag> }
                        })
                }}
            </Show>
            <Show when=move || !quiz_variant_matches_card_type(quiz_variant, card_type)>
                {move || {
                    match quiz_variant {
                        QuizVariant::Meaning => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.quiz)}
                            </Tag>
                        }.into_any(),
                        QuizVariant::Reading => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.reading)}
                            </Tag>
                        }.into_any(),
                        QuizVariant::Grammar => view! {
                            <Tag variant=Signal::derive(move || TagVariant::Filled)>
                                {t!(i18n, lesson.grammar)}
                            </Tag>
                        }.into_any(),
                    }
                }}
            </Show>
        </div>
    }
}

/// Audio button row rendered above the Card alongside tags (or inside,
/// as the first child). Hidden on reversed question side and kanji cards.
#[component]
pub fn LessonCardAudio(
    card_type: CardType,
    question_text: String,
    is_reversed: bool,
    #[prop(into)] audio_path: Option<String>,
) -> impl IntoView {
    let text = StoredValue::new(question_text);
    let path = StoredValue::new(audio_path);
    view! {
        <Show when=move || card_type != CardType::Kanji && !is_reversed>
            <div class="flex justify-center mb-2">
                <AudioButtons
                    text=text.get_value()
                    audio_path=path.get_value()
                    class=Signal::derive(|| "shrink-0".to_string())
                />
            </div>
        </Show>
    }
}

/// Audio button for the answer side of a card — always visible
/// (the user has already seen the answer, audio is a learning aid).
#[component]
pub fn LessonCardAnswerAudio(
    card_type: CardType,
    question_text: String,
    #[prop(into)] audio_path: Option<String>,
) -> impl IntoView {
    let text = StoredValue::new(question_text);
    let path = StoredValue::new(audio_path);
    view! {
        <Show when=move || card_type != CardType::Kanji>
            <div class="flex justify-center mb-2">
                <AudioButtons
                    text=text.get_value()
                    audio_path=path.get_value()
                    class=Signal::derive(|| "shrink-0".to_string())
                />
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grammar_variant_matches_grammar_card_type() {
        assert!(quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Grammar
        ));
    }

    #[test]
    fn meaning_variant_never_matches() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Meaning,
            CardType::Grammar
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Meaning,
            CardType::Vocabulary
        ));
    }

    #[test]
    fn reading_variant_never_matches() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Reading,
            CardType::Grammar
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Reading,
            CardType::Kanji
        ));
    }

    #[test]
    fn grammar_variant_does_not_match_other_card_types() {
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Vocabulary
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Kanji
        ));
        assert!(!quiz_variant_matches_card_type(
            QuizVariant::Grammar,
            CardType::Phrase
        ));
    }
}
