//! Consent screen for the mandatory first-run resource download.
//!
//! App Review Guideline 4.2.3(ii): when the app must download additional
//! resources to be usable, the download size must be disclosed and the user
//! must be prompted before it starts. This screen gates the dictionary /
//! study-material fetch (~40 MB) behind an explicit "Download" click; the
//! choice is persisted so subsequent launches load without prompting.

use crate::i18n::{t, use_i18n};
use crate::ui_components::{Button, ButtonVariant, Text, TextSize, TypographyVariant};
use leptos::prelude::*;

/// LocalStorage flag marking that the user approved resource downloads.
pub const RESOURCE_DOWNLOAD_CONSENTED_KEY: &str = "origa_resource_download_consented";

/// Returns `true` if the user has already approved resource downloads.
///
/// Missing flag (first launch, or pre-consent users after the update) means
/// the prompt is shown once — with a warm cache it passes instantly.
pub fn is_resource_download_consented() -> bool {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::get::<bool>(RESOURCE_DOWNLOAD_CONSENTED_KEY).unwrap_or(false)
}

/// Persists the user's approval. Treated as a standing permission: later
/// launches start loading immediately, and offline failures retry on the
/// next launch through the same auto-start path.
pub fn persist_resource_download_consent() {
    use gloo_storage::{LocalStorage, Storage};
    LocalStorage::set(RESOURCE_DOWNLOAD_CONSENTED_KEY, true)
        .unwrap_or_else(|e| tracing::warn!("Failed to persist download consent: {e:?}"));
}

#[component]
pub fn ResourceDownloadConsent(#[prop(into)] on_start: Callback<()>) -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="loading-overlay anima-page-fade">
            <div class="resource-download-consent">
                <Text
                    size=TextSize::Large
                    variant=TypographyVariant::Primary
                    test_id=Signal::derive(|| "resource-consent-title".to_string())
                >
                    {t!(i18n, ui.resource_download.title)}
                </Text>
                <div class="mt-4 max-w-md text-center">
                    <Text
                        size=TextSize::Default
                        variant=TypographyVariant::Muted
                        test_id=Signal::derive(|| "resource-consent-body".to_string())
                    >
                        {t!(i18n, ui.resource_download.body)}
                    </Text>
                </div>
                <div class="mt-8">
                    <Button
                        variant=ButtonVariant::Filled
                        test_id="resource-consent-download-button"
                        on_click=Callback::new(move |_| {
                            persist_resource_download_consent();
                            on_start.run(());
                        })
                    >
                        {t!(i18n, ui.resource_download.start)}
                    </Button>
                </div>
            </div>
        </div>
    }
}
