use leptos::prelude::*;

/// Заполнение полосы руки в фазе показа: пройденные слайды — полные ячейки,
/// текущий и будущие — пустые (движение видно с первого «Дальше»).
pub(crate) fn presentation_fill(total: usize, slide_index: usize) -> Vec<u8> {
    (0..total)
        .map(|index| if index < slide_index { 3 } else { 0 })
        .collect()
}

/// Полоса прогресса руки: ячейка на карту, заполнение — видимый успех
/// карты в активной подфазе (0..=3). Ячейки не исчезают: рука не «тает»
/// (docs/acquaintance-mode.md, правило «Прогресс виден без таяния»).
#[component]
pub fn HandProgressStrip(
    #[prop(into)] total: Signal<usize>,
    #[prop(into)] progress: Signal<Vec<u8>>,
    #[prop(into)] label: Signal<String>,
) -> impl IntoView {
    let closed_cells = move || {
        progress
            .get()
            .iter()
            .filter(|fill| **fill >= 3 || **fill == u8::MAX)
            .count()
    };
    view! {
        <div
            data-testid="acquaintance-strip"
            class="flex gap-1 items-center"
            role="progressbar"
            aria-label=move || label.get()
            aria-valuemin="0"
            aria-valuemax=move || total.get().to_string()
            aria-valuenow=move || closed_cells().to_string()
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

#[cfg(test)]
mod presentation_fill_tests {
    use super::*;

    #[test]
    fn passed_slides_fill_cells_before_the_current_one() {
        // Arrange / Act: рука из 3 слайдов, юзер на втором
        let fill = presentation_fill(3, 1);

        // Assert: первый слайд пройден — полная ячейка, дальше пусто
        assert_eq!(fill, vec![3, 0, 0]);
    }

    #[test]
    fn start_of_presentation_leaves_all_cells_empty() {
        assert_eq!(presentation_fill(4, 0), vec![0, 0, 0, 0]);
    }

    #[test]
    fn last_slide_keeps_the_final_cell_empty() {
        // Текущий слайд не считается пройденным: последняя ячейка
        // заполняется только переходом в тренировку.
        assert_eq!(presentation_fill(2, 1), vec![3, 0]);
    }
}
