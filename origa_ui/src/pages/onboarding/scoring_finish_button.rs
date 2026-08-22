use crate::i18n::*;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;

/// The Finish button of the onboarding scoring step.
///
/// Finishing is an async checkpoint (`CompleteOnboardingScoringUseCase` +
/// navigation), so while it runs the button must be disabled and flag its
/// loading state on `data-loading` — the same contract as the Summary step's
/// import button (`onboarding-import`).
#[component]
pub fn ScoringFinishButton(
    is_finishing: RwSignal<bool>,
    on_finish: Callback<()>,
    #[prop(optional, into)] test_id: Signal<String>,
) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <Button
            variant=ButtonVariant::Olive
            on_click=Callback::new(move |_: leptos::ev::MouseEvent| {
                on_finish.run(());
            })
            disabled=Signal::derive(move || is_finishing.get())
            attr:data-loading=Signal::derive(move || is_finishing.get().to_string())
            test_id=test_id
        >
            {move || if is_finishing.get() { t!(i18n, onboarding.completing).into_any() } else { t!(i18n, onboarding.finish).into_any() }}
        </Button>
    }
}
