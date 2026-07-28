use crate::i18n::use_i18n;
use crate::ui_components::{FuriganaText, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;
use origa::dictionary::grammar::GrammarRule;
use origa::domain::{GrammarPracticeQuestion, User, generate_grammar_practice_questions};
use rand::rng;

const QUESTION_COUNT: usize = 20;

fn generate_questions(rule: &GrammarRule, user: &User) -> Vec<GrammarPracticeQuestion> {
    let mut rng = rng();
    match generate_grammar_practice_questions(rule, user.knowledge_set(), QUESTION_COUNT, &mut rng)
    {
        Ok(questions) => {
            if questions.is_empty() {
                tracing::warn!("Grammar practice generated 0 questions for rule");
            }
            questions
        },
        Err(e) => {
            tracing::error!("Failed to generate grammar practice questions: {e}");
            Vec::new()
        },
    }
}

#[derive(Clone, Copy, PartialEq)]
enum AnswerState {
    Idle,
    Answered { selected: usize, correct: bool },
}

fn option_classes(index: usize, state: AnswerState, correct_index: usize) -> String {
    let base = "p-3 border text-left transition-all";
    let ptr = "pointer-events-none";

    match state {
        AnswerState::Idle => format!(
            "{base} cursor-pointer quiz-option-neutral hover:bg-[var(--fg-black)] hover:text-[var(--bg-paper)]"
        ),
        AnswerState::Answered { selected, .. } => {
            let is_correct = index == correct_index;
            if index == selected && is_correct {
                format!("{base} {ptr} quiz-option-correct")
            } else if index == selected {
                format!("{base} {ptr} quiz-option-wrong")
            } else if is_correct {
                format!("{base} {ptr} quiz-option-correct")
            } else {
                format!("{base} {ptr} quiz-option-dimmed")
            }
        },
    }
}

#[component]
pub fn GrammarPracticeSession(
    rule: &'static GrammarRule,
    user: User,
    known_kanji: std::collections::HashSet<char>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let known_kanji = StoredValue::new(known_kanji);

    let initial = generate_questions(rule, &user);
    let has_questions = RwSignal::new(!initial.is_empty());
    let questions: RwSignal<Vec<GrammarPracticeQuestion>> = RwSignal::new(initial);
    let current_index: RwSignal<usize> = RwSignal::new(0);
    let correct_count: RwSignal<usize> = RwSignal::new(0);
    let selected: RwSignal<Option<usize>> = RwSignal::new(None);
    let is_completed: RwSignal<bool> = RwSignal::new(false);

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() {
            "grammar-practice-session".to_string()
        } else {
            val
        }
    };

    let reset = Callback::new(move |_: ()| {
        let new_questions = generate_questions(rule, &user);
        has_questions.set(!new_questions.is_empty());
        questions.set(new_questions);
        current_index.set(0);
        correct_count.set(0);
        selected.set(None);
        is_completed.set(false);
    });

    let on_option_click = move |index: usize| {
        if selected.get().is_some() {
            return;
        }
        let current = current_index.get();
        let qs = questions.get();
        let current_q = match qs.get(current) {
            Some(q) => q,
            None => return,
        };
        let is_correct = index == current_q.correct_index();
        if is_correct {
            correct_count.update(|c| *c += 1);
        }
        selected.set(Some(index));
    };

    let on_next = move || {
        let current = current_index.get();
        let total = questions.get().len();
        if current + 1 >= total {
            is_completed.set(true);
        } else {
            current_index.update(|i| *i += 1);
            selected.set(None);
        }
    };

    let current_question = Memo::new(move |_| {
        let idx = current_index.get();
        questions.get().get(idx).cloned()
    });

    let option_0 = Memo::new(move |_| {
        current_question
            .get()
            .as_ref()
            .and_then(|q| q.options().first().cloned())
    });
    let option_1 = Memo::new(move |_| {
        current_question
            .get()
            .as_ref()
            .and_then(|q| q.options().get(1).cloned())
    });
    let option_2 = Memo::new(move |_| {
        current_question
            .get()
            .as_ref()
            .and_then(|q| q.options().get(2).cloned())
    });
    let option_3 = Memo::new(move |_| {
        current_question
            .get()
            .as_ref()
            .and_then(|q| q.options().get(3).cloned())
    });

    let answer_state = Memo::new(move |_| match selected.get() {
        Some(sel) => {
            let correct_index = current_question
                .get()
                .as_ref()
                .map(|q| q.correct_index())
                .unwrap_or(0);
            AnswerState::Answered {
                selected: sel,
                correct: sel == correct_index,
            }
        },
        None => AnswerState::Idle,
    });

    let option_class = move |index: usize| -> String {
        let correct_index = current_question
            .get()
            .as_ref()
            .map(|q| q.correct_index())
            .unwrap_or(0);
        option_classes(index, answer_state.get(), correct_index)
    };

    let no_words_text = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .practice_no_words()
            .inner()
            .to_string()
    });

    let next_text = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .quiz_next()
            .inner()
            .to_string()
    });

    let correct_label = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .quiz_correct()
            .inner()
            .to_string()
    });
    let wrong_label = Signal::derive(move || {
        i18n.get_keys()
            .grammar_page()
            .quiz_wrong()
            .inner()
            .to_string()
    });

    let result_text = Memo::new(move |_| match answer_state.get() {
        AnswerState::Answered { correct: true, .. } => correct_label.get(),
        AnswerState::Answered { correct: false, .. } => wrong_label.get(),
        AnswerState::Idle => String::new(),
    });

    let result_class = Memo::new(move |_| match answer_state.get() {
        AnswerState::Answered { correct: true, .. } => {
            "text-[var(--success)] font-bold".to_string()
        },
        AnswerState::Answered { correct: false, .. } => "text-[var(--error)] font-bold".to_string(),
        AnswerState::Idle => String::new(),
    });

    let is_answered =
        Memo::new(move |_| matches!(answer_state.get(), AnswerState::Answered { .. }));

    let progress_pct = Memo::new(move |_| {
        let total = questions.get().len();
        if total == 0 {
            0.0
        } else {
            ((current_index.get() + 1) as f64 / total as f64) * 100.0
        }
    });

    let _ = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            // The session is mounted inline (always present on the page while a
            // quiz rule is open), so don't hijack keys typed into inputs or
            // contenteditable elements that share the page.
            let typing_target = ev
                .target()
                .as_ref()
                .and_then(|t| t.dyn_ref::<web_sys::HtmlElement>())
                .is_some_and(|el| {
                    let tag = el.tag_name();
                    tag.eq_ignore_ascii_case("INPUT")
                        || tag.eq_ignore_ascii_case("TEXTAREA")
                        || el.is_content_editable()
                });
            if typing_target || !has_questions.get() {
                return;
            }

            let key = ev.key();

            if is_completed.get() {
                if key == " " {
                    ev.prevent_default();
                    reset.run(());
                }
                return;
            }

            if selected.get().is_none() {
                if let Some(index) = key.parse::<usize>().ok().filter(|&i| (1..=4).contains(&i)) {
                    ev.prevent_default();
                    on_option_click(index - 1);
                    return;
                }
            }

            if selected.get().is_some() && key == " " {
                ev.prevent_default();
                on_next();
            }
        },
    );

    view! {
        <div class="space-y-4" data-testid=test_id_val>
            <Show when=move || !has_questions.get()>
                <div class="text-center py-8" data-testid="grammar-practice-no-words">
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {no_words_text}
                    </Text>
                </div>
            </Show>

            <Show when=move || has_questions.get() && !is_completed.get()>
                <div class="flex justify-between items-center">
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        <span data-testid="grammar-practice-progress">
                            {move || {
                                let fmt = i18n
                                    .get_keys()
                                    .grammar_page()
                                    .practice_progress()
                                    .inner()
                                    .to_string();
                                fmt.replacen("{}", &(current_index.get() + 1).to_string(), 1)
                                    .replacen("{}", &questions.get().len().to_string(), 1)
                            }}
                        </span>
                    </Text>
                    <Text size=TextSize::Small variant=TypographyVariant::Muted>
                        <span data-testid="grammar-practice-correct-count">
                            {move || {
                                let fmt = i18n
                                    .get_keys()
                                    .grammar_page()
                                    .practice_correct_count()
                                    .inner()
                                    .to_string();
                                fmt.replacen("{}", &correct_count.get().to_string(), 1)
                            }}
                        </span>
                    </Text>
                </div>

                <div class="h-1 bg-[var(--border-dark)] w-full">
                    <div
                        class="h-1 bg-[var(--accent-olive)] transition-all"
                        style=move || format!("width: {}%", progress_pct.get())
                    ></div>
                </div>

                <div class="text-[length:var(--text-default)] text-[var(--fg)]">
                    {move || {
                        let template = i18n
                            .get_keys()
                            .grammar_page()
                            .quiz_question()
                            .inner()
                            .to_string();
                        let word = current_question
                            .get()
                            .as_ref()
                            .map(|q| q.word_text().to_string())
                            .unwrap_or_default();
                        let parts: Vec<&str> = template.splitn(2, "{}").collect();
                        let before = parts.first().copied().unwrap_or("");
                        let after = parts.get(1).copied().unwrap_or("");
                        view! {
                            <span>{before.to_string()}</span>
                            <FuriganaText text=word known_kanji=known_kanji.get_value()/>
                            <span>{after.to_string()}</span>
                        }
                    }}
                </div>

                <div class="grid grid-cols-2 gap-2 sm:gap-3">
                    <button
                        class=move || option_class(0)
                        data-testid="grammar-practice-option-0"
                        on:click=move |_| on_option_click(0)
                    >
                        {move || view! {
                            <FuriganaText
                                text=option_0.get().unwrap_or_default()
                                known_kanji=known_kanji.get_value()
                            />
                        }}
                        <span class="kbd-hint ml-1">"[1]"</span>
                    </button>
                    <button
                        class=move || option_class(1)
                        data-testid="grammar-practice-option-1"
                        on:click=move |_| on_option_click(1)
                    >
                        {move || view! {
                            <FuriganaText
                                text=option_1.get().unwrap_or_default()
                                known_kanji=known_kanji.get_value()
                            />
                        }}
                        <span class="kbd-hint ml-1">"[2]"</span>
                    </button>
                    <button
                        class=move || option_class(2)
                        data-testid="grammar-practice-option-2"
                        on:click=move |_| on_option_click(2)
                    >
                        {move || view! {
                            <FuriganaText
                                text=option_2.get().unwrap_or_default()
                                known_kanji=known_kanji.get_value()
                            />
                        }}
                        <span class="kbd-hint ml-1">"[3]"</span>
                    </button>
                    <button
                        class=move || option_class(3)
                        data-testid="grammar-practice-option-3"
                        on:click=move |_| on_option_click(3)
                    >
                        {move || view! {
                            <FuriganaText
                                text=option_3.get().unwrap_or_default()
                                known_kanji=known_kanji.get_value()
                            />
                        }}
                        <span class="kbd-hint ml-1">"[4]"</span>
                    </button>
                </div>

                <Show when=move || is_answered.get()>
                    <div class="text-center">
                        <Text
                            size=TextSize::Default
                            class=Signal::derive(move || result_class.get())
                        >
                            {move || result_text.get()}
                        </Text>
                    </div>

                    // The Next button is always available after an answer;
                    // on the final question its handler (on_next) sets
                    // is_completed, which swaps the practice session for the
                    // completion screen. Hiding it on is_last (the previous
                    // behaviour) left the user stuck with no way to finish.
                    <div class="flex justify-center gap-3 mt-4">
                        <button
                            class="px-4 py-2 bg-[var(--accent-olive)] text-[var(--bg-paper)] border border-[var(--accent-olive)] hover:bg-[var(--bg-paper)] hover:text-[var(--accent-olive)] transition-colors cursor-pointer"
                            data-testid="grammar-practice-next-btn"
                            on:click=move |_| on_next()
                        >
                            {next_text} <span class="kbd-hint">"[Space]"</span>
                        </button>
                    </div>
                </Show>
            </Show>

            <Show when=move || is_completed.get()>
                <div class="text-center py-8" data-testid="grammar-practice-complete">
                    <Text size=TextSize::Large variant=TypographyVariant::Primary>
                        {move || {
                            i18n.get_keys().grammar_page().practice_complete().inner().to_string()
                        }}
                    </Text>
                    <div class="mt-3">
                        <Text size=TextSize::Default variant=TypographyVariant::Muted>
                            {move || {
                                let fmt = i18n
                                    .get_keys()
                                    .grammar_page()
                                    .practice_complete_stats()
                                    .inner()
                                    .to_string();
                                fmt.replacen("{}", &correct_count.get().to_string(), 1)
                                    .replacen("{}", &questions.get().len().to_string(), 1)
                            }}
                        </Text>
                    </div>
                    <div class="flex justify-center gap-3 mt-6">
                        <button
                            class="px-4 py-2 bg-[var(--accent-olive)] text-[var(--bg-paper)] border border-[var(--accent-olive)] hover:bg-[var(--bg-paper)] hover:text-[var(--accent-olive)] transition-colors cursor-pointer"
                            data-testid="grammar-practice-again-btn"
                            on:click=move |_| reset.run(())
                        >
                            {move || {
                                i18n.get_keys().grammar_page().practice_again().inner().to_string()
                            }} <span class="kbd-hint">"[Space]"</span>
                        </button>
                    </div>
                </div>
            </Show>
        </div>
    }
}
