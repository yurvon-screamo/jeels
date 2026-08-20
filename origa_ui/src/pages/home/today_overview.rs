use super::dashboard_stats::CompletionForecast;
use crate::i18n::{t, td_string, use_i18n};
use crate::ui_components::{Card, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::TodayOverview;

fn delta_color(delta: i32, is_positive_good: bool) -> &'static str {
    if delta == 0 {
        "var(--fg-muted)"
    } else if is_positive_good == (delta > 0) {
        "var(--success)"
    } else {
        "var(--error)"
    }
}

fn format_delta(delta: i32) -> String {
    if delta > 0 {
        format!("▲ +{}", delta)
    } else if delta < 0 {
        format!("▼ {}", delta)
    } else {
        "0".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_delta_positive_uses_up_arrow() {
        assert_eq!(format_delta(5), "▲ +5");
    }

    #[test]
    fn format_delta_negative_uses_down_arrow() {
        assert_eq!(format_delta(-3), "▼ -3");
    }

    #[test]
    fn format_delta_zero_is_plain_zero() {
        assert_eq!(format_delta(0), "0");
    }

    #[test]
    fn delta_color_positive_good_direction_is_success() {
        // New words learned: up is good.
        assert_eq!(delta_color(4, true), "var(--success)");
        // Words regressed to hard: up is bad.
        assert_eq!(delta_color(4, false), "var(--error)");
        assert_eq!(delta_color(-4, false), "var(--success)");
    }

    #[test]
    fn delta_color_zero_is_muted() {
        assert_eq!(delta_color(0, true), "var(--fg-muted)");
    }
}

#[component]
pub fn TodayOverviewCard(
    overview: Signal<TodayOverview>,
    forecast: Signal<CompletionForecast>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    let total = Signal::derive(move || overview.get().total());
    let new_count = Signal::derive(move || overview.get().new_count);
    let learned_count = Signal::derive(move || overview.get().learned_count);
    let in_progress_count = Signal::derive(move || overview.get().in_progress_count);
    let difficult_count = Signal::derive(move || overview.get().difficult_count);

    let new_delta = Signal::derive(move || overview.get().new_delta);
    let learned_delta = Signal::derive(move || overview.get().learned_delta);
    let in_progress_delta = Signal::derive(move || overview.get().in_progress_delta);
    let difficult_delta = Signal::derive(move || overview.get().difficult_delta);

    let max_count = Signal::derive(move || {
        let ov = overview.get();
        ov.new_count
            .max(ov.learned_count)
            .max(ov.in_progress_count)
            .max(ov.difficult_count)
            .max(1)
    });

    let progress_pct = move |count: usize| -> String {
        if count == 0 {
            "0%".to_string()
        } else {
            let pct = (count as f64 / max_count.get() as f64 * 100.0).min(100.0);
            format!("{:.0}%", pct)
        }
    };

    let days_label = Signal::derive(move || {
        let locale = i18n.get_locale();
        td_string!(locale, home.days_label)
    });

    view! {
        <Card shadow=true class=Signal::derive(|| "p-6 h-full".to_string()) test_id=test_id>
            <div class="flex flex-col h-full">
                <div class="flex items-center">
                    <Text
                        size=TextSize::Small
                        variant=TypographyVariant::Muted
                        uppercase=true
                        tracking_widest=true
                    >
                        {move || {
                            let locale = i18n.get_locale();
                            format!(
                                "{} {} {}",
                                td_string!(locale, home.total_label),
                                total.get(),
                                td_string!(locale, home.cards_suffix)
                            )
                        }}
                    </Text>

                    <div class="ml-auto">
                        <Show when=move || forecast.get().days_remaining.is_some()>
                            <span class="font-mono text-[12px] text-[var(--fg-muted)]">
                                {move || {
                                    let fc = forecast.get();
                                    let days = fc.days_remaining.unwrap_or(0);
                                    format!(
                                        "~{} {} · {}",
                                        days,
                                        days_label.get(),
                                        fc.target_date_label
                                    )
                                }}
                            </span>
                        </Show>

                        <Show when=move || forecast.get().is_all_studied>
                            <span class="font-mono text-[12px] text-[var(--accent-olive)]">
                                {t!(i18n, home.all_studied)}
                            </span>
                        </Show>
                    </div>
                </div>

                <div class="status-grid mt-4">
                    <div class="status-card status-card--new">
                        <div class="status-number">
                            <span>{move || new_count.get()}</span>
                            <Show when=move || new_delta.get().is_some()>
                                <span
                                    class="status-delta"
                                    style=move || format!("color: {}", delta_color(new_delta.get().unwrap_or(0), false))
                                >
                                    {move || format_delta(new_delta.get().unwrap_or(0))}
                                </span>
                            </Show>
                        </div>
                        <span class="status-label">{t!(i18n, home.new_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(new_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card status-card--learned">
                        <div class="status-number">
                            <span>{move || learned_count.get()}</span>
                            <Show when=move || learned_delta.get().is_some()>
                                <span
                                    class="status-delta"
                                    style=move || format!("color: {}", delta_color(learned_delta.get().unwrap_or(0), true))
                                >
                                    {move || format_delta(learned_delta.get().unwrap_or(0))}
                                </span>
                            </Show>
                        </div>
                        <span class="status-label">{t!(i18n, home.learned_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(learned_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card status-card--in-progress">
                        <div class="status-number">
                            <span>{move || in_progress_count.get()}</span>
                            <Show when=move || in_progress_delta.get().is_some()>
                                <span
                                    class="status-delta"
                                    style=move || format!(
                                        "color: {}",
                                        delta_color(in_progress_delta.get().unwrap_or(0), false)
                                    )
                                >
                                    {move || format_delta(in_progress_delta.get().unwrap_or(0))}
                                </span>
                            </Show>
                        </div>
                        <span class="status-label">{t!(i18n, home.in_progress_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(in_progress_count.get()))
                            ></div>
                        </div>
                    </div>

                    <div class="status-card status-card--difficult">
                        <div class="status-number">
                            <span>{move || difficult_count.get()}</span>
                            <Show when=move || difficult_delta.get().is_some()>
                                <span
                                    class="status-delta"
                                    style=move || format!(
                                        "color: {}",
                                        delta_color(difficult_delta.get().unwrap_or(0), false)
                                    )
                                >
                                    {move || format_delta(difficult_delta.get().unwrap_or(0))}
                                </span>
                            </Show>
                        </div>
                        <span class="status-label">{t!(i18n, home.difficult_status)}</span>
                        <div class="status-progress">
                            <div
                                class="status-progress-fill"
                                style=move || format!("width: {}", progress_pct(difficult_count.get()))
                            ></div>
                        </div>
                    </div>
                </div>
            </div>
        </Card>
    }
}
