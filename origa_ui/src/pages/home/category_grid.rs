use crate::i18n::{td_string, use_i18n};
use crate::ui_components::Card;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;
use origa::domain::CategoryProgress;

#[component]
pub fn CategoryProgressGrid(
    kanji_progress: Signal<CategoryProgress>,
    words_progress: Signal<CategoryProgress>,
    grammar_progress: Signal<CategoryProgress>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let grid_test_id = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    view! {
        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 items-stretch" data-testid=grid_test_id>
            <CategoryCard
                href="/kanji"
                icon=CategoryIcon::Kanji
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.kanji_label).to_string())
                progress=kanji_progress
                fill_class="bg-[var(--accent-terracotta)]"
                icon_color_class="text-[var(--accent-terracotta)]"
                test_id=Signal::derive(move || format!("{}-kanji", test_id.get()))
            />
            <CategoryCard
                href="/words"
                icon=CategoryIcon::Words
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.words_label).to_string())
                progress=words_progress
                fill_class="bg-[var(--accent-sage)]"
                icon_color_class="text-[var(--accent-sage)]"
                test_id=Signal::derive(move || format!("{}-words", test_id.get()))
            />
            <CategoryCard
                href="/grammar"
                icon=CategoryIcon::Grammar
                label=Signal::derive(move || td_string!(i18n.get_locale(), home.grammar_label).to_string())
                progress=grammar_progress
                fill_class="bg-[var(--fg-black)]"
                icon_color_class="text-[var(--fg-black)]"
                test_id=Signal::derive(move || format!("{}-grammar", test_id.get()))
            />
        </div>
    }
}

enum CategoryIcon {
    Kanji,
    Words,
    Grammar,
}

#[component]
fn CategoryCard(
    #[prop(into)] href: Signal<String>,
    icon: CategoryIcon,
    #[prop(into)] label: Signal<String>,
    progress: Signal<CategoryProgress>,
    #[prop(into)] fill_class: Signal<String>,
    #[prop(into)] icon_color_class: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let pct = Signal::derive(move || progress.get().percentage().min(100.0));
    let projected_pct = Signal::derive(move || progress.get().projected_percentage().min(100.0));
    let stats = Signal::derive(move || {
        let p = progress.get();
        format!("{} / {}", p.learned, p.total)
    });

    let card_test_id = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let icon_view = match icon {
        CategoryIcon::Kanji => view! {
            <div class=move || format!("{} font-serif text-[32px]", icon_color_class.get())>
                "魚"
            </div>
        }
        .into_any(),
        CategoryIcon::Words => view! {
            <div class=move || icon_color_class.get().clone()>
                <Icon icon=icondata::LuBookOpen width="32" height="32" />
            </div>
        }
        .into_any(),
        CategoryIcon::Grammar => view! {
            <div class=move || icon_color_class.get().clone()>
                <Icon icon=icondata::LuClipboardList width="32" height="32" />
            </div>
        }
        .into_any(),
    };

    view! {
        <A href=move || href.get() attr:class="block h-full">
            <Card
                shadow=Signal::from(true)
                class=Signal::derive(|| "interactive p-5 h-full".to_string())
                test_id=test_id
            >
                <div data-testid=card_test_id class="flex flex-col h-full justify-between">
                    <div>
                        {icon_view}
                        <div class="font-mono text-[11px] uppercase tracking-[0.15em] text-[var(--fg-muted)] mt-4">
                            {move || label.get()}
                        </div>
                        <div class="font-serif text-2xl text-[var(--fg-black)] mt-2">
                            {move || stats.get()}
                        </div>
                    </div>
                    <div class="progress-track mt-3">
                        <div
                            class=move || format!("progress-fill progress-fill-projected {}", fill_class.get())
                            style=move || format!("width: {:.0}%", projected_pct.get().min(100.0))
                        ></div>
                        <div
                            class=move || format!("progress-fill progress-fill-layered {}", fill_class.get())
                            style=move || format!("width: {:.0}%", pct.get())
                        ></div>
                    </div>
                </div>
            </Card>
        </A>
    }
}
