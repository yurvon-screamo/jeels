use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use keikaku::domain::LearnSettings;

use crate::components::checkbox::Checkbox;
use crate::components::input::Input;
use crate::components::switch::{Switch, SwitchThumb};

fn create_learn_settings(
    limit_enabled: bool,
    limit: String,
    show_furigana: bool,
    low_stability_mode: bool,
    force_new_cards: bool,
) -> LearnSettings {
    let limit_value = if limit_enabled {
        limit.parse::<usize>().ok()
    } else {
        None
    };

    LearnSettings::new(
        limit_value,
        show_furigana,
        low_stability_mode,
        force_new_cards,
    )
}

#[component]
pub fn LearnSettingsForm(
    settings: LearnSettings,
    on_change: EventHandler<LearnSettings>,
) -> Element {
    let mut limit_enabled = use_signal(|| settings.limit().is_some());
    let limit = use_signal(|| settings.limit().unwrap_or(8).to_string());
    let mut show_furigana = use_signal(|| settings.show_furigana());
    let mut low_stability_mode = use_signal(|| settings.low_stability_mode());
    let mut force_new_cards = use_signal(|| settings.force_new_cards());

    let update_settings = {
        let limit_enabled = limit_enabled;
        let limit = limit;
        let show_furigana = show_furigana;
        let low_stability_mode = low_stability_mode;
        let force_new_cards = force_new_cards;
        let on_change = on_change;
        move || {
            let new_settings = create_learn_settings(
                limit_enabled(),
                limit(),
                show_furigana(),
                low_stability_mode(),
                force_new_cards(),
            );
            on_change.call(new_settings);
        }
    };

    rsx! {
        div { class: "space-y-4",
            div { class: "space-y-2",
                label { class: "flex items-center gap-3 cursor-pointer",
                    Checkbox {
                        checked: if limit_enabled() { CheckboxState::Checked } else { CheckboxState::Unchecked },
                        on_checked_change: move |state: CheckboxState| {
                            let v: bool = state.into();
                            limit_enabled.set(v);
                            update_settings();
                        },
                    }
                    span { class: "text-sm",
                        "Ограничить количество карточек"
                    }
                }
                if limit_enabled() {
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Лимит карточек" }
                        Input {
                            placeholder: "8",
                            value: limit(),
                            oninput: {
                                let mut limit = limit;
                                let update_settings = update_settings;
                                move |e: FormEvent| {
                                    limit.set(e.value());
                                    update_settings();
                                }
                            },
                        }
                    }
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm", "Показывать фуригану" }
                Switch {
                    aria_label: "Показывать фуригану",
                    checked: show_furigana(),
                    on_checked_change: move |v| {
                        show_furigana.set(v);
                        update_settings();
                    },
                    SwitchThumb {}
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm", "Режим низкой стабильности" }
                Switch {
                    aria_label: "Режим низкой стабильности",
                    checked: low_stability_mode(),
                    on_checked_change: move |v| {
                        low_stability_mode.set(v);
                        update_settings();
                    },
                    SwitchThumb {}
                }
            }

            div { class: "flex items-center justify-between gap-4",
                span { class: "text-sm", "Принудительно новые карточки" }
                Switch {
                    aria_label: "Принудительно новые карточки",
                    checked: force_new_cards(),
                    on_checked_change: move |v| {
                        force_new_cards.set(v);
                        update_settings();
                    },
                    SwitchThumb {}
                }
            }
        }
    }
}
