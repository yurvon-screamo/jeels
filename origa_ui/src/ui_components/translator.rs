use std::collections::HashSet;

use crate::pages::lesson::LessonContext;
use crate::ui_components::{MarkdownText, MarkdownVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos::wasm_bindgen::JsCast;
use leptos_use::use_event_listener;
use origa::domain::{NativeLanguage, TokenTranslation, lookup_tokens_translations, tokenize_text};

fn has_kanji(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}'
        )
    })
}

#[component]
pub fn TranslatorText(
    #[prop(into)] text: String,
    #[prop(optional, into, default = String::new().into())] class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
    #[prop(optional, into)] native_language: Option<Signal<NativeLanguage>>,
) -> impl IntoView {
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let native_lang: Option<Signal<NativeLanguage>> = native_language
        .or_else(|| use_context::<LessonContext>().map(|ctx| ctx.native_language.into()));

    let Some(native_lang) = native_lang else {
        return view! {
            <span class=move || format!("translator-text {}", class.get()) data-testid=test_id_val>
                <span class="translator-loading font-serif">{text.clone()}</span>
            </span>
        }
        .into_any();
    };

    let translations: RwSignal<Vec<TokenTranslation>> = RwSignal::new(vec![]);
    let expanded: RwSignal<Option<usize>> = RwSignal::new(None);
    let is_loaded: RwSignal<bool> = RwSignal::new(false);
    let container_ref = NodeRef::<leptos::html::Span>::new();

    let text_for_spawn = text.clone();
    spawn_local(async move {
        let lang = native_lang.get();
        let tokens = tokenize_text(&text_for_spawn).unwrap_or_default();
        translations.set(lookup_tokens_translations(&tokens, &lang, &text_for_spawn));
        is_loaded.set(true);
    });

    let _ = use_event_listener(document(), leptos::ev::click, {
        move |ev: leptos::ev::MouseEvent| {
            let mut should_close = true;
            if let Some(el) = container_ref.get()
                && let Some(target) = ev.target()
            {
                let target_node: Option<web_sys::Node> = target.dyn_into().ok();
                let el_node: &web_sys::Node = &el;
                should_close = !el_node.contains(target_node.as_ref());
            }
            if should_close {
                expanded.set(None);
            }
        }
    });

    let _ = use_event_listener(
        document(),
        leptos::ev::keydown,
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Escape" {
                expanded.set(None);
            }
        },
    );

    let indexed = move || -> Vec<(usize, TokenTranslation)> {
        translations.get().into_iter().enumerate().collect()
    };

    view! {
        <span
            class=move || format!("translator-text {}", class.get())
            node_ref=container_ref
            data-testid=test_id_val
        >
            <Show
                when=move || is_loaded.get()
                fallback=move || view! {
                    <span class="translator-loading font-serif">{text.clone()}</span>
                }
            >
                <For
                    each=indexed
                    key=|(idx, _)| *idx
                    children=move |(idx, token): (usize, TokenTranslation)| {
                        let surface = token.surface_form.clone();
                        let reading = token.reading.clone();
                        let base_form = token.base_form.clone();
                        let translation_text = token.translation.clone();
                        let grammar_label = token.grammar_label.clone();
                        let grammar_description = token.grammar_description.clone();
                        let clickable = token.pos.is_vocabulary_word() || grammar_label.is_some();
                        let has_kanji = has_kanji(&surface);
                        let show_base = base_form != surface;

                        let surface_view = if has_kanji {
                            view! {
                                <ruby class="furigana-ruby">
                                    {surface.clone()}
                                    <rp>"("</rp>
                                    <rt class="furigana-rt">{reading.clone()}</rt>
                                    <rp>")"</rp>
                                </ruby>
                            }.into_any()
                        } else {
                            view! { <span>{surface.clone()}</span> }.into_any()
                        };

                        // Store fields for the popup closure
                        let popup_data = StoredValue::new((
                            surface.clone(),
                            reading.clone(),
                            base_form.clone(),
                            grammar_label.clone(),
                            grammar_description.clone(),
                            translation_text.clone(),
                            show_base,
                        ));

                        if clickable {
                            view! {
                                <span class=move || {
                                    if expanded.get() == Some(idx) {
                                        "token-word token-word-active"
                                    } else {
                                        "token-word"
                                    }
                                }>
                                    <span
                                        class="token-surface"
                                        tabindex="0"
                                        on:click=move |ev: leptos::ev::MouseEvent| {
                                            ev.stop_propagation();
                                            expanded.update(|e| {
                                                *e = if *e == Some(idx) { None } else { Some(idx) };
                                            });
                                        }
                                        on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                            if ev.key() == "Enter" || ev.key() == " " {
                                                ev.prevent_default();
                                                expanded.update(|e| {
                                                    *e = if *e == Some(idx) { None } else { Some(idx) };
                                                });
                                            }
                                        }
                                    >
                                        {surface_view}
                                    </span>
                                    {move || {
                                        if expanded.get() == Some(idx) {
                                            let (s, r, bf, gl, gd, tt, sb) = popup_data.get_value();
                                            view! {
                                                <TokenPopup
                                                    surface=s
                                                    reading=r
                                                    base_form=Signal::derive(move || if sb { Some(bf.clone()) } else { None })
                                                    grammar_label=Signal::derive(move || gl.clone())
                                                    grammar_description=Signal::derive(move || gd.clone())
                                                    translation_text=Signal::derive(move || tt.clone())
                                                />
                                            }.into_any()
                                        } else {
                                            ().into_any()
                                        }
                                    }}
                                </span>
                            }.into_any()
                        } else {
                            view! {
                                <span class="token-plain">{surface}</span>
                            }.into_any()
                        }
                    }
                />
            </Show>
        </span>
    }.into_any()
}

/// Popup that shows translation/grammar info for a token. Measures its
/// viewport position on mount and shifts horizontally to stay on-screen.
#[component]
fn TokenPopup(
    surface: String,
    reading: String,
    #[prop(optional, into)] base_form: Signal<Option<String>>,
    #[prop(optional, into)] grammar_label: Signal<Option<String>>,
    #[prop(optional, into)] grammar_description: Signal<Option<String>>,
    #[prop(optional, into)] translation_text: Signal<Option<String>>,
) -> impl IntoView {
    let popup_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let shift_x: RwSignal<f64> = RwSignal::new(0.0);

    Effect::new(move |_| {
        if let Some(el) = popup_ref.get() {
            let rect = el.get_bounding_client_rect();
            let vw = web_sys::window()
                .and_then(|w| w.inner_width().ok())
                .and_then(|v| v.as_f64())
                .unwrap_or(400.0);
            let margin = 8.0_f64;
            let popup_right = rect.right();
            let popup_left = rect.left();

            if popup_right > vw - margin {
                shift_x.set((vw - margin) - popup_right);
            } else if popup_left < margin {
                shift_x.set(margin - popup_left);
            }
        }
    });

    view! {
        <div
            class="token-popup"
            node_ref=popup_ref
            style=move || format!("--token-popup-shift: {}px", shift_x.get())
            on:click=move |ev: leptos::ev::MouseEvent| ev.stop_propagation()
        >
            <div class="token-popup-surface">{surface}</div>
            <div class="token-popup-reading">{reading}</div>
            {move || {
                base_form.get().map(|bf| view! {
                    <div class="token-popup-reading">{bf}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                grammar_label.get().map(|label| view! {
                    <div class="token-popup-grammar">{label}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                grammar_description.get().map(|desc| view! {
                    <div class="token-popup-grammar-description">{desc}</div>
                }.into_any()).unwrap_or_else(|| ().into_any())
            }}
            {move || {
                translation_text.get().map(|text| {
                    view! {
                        <MarkdownText
                            content=Signal::derive(move || text.clone())
                            known_kanji=HashSet::new()
                            variant=Signal::derive(|| MarkdownVariant::Compact)
                            furigana=false
                        />
                    }.into_any()
                }).unwrap_or_else(|| ().into_any())
            }}
        </div>
    }
}
