use futures::future::{AbortHandle, abortable};
use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::i18n::*;
use crate::loaders::precache_loader::{self, PreCacheProgress};
use crate::store::offline_bundle_store::{CardCacheState, OfflineBundleStore};
use crate::ui_components::{Divider, ProgressBar, Text, TextSize, TypographyVariant};

#[derive(Clone, Copy, PartialEq)]
enum BundleState {
    Idle,
    CheckingCache,
    Downloading,
    Downloaded,
    Error,
}

#[component]
pub fn OfflineBundleCard(#[prop(optional, into)] test_id: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let state = RwSignal::new(BundleState::CheckingCache);
    let progress = RwSignal::new(PreCacheProgress::default());
    let abort_handle = RwSignal::new(None::<AbortHandle>);
    let progress_percent = RwSignal::new(0u32);

    let card_cache_state: Option<RwSignal<CardCacheState>> =
        use_context::<OfflineBundleStore>().map(|s| s.card_cache_state);
    let card_cache_progress: Option<RwSignal<PreCacheProgress>> =
        use_context::<OfflineBundleStore>().map(|s| s.card_cache_progress);

    Effect::new(move |_| {
        tracing::debug!("OfflineBundleCard Effect: checking bundle download state on mount");
        spawn_local(async move {
            if precache_loader::is_bundle_downloaded().await {
                tracing::debug!("OfflineBundleCard: bundle already downloaded");
                state.set(BundleState::Downloaded);
            } else {
                tracing::debug!("OfflineBundleCard: bundle not downloaded");
                state.set(BundleState::Idle);
            }
        });
    });

    Effect::new(move |_| {
        let p = progress.get();
        if p.total > 0 {
            let pct = (p.completed as f64 / p.total as f64 * 100.0).min(100.0) as u32;
            progress_percent.set(pct);
        }
    });

    let test_id_val = move || {
        let val = test_id.get();
        if val.is_empty() { None } else { Some(val) }
    };

    let is_downloading = Signal::derive(move || state.get() == BundleState::Downloading);
    let is_downloaded = Signal::derive(move || state.get() == BundleState::Downloaded);
    let is_error = Signal::derive(move || state.get() == BundleState::Error);

    let has_card_cache = card_cache_state.is_some();
    let card_state_signal = card_cache_state;
    let card_progress_signal = card_cache_progress;

    let on_download_click = move |_ev: leptos::ev::MouseEvent| {
        let state = state;
        let progress = progress;
        let abort_handle = abort_handle;

        let (future, handle) = abortable(async move {
            let result = precache_loader::precache_base_bundle(move |p| progress.set(p)).await;
            // After base bundle download, extract phrase data bundles into
            // individual cache entries so phrase_data_loader gets cache hits.
            if let Ok(_) = &result {
                if let Err(e) = precache_loader::extract_phrase_bundles_to_cache().await {
                    tracing::warn!(error = ?e, "Phrase data bundle extraction failed");
                }
            }
            result
        });
        abort_handle.set(Some(handle));
        state.set(BundleState::Downloading);

        spawn_local(async move {
            match future.await {
                Ok(Ok(_result)) => {
                    state.set(BundleState::Downloaded);
                },
                Ok(Err(e)) => {
                    tracing::error!(error = ?e, "Bundle download failed");
                    state.set(BundleState::Error);
                },
                Err(_) => {
                    state.set(BundleState::Idle);
                },
            }
        });
    };

    let on_cancel_click = move |_ev: leptos::ev::MouseEvent| {
        if let Some(handle) = abort_handle.get() {
            handle.abort();
        }
    };

    view! {
        <div data-testid=test_id_val class="p-6 space-y-4">
            <div class="space-y-2">
                <Text size=TextSize::Large>
                    {t!(i18n, profile.offline_bundle)}
                </Text>
                <Text size=TextSize::Small variant=TypographyVariant::Muted>
                    {t!(i18n, profile.offline_bundle_desc)}
                </Text>
            </div>

            <Show when=move || is_downloading.get()>
                <div class="space-y-3">
                    <ProgressBar
                        value=progress_percent
                        max=100
                        label=Signal::derive(move || {
                            let p = progress.get();
                            crate::i18n::td_string!(i18n.get_locale(), profile.files_progress)
                                .replace("{completed}", &p.completed.to_string())
                                .replace("{total}", &p.total.to_string())
                        })
                        test_id="bundle-progress"
                    />
                    <p class="text-xs text-[var(--fg-muted)] font-mono truncate">
                        {move || progress.get().current_file}
                    </p>
                </div>
            </Show>

            <Show when=move || is_downloaded.get()>
                <div class="flex items-center gap-2 text-sm text-[var(--fg-muted)]">
                    <span class="font-mono">"\u{2713}"</span>
                    <span class="font-mono">{t!(i18n, profile.bundle_downloaded)}</span>
                </div>
            </Show>

            <Show when=move || {
                matches!(state.get(), BundleState::Idle | BundleState::Error) || is_downloading.get()
            }>
                <div class="flex gap-3">
                    <Show when=move || {
                        matches!(state.get(), BundleState::Idle | BundleState::Error)
                    }>
                        <button
                            class="btn btn-filled anima-press anima-focus-ring"
                            on:click=on_download_click
                            data-testid="download-bundle-btn"
                        >
                            <span class="btn-text">
                                {move || if is_error.get() {
                                    t!(i18n, profile.retry_download).into_any()
                                } else {
                                    t!(i18n, profile.download_bundle).into_any()
                                }}
                            </span>
                        </button>
                    </Show>

                    <Show when=move || is_downloading.get()>
                        <button
                            class="btn anima-press anima-focus-ring"
                            on:click=on_cancel_click
                            data-testid="cancel-bundle-btn"
                        >
                            <span class="btn-text">
                                {t!(i18n, profile.cancel_download)}
                            </span>
                        </button>
                    </Show>
                </div>
            </Show>

            // Card cache status section
            <Show when=move || has_card_cache>
                <Divider />
                <div class="space-y-1">
                    {move || {
                        let signal = card_state_signal?;
                        let progress_signal = card_progress_signal?;
                        let cache_state = signal.get();

                        match cache_state {
                            CardCacheState::Running => {
                                let p = progress_signal.get();
                                let detail = if p.total > 0 {
                                    crate::i18n::td_string!(i18n.get_locale(), profile.files_progress)
                                        .replace("{completed}", &p.completed.to_string())
                                        .replace("{total}", &p.total.to_string())
                                } else {
                                    String::new()
                                };
                                Some(view! {
                                    <div class="flex items-center gap-2 text-sm text-[var(--fg-muted)]">
                                        <span class="font-mono animate-pulse">"..."</span>
                                        <span class="font-mono">{t!(i18n, profile.card_cache_running)}</span>
                                        <span class="font-mono text-xs">{detail}</span>
                                    </div>
                                }.into_any())
                            },
                            CardCacheState::Complete => {
                                Some(view! {
                                    <div class="flex items-center gap-2 text-sm text-[var(--fg-muted)]">
                                        <span class="font-mono">"\u{2713}"</span>
                                        <span class="font-mono">{t!(i18n, profile.card_cache_complete)}</span>
                                    </div>
                                }.into_any())
                            },
                            CardCacheState::Idle => None,
                        }
                    }}
                </div>
            </Show>
        </div>
    }
}
