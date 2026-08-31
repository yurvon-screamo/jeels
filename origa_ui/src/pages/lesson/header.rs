use super::acquaintance_state::{AcquaintanceContext, AcquaintanceStage};
use super::acquaintance_view::AcquaintanceHeaderStrip;
use super::lesson_progress::LessonProgress;
use super::lesson_state::LessonContext;
use crate::i18n::use_i18n;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn LessonHeader() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let lesson_ctx = use_context::<LessonContext>().expect("LessonContext not provided");
    let is_muted = lesson_ctx.is_muted;
    let lesson_state = lesson_ctx.lesson_state;
    let core_count = lesson_ctx.core_count;

    // Во время руки знакомства место LessonProgress занимает полоса руки:
    // прогресс живёт в общем хедере и не съедает полезную высоту карточки
    // (баг-репорт: пустой хедер + полоса над контентом).
    let hand_active = use_context::<AcquaintanceContext>().map(|acq| {
        Signal::derive(move || {
            acq.state
                .with(|state| state.stage != AcquaintanceStage::Inactive && state.hand.is_some())
        })
    });
    let show_lesson_progress = move || {
        hand_active
            .as_ref()
            .map(|signal| !signal.get())
            .unwrap_or(true)
    };

    let toggle_mute = move || {
        is_muted.update(|m| *m = !*m);
    };

    let current = Signal::derive(move || lesson_state.get().current_index + 1);
    let total = Signal::derive(move || lesson_state.get().card_ids.len());
    let core_count_signal = Signal::derive(move || core_count.get());

    let back_label = Signal::derive(move || i18n.get_keys().common().back().inner().to_string());

    view! {
        <div class="flex items-center gap-2 mb-2 shrink-0" data-testid="lesson-header">
            <button
                data-testid="lesson-back-btn"
                class="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground transition-colors shrink-0 cursor-pointer"
                on:click=move |_| navigate("/home", Default::default())
            >
                <Icon icon=icondata::LuArrowLeft width="14" height="14" />
                <span class="font-mono text-[11px] tracking-widest uppercase">{back_label}</span>
            </button>

            <div class="flex-1 min-w-0">
                <Show when=show_lesson_progress fallback=move || view! { <AcquaintanceHeaderStrip /> }>
                    <LessonProgress current=current total=total core_count=core_count_signal />
                </Show>
            </div>

            <button
                data-testid="lesson-mute-btn"
                class="p-1.5 text-muted-foreground hover:text-foreground transition-colors shrink-0 cursor-pointer"
                data-muted=move || if is_muted.get() { "true" } else { "false" }
                on:click=move |_| toggle_mute()
            >
                {move || if is_muted.get() {
                    view! { <Icon icon=icondata::LuVolumeX width="16" height="16" /> }
                        .into_any()
                } else {
                    view! { <Icon icon=icondata::LuVolume2 width="16" height="16" /> }
                        .into_any()
                }}
            </button>
        </div>
    }
}
