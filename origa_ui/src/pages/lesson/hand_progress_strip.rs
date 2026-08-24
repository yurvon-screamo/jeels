use leptos::prelude::*;

/// Полоса прогресса руки: ячейка на карту, заполнение — видимый успех
/// карты в активной подфазе (0..=3). Ячейки не исчезают: рука не «тает»
/// (docs/acquaintance-mode.md, правило «Прогресс виден без таяния»).
#[component]
pub fn HandProgressStrip(
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] progress: Signal<Vec<u8>>,
) -> impl IntoView {
    view! {
        <div
            data-testid="acquaintance-strip"
            class="flex gap-1 items-center"
            aria-hidden="true"
        >
            {move || {
                let total = total.get();
                let progress = progress.get();
                (0..total)
                    .map(|index| {
                        let filled = progress.get(index).copied().unwrap_or(0);
                        if filled == u8::MAX {
                            // Карта выведена из руки: ячейка схлопывается.
                            return view! { <div class="w-0 h-full" /> }.into_any();
                        }
                        let percent = (filled.min(3)) as f64 / 3.0 * 100.0;
                        view! {
                            <div class="w-6 h-2 border border-[var(--border-dark)] bg-[var(--bg-paper)] overflow-hidden">
                                <div
                                    class="h-full bg-[var(--accent-olive)]"
                                    style=format!("width: {percent}%")
                                ></div>
                            </div>
                        }
                            .into_any()
                    })
                    .collect_view()
            }}
        </div>
    }
}
