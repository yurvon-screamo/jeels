use crate::i18n::{t, use_i18n};
use crate::pages::home::category_grid::CategoryProgressGrid;
use crate::ui_components::{Card, DisplayText, Tag, TagVariant, Text, TextSize};
use leptos::prelude::*;
use origa::domain::{JlptProgress, LevelProgressDetail};

#[component]
pub fn JlptProgressCard(
    jlpt_progress: Signal<JlptProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let current_level = Signal::derive(move || jlpt_progress.get().current_level());
    let level_detail = Signal::derive(move || {
        let level = jlpt_progress.get().current_level();
        jlpt_progress.get().level_progress(level).cloned()
    });
    let overall_pct = Signal::derive(move || {
        level_detail
            .get()
            .map(|d| d.overall_percentage())
            .unwrap_or(0.0)
    });
    let projected_pct = Signal::derive(move || {
        level_detail
            .get()
            .map(|d| d.overall_projected_percentage())
            .unwrap_or(0.0)
    });

    view! {
        <Card
            shadow=Signal::from(true)
            class=Signal::derive(|| "p-4 sm:p-6 lg:p-8".to_string())
            test_id=test_id
        >
            <div
                class="flex items-center gap-2 sm:gap-4 mb-6"
                data-testid=move || {
                    let val = test_id.get();
                    if val.is_empty() { None } else { Some(format!("{}-progress", val)) }
                }
            >
                <Tag
                    variant=Signal::from(TagVariant::Default)
                    test_id=Signal::derive(move || format!("{}-stamp", test_id.get()))
                >
                    {move || format!("JLPT {}", current_level.get().code())}
                </Tag>
                <div class="flex-1 progress-track">
                    <div
                        class="progress-fill progress-fill-projected"
                        data-testid=move || {
                            let val = test_id.get();
                            if val.is_empty() { None } else { Some(format!("{}-progress-projected", val)) }
                        }
                        style=move || format!("width: {:.0}%", projected_pct.get().min(100.0))
                    ></div>
                    <div
                        class="progress-fill progress-fill-layered"
                        style=move || format!("width: {:.0}%", overall_pct.get().min(100.0))
                    ></div>
                </div>
                <DisplayText
                    class=Signal::derive(|| "text-sm".to_string())
                    test_id=Signal::derive(move || format!("{}-pct", test_id.get()))
                >
                    {move || format!("{:.0}%", overall_pct.get())}
                </DisplayText>
            </div>

            <CategoryDetailSection detail=level_detail test_id=test_id />
        </Card>
    }
}

#[component]
fn CategoryDetailSection(
    detail: Signal<Option<LevelProgressDetail>>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let is_expanded = RwSignal::new(false);
    let kanji = Signal::derive(move || detail.get().map(|d| d.kanji.clone()));
    let words = Signal::derive(move || detail.get().map(|d| d.words.clone()));
    let grammar = Signal::derive(move || detail.get().map(|d| d.grammar.clone()));

    let section_test_id = move || {
        let val = test_id.get();
        if val.is_empty() {
            None
        } else {
            Some(format!("{}-categories", val))
        }
    };

    view! {
        <div data-testid=section_test_id>
            <button
                class="flex items-center justify-between w-full cursor-pointer select-none"
                on:click=move |_| is_expanded.update(|v| *v = !*v)
                aria-expanded=move || is_expanded.get()
                aria-controls=move || format!("{}-panel", test_id.get())
                id=move || format!("{}-toggle", test_id.get())
                data-testid=move || {
                    let val = test_id.get();
                    if val.is_empty() { None } else { Some(format!("{}-toggle", val)) }
                }
            >
                <Text size=Signal::from(TextSize::Small)>
                    <span>{t!(i18n, home.more)}</span>
                </Text>
                <span class="text-sm transition-transform" class:rotate-180=is_expanded aria-hidden="true">
                    "▼"
                </span>
            </button>

            <Show when=move || is_expanded.get()>
                <div class="mt-4" id=move || format!("{}-panel", test_id.get()) role="region" aria-labelledby=move || format!("{}-toggle", test_id.get())>
                    <CategoryProgressGrid
                        kanji_progress=Signal::derive(move || kanji.get().unwrap_or_default())
                        words_progress=Signal::derive(move || words.get().unwrap_or_default())
                        grammar_progress=Signal::derive(move || grammar.get().unwrap_or_default())
                        test_id=Signal::derive(move || format!("{}-expanded", test_id.get()))
                    />
                </div>
            </Show>
        </div>
    }
}
