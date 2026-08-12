use crate::i18n::{t, use_i18n};
use crate::pages::words::add_words_preview_modal_handlers::create_preview_modal_handlers;
use crate::pages::words::add_words_preview_modal_state::{
    AnalysisStage, InputMode, PreviewModalState, analysis_stage,
};
use crate::pages::words::analyzed_word_item::AnalyzedWordItem;
use crate::pages::words::anki_import_stage::AnkiImportStage;
use crate::pages::words::audio_input_stage::AudioInputStage;
use crate::pages::words::image_input_stage::ImageInputStage;
use crate::repository::HybridUserRepository;
use crate::ui_components::{
    Alert, AlertType, Button, ButtonVariant, Drawer, Input, TabItem, Tabs, Text, TextSize,
    TypographyVariant,
};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use origa::domain::User;
use origa::traits::UserRepository;
use origa::use_cases::AnalyzedWord;

#[component]
pub fn AddWordsPreviewModal(
    is_open: RwSignal<bool>,
    refresh_trigger: RwSignal<u32>,
) -> impl IntoView {
    let i18n = use_i18n();
    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");

    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let repo_for_effect = repository.clone();
    let disposed = StoredValue::new(());

    Effect::new(move |_| {
        let repo = repo_for_effect.clone();
        spawn_local(async move {
            if let Ok(Some(user)) = repo.get_current_user().await {
                if disposed.is_disposed() {
                    return;
                }
                current_user.set(Some(user));
            }
        });
    });

    let known_kanji = Memo::new(move |_| {
        current_user
            .get()
            .map(|u| u.knowledge_set().get_known_kanji())
            .unwrap_or_default()
    });

    let state = PreviewModalState::new(is_open, refresh_trigger);
    let analyzed_words = state.analyzed_words;
    let input_text = state.input_text;
    let is_analyzing = state.is_analyzing;
    let error_message = state.error_message;
    let selected_words = state.selected_words;
    let is_creating = state.is_creating;
    let input_mode = state.input_mode;
    let active_tab = state.active_tab;
    let handlers = create_preview_modal_handlers(state.clone(), is_open);

    Effect::new({
        let state = state.clone();
        move |_| {
            if !is_open.get() {
                state.reset();
            }
        }
    });

    let has_analyzed = state.has_analyzed;

    let tabs = Signal::derive(move || {
        let mut items = vec![
            TabItem {
                id: "text".to_string(),
                label: i18n.get_keys().words().tab_text().inner().to_string(),
            },
            TabItem {
                id: "image".to_string(),
                label: i18n.get_keys().words().tab_image().inner().to_string(),
            },
        ];
        items.push(TabItem {
            id: "anki".to_string(),
            label: i18n.get_keys().words().tab_anki().inner().to_string(),
        });
        items.push(TabItem {
            id: "audio".to_string(),
            label: i18n.get_keys().words().tab_audio().inner().to_string(),
        });
        items
    });

    Effect::new({
        let state = state.clone();
        move || {
            let tab = state.active_tab.get();
            state.input_mode.set(if tab == "image" {
                InputMode::Image
            } else if tab == "anki" {
                InputMode::Anki
            } else if tab == "audio" {
                InputMode::Audio
            } else {
                InputMode::Text
            });
        }
    });

    let on_text_extracted = {
        let state = state.clone();
        Callback::new(move |text: String| {
            state.set_extracted_text(text);
        })
    };

    let on_ocr_error = {
        Callback::new(move |_msg: String| {
            error_message.set(None);
        })
    };

    let on_switch_to_text = {
        Callback::new(move |_| {
            input_mode.set(InputMode::Text);
            active_tab.set("text".to_string());
        })
    };

    view! {
        <Drawer
            is_open=is_open
            title=Signal::derive(move || i18n.get_keys().words().add_words().inner().to_string())
            test_id="words-add-drawer"
        >
            <div class="space-y-4">
                {move || {
                    let words = analyzed_words.get();
                    let stage = analysis_stage(words.len(), is_analyzing.get());
                    match stage {
                        AnalysisStage::Analyzing => view! {
                            <div class="space-y-4">
                                <Tabs tabs=tabs active=active_tab test_id=Signal::derive(|| "words-add-tabs".to_string()) />
                                <div class="flex items-center justify-center py-8">
                                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                                        {t!(i18n, words.analyzing)}
                                    </Text>
                                </div>
                            </div>
                        }.into_any(),
                        AnalysisStage::Preview => view! {
                            <PreviewStage
                                analyzed_words=words
                                selected_words=selected_words
                                known_kanji=known_kanji.get()
                                is_creating=is_creating
                                on_word_toggle=handlers.on_word_toggle
                                on_cancel=handlers.on_cancel
                                on_create=handlers.on_create
                            />
                        }.into_any(),
                        AnalysisStage::Input => view! {
                            <div class="space-y-4">
                                {move || {
                                    if has_analyzed.get() && analyzed_words.get().is_empty() {
                                        Some(view! {
                                            <Alert
                                                alert_type=Signal::derive(|| AlertType::Warning)
                                                title=Signal::derive(move || i18n.get_keys().words().words_not_found().inner().to_string())
                                                message=Signal::derive(move || i18n.get_keys().words().words_not_found_hint().inner().to_string())
                                                test_id=Signal::derive(|| "words-no-results".to_string())
                                            />
                                        })
                                    } else {
                                        None
                                    }
                                }}
                                <Tabs tabs=tabs active=active_tab test_id=Signal::derive(|| "words-add-tabs".to_string()) />
                                {move || {
                                    let mode = input_mode.get();
                                    match mode {
                                        InputMode::Text => view! {
                                            <InputStage
                                                input_text=input_text
                                                is_analyzing=is_analyzing
                                                error_message=error_message
                                                on_analyze=handlers.on_analyze
                                            />
                                        }.into_any(),
                                        InputMode::Anki => {
                                            view! {
                                                <AnkiImportStage
                                                    is_open=is_open
                                                    refresh_trigger=refresh_trigger
                                                    test_id=Signal::derive(|| "words-drawer-anki".to_string())
                                                />
                                            }.into_any()
                                        },
                                        InputMode::Image => view! {
                                            <ImageInputStage
                                                is_open=is_open
                                                on_text_extracted=on_text_extracted
                                                on_error=on_ocr_error
                                                on_switch_to_text=on_switch_to_text
                                            />
                                        }.into_any(),
                                        InputMode::Audio => view! {
                                            <AudioInputStage
                                                is_open=Signal::derive(move || is_open.get())
                                                on_text_extracted=on_text_extracted
                                                on_error=on_ocr_error
                                                on_switch_to_text=on_switch_to_text
                                            />
                                        }.into_any(),
                                    }
                                }}
                            </div>
                        }.into_any(),
                    }
                }}
            </div>
        </Drawer>
    }
}

#[component]
fn PreviewStage(
    analyzed_words: Vec<AnalyzedWord>,
    selected_words: RwSignal<std::collections::HashSet<String>>,
    known_kanji: std::collections::HashSet<char>,
    is_creating: RwSignal<bool>,
    on_word_toggle: Callback<String>,
    on_cancel: Callback<MouseEvent>,
    on_create: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    let analyzed_words_count = analyzed_words.len();
    let new_words_count = analyzed_words.iter().filter(|w| !w.is_known).count();

    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                {move || {
                    i18n.get_keys().words().found_words().inner().to_string()
                        .replacen("{}", &analyzed_words_count.to_string(), 1)
                        .replacen("{}", &new_words_count.to_string(), 1)
                }}
            </Text>
        </div>
        <div class="space-y-2 overflow-y-auto">
            <For
                each=move || analyzed_words.clone()
                key=|word| word.base_form.clone()
                children=move |word| {
                    let base_form = word.base_form.clone();
                    view! {
                        <AnalyzedWordItem
                            analyzed_word=word
                            selected_words=selected_words
                            known_kanji=known_kanji.clone()
                            on_toggle=Callback::new(move |_| on_word_toggle.run(base_form.clone()))
                        />
                    }
                }
            />
        </div>
        <div class="flex gap-2 justify-between">
            <Button
                variant=ButtonVariant::Ghost
                on_click=on_cancel
                test_id="words-drawer-cancel-btn"
            >
                {t!(i18n, words.cancel)}
            </Button>
            <Button
                variant=ButtonVariant::Olive
                disabled=Signal::derive(move || {
                    selected_words.get().is_empty()
                        || is_creating.get()
                })
                on_click=Callback::new(move |_| on_create.run(()))
                test_id="words-drawer-add-btn"
            >
                {move || {
                    if is_creating.get() {
                        t!(i18n, words.creating).into_any()
                    } else {
                        t!(i18n, words.add_selected).into_any()
                    }
                }}
            </Button>
        </div>
    }
}

#[component]
fn InputStage(
    input_text: RwSignal<String>,
    is_analyzing: RwSignal<bool>,
    error_message: RwSignal<Option<String>>,
    on_analyze: Callback<()>,
) -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <div>
            <Text size=TextSize::Small variant=TypographyVariant::Muted class=Signal::derive(|| "mb-2".to_string())>
                {t!(i18n, words.enter_japanese)}
            </Text>
            <Input
                value=input_text
                placeholder=Signal::derive(|| "例えば、本を読みます。".to_string())
                rows=Signal::derive(|| Some(10))
                test_id="words-drawer-textarea"
            />
        </div>
        {move || {
            error_message.get().map(move |msg| view! {
                <Alert
                    alert_type=Signal::derive(|| AlertType::Error)
                    title=Signal::derive(move || i18n.get_keys().words().error().inner().to_string())
                    message=Signal::derive(move || msg.clone())
                />
            })
        }}
        <Button
            variant=ButtonVariant::Olive
            disabled=Signal::derive(move || {
                input_text.get().trim().is_empty()
                    || is_analyzing.get()
            })
            on_click=Callback::new(move |_| on_analyze.run(()))
            test_id="words-drawer-analyze-btn"
        >
            {move || {
                if is_analyzing.get() {
                    t!(i18n, words.analyzing).into_any()
                } else {
                    t!(i18n, words.analyze).into_any()
                }
            }}
        </Button>
    }
}
