use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant, Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use origa::domain::LessonEmptyDiagnosis;

/// Formats the next-review timestamp as a short local date-time label
/// (`dd.mm hh:mm`), matching the dashboard forecast label style.
fn format_next_review(next: chrono::DateTime<chrono::Utc>) -> String {
    use chrono::TimeZone;
    let local = chrono::Local.from_utc_datetime(&next.naive_utc());
    local.format("%d.%m %H:%M").to_string()
}

/// Rendered when a lesson selection comes out empty; renders nothing while
/// the diagnosis is still `None` (selection pending). Shows EVERY applicable
/// diagnosis block ([`LessonEmptyDiagnosis`]): deck exhaustion (import sets),
/// a spent daily new-card allowance (increase load), and the earliest future
/// review. Each block carries its own CTA.
///
/// The returned view is type-erased (`AnyView`) and the `None` guard lives
/// INSIDE the component: the call site in `LessonContent` then adds a flat
/// type to the lesson view tree instead of a `Show` wrapper + derive
/// closure, keeping the `origa_ui_bin` crate under its 128 query-depth
/// ceiling (see `origa_ui/AGENTS.md` §recursion_limit).
#[component]
pub fn LessonEmptyState(diagnosis: RwSignal<Option<LessonEmptyDiagnosis>>) -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    let go_to_sets = {
        let navigate = navigate.clone();
        Callback::new(move |_: leptos::ev::MouseEvent| {
            navigate("/sets", Default::default());
        })
    };
    let go_to_profile = Callback::new(move |_: leptos::ev::MouseEvent| {
        navigate("/profile", Default::default());
    });

    view! {
        <Show when=move || diagnosis.get().is_some()>
            <div data-testid="lesson-empty-state" class="py-8 max-w-xl mx-auto">
                <Card class=Signal::derive(|| "p-6 flex flex-col gap-4".to_string())>
                    <Text size=TextSize::Default variant=TypographyVariant::Muted>
                        {t!(i18n, lesson.empty_title)}
                    </Text>

                    <Show when=move || diagnosis.get().unwrap_or_default().deck_exhausted>
                        <div class="flex flex-col gap-2 items-center" data-testid="lesson-empty-deck-block">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {t!(i18n, lesson.empty_deck_exhausted)}
                            </Text>
                            <Button
                                test_id=Signal::derive(|| "lesson-empty-import-btn".to_string())
                                variant=Signal::derive(|| ButtonVariant::Olive)
                                on_click=go_to_sets
                            >
                                {t!(i18n, lesson.empty_import_sets)}
                            </Button>
                        </div>
                    </Show>

                    <Show when=move || diagnosis.get().unwrap_or_default().daily_limit_reached>
                        <div class="flex flex-col gap-2 items-center" data-testid="lesson-empty-limit-block">
                            <Text size=TextSize::Small variant=TypographyVariant::Muted>
                                {t!(i18n, lesson.empty_daily_limit)}
                            </Text>
                            <Button
                                test_id=Signal::derive(|| "lesson-empty-profile-btn".to_string())
                                variant=Signal::derive(|| ButtonVariant::Default)
                                on_click=go_to_profile
                            >
                                {t!(i18n, lesson.empty_increase_load)}
                            </Button>
                        </div>
                    </Show>

                    <Show
                        when=move || diagnosis.get().unwrap_or_default().next_review.is_some()
                        fallback=move || ().into_any()
                    >
                        <div class="font-mono text-[12px] text-[var(--fg-muted)]" data-testid="lesson-empty-next-review">
                            {move || {
                                let label = diagnosis
                                    .get()
                                    .and_then(|d| d.next_review)
                                    .map(format_next_review)
                                    .unwrap_or_default();
                                i18n.get_keys().lesson().empty_next_review().inner().to_string()
                                    .replace("{}", &label)
                            }}
                        </div>
                    </Show>
                </Card>
            </div>
        </Show>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_review_label_is_local_short_datetime() {
        // 2026-08-20 12:00 UTC — the label must be a non-empty dd.mm hh:mm
        // string regardless of the host timezone.
        let dt = chrono::DateTime::parse_from_rfc3339("2026-08-20T12:00:00Z")
            .expect("valid rfc3339")
            .with_timezone(&chrono::Utc);
        let label = format_next_review(dt);
        assert!(
            label.contains('.'),
            "label must use the dd.mm form, got: {label}"
        );
        assert!(
            label.contains(':'),
            "label must include the hh:mm time, got: {label}"
        );
    }
}
