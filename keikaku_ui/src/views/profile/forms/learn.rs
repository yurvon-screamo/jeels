use dioxus::prelude::*;
use keikaku::domain::LearnSettings;

use crate::ui::{Checkbox, Switch, TextInput};

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
    let mut limit = use_signal(|| settings.limit().unwrap_or(30).to_string());
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
                Checkbox {
                    checked: limit_enabled(),
                    onchange: move |v| {
                        limit_enabled.set(v);
                        update_settings();
                    },
                    label: Some("Ограничить количество карточек".to_string()),
                }
                if limit_enabled() {
                    TextInput {
                        label: Some("Лимит карточек".to_string()),
                        placeholder: Some("30".to_string()),
                        value: limit,
                        oninput: Some(
                            EventHandler::new({
                                let mut limit = limit;
                                let update_settings = update_settings;
                                move |e: Event<FormData>| {
                                    limit.set(e.value());
                                    update_settings();
                                }
                            }),
                        ),
                        class: None,
                        r#type: None,
                    }
                }
            }

            Switch {
                checked: show_furigana(),
                onchange: move |v| {
                    show_furigana.set(v);
                    update_settings();
                },
                label: Some("Показывать фуригану".to_string()),
            }

            Switch {
                checked: low_stability_mode(),
                onchange: move |v| {
                    low_stability_mode.set(v);
                    update_settings();
                },
                label: Some("Режим низкой стабильности".to_string()),
            }

            Switch {
                checked: force_new_cards(),
                onchange: move |v| {
                    force_new_cards.set(v);
                    update_settings();
                },
                label: Some("Принудительно новые карточки".to_string()),
            }
        }
    }
}
