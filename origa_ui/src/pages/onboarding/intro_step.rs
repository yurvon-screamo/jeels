use crate::i18n::*;
use crate::ui_components::{Input, NativeLanguageToggle, Text, TextSize, TypographyVariant};
use leptos::prelude::*;
use origa::domain::NativeLanguage;

/// The greeting step: language toggle plus the very first personalization
/// question — the display name. The name is optional: leaving it empty (e.g.
/// for Apple relay emails, which seed no usable default) keeps the current
/// profile name, and it can always be changed later on the profile page.
#[component]
pub fn IntroStep(
    selected_language: RwSignal<NativeLanguage>,
    username: RwSignal<String>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };
    let i18n_for_placeholder = i18n;
    let username_placeholder = Signal::derive(move || {
        let locale = i18n_for_placeholder.get_locale();
        td_string!(locale, common.username_placeholder).to_string()
    });

    view! {
        <div data-testid=test_id_val class="intro-step max-w-xl mx-auto text-center">
            // Language toggle sits on its own row above the title so long
            // translated titles never overlap the toggle on narrow screens.
            <div class="flex justify-end mb-4" data-testid="intro-step-language-bar">
                <NativeLanguageToggle selected_language=selected_language test_id=Signal::derive(|| "intro-lang-toggle".to_string()) />
            </div>
            <div class="intro-welcome flex flex-col items-center gap-4 mb-8">
                <Text size=TextSize::Large variant=TypographyVariant::Primary test_id="intro-step-title" class="w-full text-center">
                    {t!(i18n, onboarding.intro.title)}
                </Text>
                <Text size=TextSize::Default variant=TypographyVariant::Muted test_id="intro-step-subtitle">
                    {t!(i18n, onboarding.intro.subtitle)}
                </Text>
            </div>
            <div class="intro-name flex flex-col items-center gap-2 mb-8 w-full max-w-md mx-auto" data-testid="intro-step-name">
                <label class="label-muted block w-full text-left" for="intro-step-name-input" data-testid="intro-step-name-label">
                    {t!(i18n, profile.username)}
                </label>
                <Input
                    value=username
                    placeholder=username_placeholder
                    id=Signal::derive(|| "intro-step-name-input".to_string())
                    name=Signal::derive(|| "display_name".to_string())
                    maxlength=Signal::derive(|| Some(origa::use_cases::USERNAME_MAX_CHARS))
                    class=Signal::derive(|| "w-full".to_string())
                    test_id=Signal::derive(|| "intro-step-name-input".to_string())
                />
            </div>
        </div>
    }
}
