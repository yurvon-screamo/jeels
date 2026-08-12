use crate::i18n::{Locale, use_i18n};
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_router::components::A;

fn get_greeting(is_ru: bool) -> &'static str {
    let hours = js_sys::Date::new_0().get_hours() as u8;
    match hours {
        5..=11 => {
            if is_ru {
                "Доброе утро"
            } else {
                "Good morning"
            }
        },
        12..=17 => {
            if is_ru {
                "Добрый день"
            } else {
                "Good afternoon"
            }
        },
        18..=22 => {
            if is_ru {
                "Добрый вечер"
            } else {
                "Good evening"
            }
        },
        _ => {
            if is_ru {
                "Доброй ночи"
            } else {
                "Good night"
            }
        },
    }
}

fn get_subline(is_ru: bool) -> &'static str {
    if is_ru {
        "Готовы к занятию?"
    } else {
        "Ready to study?"
    }
}

#[component]
pub fn WelcomeCard(
    username: Signal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let is_ru = Signal::derive(move || i18n.get_locale() == Locale::ru);
    let greeting = Signal::derive(move || get_greeting(is_ru.get()));
    let subline = Signal::derive(move || get_subline(is_ru.get()));

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let lesson_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "lesson-buttons-lesson".to_string()
        } else {
            format!("{}-lesson", val)
        }
    });

    view! {
        <div class="py-4 sm:py-6" data-testid=test_id_val>
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div class="font-serif text-2xl sm:text-3xl text-[var(--fg-black)]">
                    {move || greeting.get()} ", "
                    <span class="text-[var(--accent-olive)]">{move || username.get()}</span>
                    ". "
                    <span class="text-[var(--fg-muted)]">{move || subline.get()}</span>
                </div>
                <div class="shrink-0">
                    <A href="/lesson">
                        <button
                            class="btn btn-olive flex items-center justify-center gap-2 w-full sm:w-auto sm:min-w-[280px] px-6 py-3 sm:px-10 sm:py-2.5"
                            data-testid=move || Some(lesson_test_id.get())
                        >
                            <Icon icon=icondata::LuBookOpen width="16" height="16" />
                            {move || if is_ru.get() { "УРОК".to_string() } else { "LESSON".to_string() }}
                        </button>
                    </A>
                </div>
            </div>
        </div>
    }
}
