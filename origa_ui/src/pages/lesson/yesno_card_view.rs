use crate::i18n::*;
use crate::ui_components::{
    Button, ButtonVariant, Card, DisplayText, MarkdownText, MarkdownVariant, Text, TextSize,
    TypographyVariant, is_speech_supported, speak_word, stop_current_audio,
};
use leptos::prelude::*;
use origa::domain::{Card as DomainCard, NativeLanguage, YesNoCard};
use std::collections::HashSet;
use tracing::warn;

use super::answer_display::{CardAnswerDisplay, extract_card_answer};
use super::card_type::CardType;
use super::lesson_card_header::{CardHeaderAudio, LessonCardTagsQuiz};
use super::next_card_button::NextCardButton;
use super::quiz_result::QuizResult;
use super::quiz_result_display::QuizResultDisplay;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum YesNoResult {
    #[default]
    None,
    Correct,
    Incorrect,
    DontKnow,
}

impl YesNoResult {
    pub fn from_answer(is_correct: bool, user_said_yes: bool, showing_result: bool) -> Self {
        if !showing_result {
            return YesNoResult::None;
        }
        let answered_correctly = (is_correct && user_said_yes) || (!is_correct && !user_said_yes);
        if answered_correctly {
            YesNoResult::Correct
        } else {
            YesNoResult::Incorrect
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_answer_before_result_is_always_none() {
        assert_eq!(
            YesNoResult::from_answer(true, true, false),
            YesNoResult::None
        );
    }

    #[test]
    fn from_answer_correct_statement_yes_is_correct() {
        assert_eq!(
            YesNoResult::from_answer(true, true, true),
            YesNoResult::Correct
        );
    }

    #[test]
    fn from_answer_correct_statement_no_is_incorrect() {
        assert_eq!(
            YesNoResult::from_answer(true, false, true),
            YesNoResult::Incorrect
        );
    }

    #[test]
    fn from_answer_wrong_statement_no_is_correct() {
        assert_eq!(
            YesNoResult::from_answer(false, false, true),
            YesNoResult::Correct
        );
    }

    #[test]
    fn from_answer_wrong_statement_yes_is_incorrect() {
        assert_eq!(
            YesNoResult::from_answer(false, true, true),
            YesNoResult::Incorrect
        );
    }
}

#[component]
pub fn YesNoCardView(
    yesno_card: YesNoCard,
    show_result: Signal<bool>,
    selected_answer: Option<bool>,
    on_answer: Callback<bool>,
    on_dont_know: Callback<()>,
    dont_know_selected: Signal<bool>,
    native_language: NativeLanguage,
    #[prop(into)] known_kanji: Signal<HashSet<char>>,
    #[prop(default = Signal::derive(|| false))] waiting_for_next: Signal<bool>,
    #[prop(default = Callback::new(|_: ()| {}))] on_next_card: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let card = yesno_card.card().clone();
    let card_type = CardType::from(&card);
    let lang = native_language;
    let is_na_adj = super::na_adjective_helper::is_na_adjective_card(&card);
    let part_of_speech = card.vocabulary_part_of_speech();

    let word_raw = yesno_card.word().to_string();
    let display_word = StoredValue::new(if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&word_raw)
    } else {
        word_raw
    });
    let statement_value = StoredValue::new(yesno_card.statement().to_string());
    let is_statement_correct = yesno_card.is_correct();

    let question_text_raw = match card.question(&lang) {
        Ok(q) => q.text().to_string(),
        Err(e) => {
            warn!(
                card_type = ?card_type,
                content_key = %card.content_key(),
                error = %e,
                "Failed to get card question"
            );
            String::new()
        },
    };
    let question_text = StoredValue::new(question_text_raw.clone());
    let display_question = StoredValue::new(if is_na_adj {
        super::na_adjective_helper::append_na_suffix(&question_text_raw)
    } else {
        question_text_raw
    });

    let kanji_for_animation: StoredValue<Option<String>> = StoredValue::new(match &card {
        DomainCard::Kanji(_) => {
            let text = match card.question(&lang) {
                Ok(q) => q.text().to_string(),
                Err(e) => {
                    warn!(
                        card_type = ?card_type,
                        content_key = %card.content_key(),
                        error = %e,
                        "Failed to get card question"
                    );
                    String::new()
                },
            };
            Some(text)
        },
        _ => None,
    });

    let answer_data = extract_card_answer(&card, &native_language, &card_type);
    let answer_vocab_translations_stored = StoredValue::new(answer_data.translations);
    let answer_vocab_description_stored = StoredValue::new(answer_data.description);
    let answer_text_display_stored = StoredValue::new(answer_data.text);

    let lesson_ctx = use_context::<super::lesson_state::LessonContext>();
    Effect::new(move |_| {
        let is_muted = lesson_ctx
            .as_ref()
            .map(|ctx| ctx.is_muted.get_untracked())
            .unwrap_or(false);
        if !show_result.get() && card_type != CardType::Kanji && is_speech_supported() && !is_muted
        {
            speak_word(&question_text.get_value(), 1.0);
        }
    });

    on_cleanup(move || {
        stop_current_audio();
    });

    let yesno_result = move || {
        if dont_know_selected.get() && show_result.get() {
            return YesNoResult::DontKnow;
        }
        YesNoResult::from_answer(
            is_statement_correct,
            selected_answer.unwrap_or(false),
            show_result.get(),
        )
    };

    let yes_selected = selected_answer == Some(true);
    let no_selected = selected_answer == Some(false);

    let no_btn_class = Signal::derive(move || {
        if show_result.get() {
            if !is_statement_correct {
                "quiz-option-correct".to_string()
            } else if no_selected {
                "quiz-option-wrong anima-shake".to_string()
            } else {
                "quiz-option-dimmed".to_string()
            }
        } else {
            String::new()
        }
    });

    let yes_btn_class = Signal::derive(move || {
        if show_result.get() {
            if is_statement_correct {
                "quiz-option-correct".to_string()
            } else if yes_selected {
                "quiz-option-wrong anima-shake".to_string()
            } else {
                "quiz-option-dimmed".to_string()
            }
        } else {
            String::new()
        }
    });

    let yes_variant = Signal::derive(move || {
        if show_result.get() {
            ButtonVariant::Default
        } else {
            ButtonVariant::Olive
        }
    });

    let correct_answer_text = move || {
        if is_statement_correct {
            i18n.get_keys().lesson().yes().inner().to_string()
        } else {
            i18n.get_keys().lesson().no().inner().to_string()
        }
    };

    view! {
        <div class="flex flex-col">
            <LessonCardTagsQuiz
                card_type=card_type
                part_of_speech=part_of_speech
                audio=Signal::derive(move || Some(CardHeaderAudio {
                    card_type,
                    question_text: display_question.get_value(),
                    is_reversed: false,
                    audio_path: None,
                }))
            />
            <Card class=Signal::derive(|| super::LESSON_CARD_CLASS.to_string()) shadow=true test_id="lesson-card-root">

            <div class="flex-1 flex flex-col justify-center">
                <div class="text-center mb-3 sm:mb-6">
                    <Show when=move || kanji_for_animation.get_value().is_none()>
                        <div class="mb-2 sm:mb-3">
                            <MarkdownText
                                content=Signal::derive(move || display_word.get_value())
                                known_kanji=known_kanji.get()
                                variant=Signal::derive(|| MarkdownVariant::Large)
                            />
                        </div>
                    </Show>

                    <Show when=move || kanji_for_animation.get_value().is_some()>
                        {move || {
                            kanji_for_animation.get_value().map(|kanji: String| {
                                view! {
                                    <div class="mb-2 sm:mb-3">
                                        <DisplayText>
                                            {kanji}
                                        </DisplayText>
                                    </div>
                                }
                            })
                        }}
                    </Show>

                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {move || statement_value.get_value()}
                    </Text>

                    <Show when=move || !show_result.get()>
                        <Text size=TextSize::Default variant=TypographyVariant::Muted class="mt-4">
                            {t!(i18n, lesson.is_this_correct)}
                        </Text>
                    </Show>
                </div>

                <Show when=move || !show_result.get()>
                    <div class="grid grid-cols-2 gap-3">
                        <Button
                            test_id=Signal::derive(|| "yesno-no-btn".to_string())
                            variant=Signal::derive(|| ButtonVariant::Default)
                            class=no_btn_class
                            disabled=Signal::derive(move || show_result.get())
                            on_click=Callback::new(move |_| on_answer.run(false))
                        >
                            {t!(i18n, lesson.no)} <span class="kbd-hint">"[1]"</span>
                        </Button>

                        <Button
                            test_id=Signal::derive(|| "yesno-yes-btn".to_string())
                            variant=yes_variant
                            class=yes_btn_class
                            disabled=Signal::derive(move || show_result.get())
                            on_click=Callback::new(move |_| on_answer.run(true))
                        >
                            {t!(i18n, lesson.yes)} <span class="kbd-hint">"[2]"</span>
                        </Button>
                    </div>
                    <button
                        data-testid="yesno-dont-know-btn"
                        class=move || {
                            let base = "w-full mt-2 p-2 sm:p-4 border text-center transition-all cursor-pointer flex items-center justify-center gap-2";
                            if dont_know_selected.get() {
                                format!("{} quiz-option-neutral ring-2 ring-[var(--accent-olive)]", base)
                            } else if show_result.get() {
                                format!("{} quiz-option-dimmed pointer-events-none", base)
                            } else {
                                format!("{} quiz-option-neutral", base)
                            }
                        }
                        on:click=move |_| {
                            if !show_result.get() {
                                on_dont_know.run(());
                            }
                        }
                    >
                        <Text size=TextSize::Default>{t!(i18n, lesson.dont_know)}</Text>
                        <span class="kbd-hint text-[var(--fg-muted)] text-xs font-mono">{t!(i18n, lesson.space_key)}</span>
                    </button>
                </Show>

                <Show when=move || show_result.get()>
                    <QuizResultDisplay quiz_result=QuizResult::from(yesno_result()) />

                    // Show the correct answer (Да/Нет) on every result —
                    // as a confirmation when correct and as feedback when
                    // not, same pattern as the single-quiz card.
                    <div class="mt-2 text-center">
                        <Text size=TextSize::Small variant=TypographyVariant::Muted>
                            {t!(i18n, lesson.correct_answer)}
                            <span class="font-semibold">
                                {correct_answer_text}
                            </span>
                        </Text>
                    </div>
                </Show>

                <Show when=move || {
                    show_result.get()
                        && super::quiz_card::should_show_answer_display(
                            QuizResult::from(yesno_result()),
                            card_type,
                        )
                }>
                    <CardAnswerDisplay
                        translations=Signal::derive(move || answer_vocab_translations_stored.get_value())
                        description=Signal::derive(move || answer_vocab_description_stored.get_value())
                        text=Signal::derive(move || answer_text_display_stored.get_value())
                        known_kanji=known_kanji
                    />
                </Show>

                <Show when=move || waiting_for_next.get() && show_result.get()>
                    <NextCardButton on_next_card=on_next_card />
                </Show>
            </div>
        </Card>
        </div>
    }
}
