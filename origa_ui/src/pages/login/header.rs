use crate::i18n::*;
use crate::ui_components::{Logo, LogoSize, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

#[component]
pub fn LoginHeader(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let subtitle_test_id = Signal::derive(move || {
        let val = test_id.get();
        if val.is_empty() {
            "login-subtitle".to_string()
        } else {
            format!("{}-subtitle", val)
        }
    });

    view! {
        <div class="text-center" data-testid=test_id_val>
            <Logo size=LogoSize::Lg class=Signal::derive(|| "mx-auto mb-6".to_string()) />
            <Text size=TextSize::Small variant=TypographyVariant::Muted uppercase=true tracking_widest=true test_id=subtitle_test_id>
                {t!(i18n, login.subtitle)}
            </Text>
        </div>
    }
}
