//! Live-microphone speech recording via native device-ai.
//!
//! Renders a record button. On click it runs one-shot native recognition
//! (`device_ai::recognize_live`) — the platform captures audio from the mic
//! and returns recognized text when the speaker pauses. On platforms where
//! device-ai is unavailable (web, Linux, Windows) the button is hidden; those
//! environments use the file-upload path with the Whisper WASM fallback.

use crate::core::device_ai::{self, Feature};
use crate::i18n::use_i18n;
use crate::ui_components::{Button, ButtonVariant};
use leptos::prelude::*;
use leptos::task::spawn_local;
use tracing::warn;

use super::asr_provider;

#[component]
pub(super) fn AudioLiveRecorder(
    on_text_extracted: Callback<String>,
    on_error: Callback<String>,
) -> impl IntoView {
    let i18n = use_i18n();
    let recording = RwSignal::new(false);
    // Whether native live recognition is actually available. Resolved once via
    // the cached capabilities query: `is_tauri()` alone is true on Windows,
    // where the device-ai plugin is not compiled in — so the button would show
    // but always fail. We hide it until capabilities confirm availability.
    let native_available = RwSignal::new(false);

    Effect::new(move |_| {
        spawn_local(async move {
            native_available.set(device_ai::available(Feature::SpeechRecognition).await);
        });
    });

    let on_record = move || {
        if recording.get() || !native_available.get() {
            return;
        }
        recording.set(true);

        let on_te = on_text_extracted;
        let on_err = on_error;
        let i18n_local = i18n;
        let recording_local = recording;

        spawn_local(async move {
            match asr_provider::recognize_live_via_device_ai().await {
                Some(text) if !text.trim().is_empty() => {
                    recording_local.set(false);
                    on_te.run(text);
                },
                Some(_) => {
                    recording_local.set(false);
                    on_err.run(
                        i18n_local
                            .get_keys()
                            .words()
                            .audio()
                            .no_speech()
                            .inner()
                            .to_string(),
                    );
                },
                None => {
                    recording_local.set(false);
                    warn!("device-ai live recognition unavailable");
                    on_err.run(
                        i18n_local
                            .get_keys()
                            .words()
                            .audio()
                            .live_unavailable()
                            .inner()
                            .to_string(),
                    );
                },
            }
        });
    };

    view! {
        <div class="flex items-center gap-3">
            {move || {
                native_available.get().then(|| {
                    view! {
                        <Button
                            variant=Signal::derive(move || {
                                if recording.get() {
                                    ButtonVariant::Ghost
                                } else {
                                    ButtonVariant::Filled
                                }
                            })
                            disabled=Signal::derive(move || recording.get())
                            on_click=Callback::new(move |_| on_record())
                        >
                            {move || {
                                if recording.get() {
                                    i18n.get_keys().words().audio().recording().inner().to_string()
                                } else {
                                    i18n.get_keys().words().audio().record().inner().to_string()
                                }
                            }}
                        </Button>
                    }
                })
            }}
            {move || {
                recording.get().then(|| view! { <span class="spinner spinner-sm"></span> })
            }}
        </div>
    }
}
