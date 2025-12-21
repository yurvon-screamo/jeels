use dioxus::prelude::*;
use keikaku::domain::TranslationSettings;

use crate::components::input::Input;

fn create_translation_settings(temperature: String, seed: String) -> TranslationSettings {
    let temp = temperature.parse().unwrap_or(0.0);
    let seed_val = seed.parse().unwrap_or(0);
    TranslationSettings::new(temp, seed_val)
}

#[component]
pub fn TranslationSettingsForm(
    settings: TranslationSettings,
    on_change: EventHandler<TranslationSettings>,
) -> Element {
    let temperature = use_signal(|| settings.temperature().to_string());
    let seed = use_signal(|| settings.seed().to_string());

    let update_settings = {
        let temperature = temperature;
        let seed = seed;
        let on_change = on_change;
        move || {
            let new_settings = create_translation_settings(temperature(), seed());
            on_change.call(new_settings);
        }
    };

    rsx! {
        div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Temperature" }
                Input {
                    value: temperature(),
                    oninput: {
                        let mut temperature = temperature;
                        let update_settings = update_settings;
                        move |e: FormEvent| {
                            temperature.set(e.value());
                            update_settings();
                        }
                    },
                }
            }
            div { class: "space-y-2",
                label { class: "text-sm font-medium", "Seed" }
                Input {
                    value: seed(),
                    oninput: {
                        let mut seed = seed;
                        let update_settings = update_settings;
                        move |e: FormEvent| {
                            seed.set(e.value());
                            update_settings();
                        }
                    },
                }
            }
        }
    }
}
