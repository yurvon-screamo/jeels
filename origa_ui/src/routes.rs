use crate::loaders::{
    data_loader::{load_grammar, load_kanji, load_radicals, load_vocabulary},
    dictionary::load_dictionary,
    furigana_dict_loader::load_furigana_dict,
    jlpt_content_loader::load_jlpt_content,
    loading_message::{
        LOADING_RESOURCES_TOTAL, LoadingFlags, format_loading_message, loading_message_state,
    },
    phrase_loader::load_phrases,
    pitch_audio_loader::load_pitch_audio,
};
use crate::pages::shared::{ResourceDownloadConsent, is_resource_download_consented};
use crate::pages::{
    Grammar, GrammarDetail, Home, Kanji, KanjiDetail, Lesson, Login, Onboarding, Phrases, Profile,
    Sets, Words,
};
use crate::store::auth_store::AuthStore;
use crate::store::connectivity::ConnectivityStore;
use crate::store::offline_bundle_store::OfflineBundleStore;
use crate::ui_components::{BottomTabBar, LoadingOverlay, Sidebar};
use futures::Future;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::*;
use leptos_router::hooks::use_location;
use leptos_router::path;
use origa::domain::{OrigaError, User};
use origa::traits::UserRepository;
use origa::use_cases::{MigrateKanjiCompanionsUseCase, SeedReadyPhrasesUseCase};

use crate::repository::HybridUserRepository;

async fn load_with_retry<F, Fut>(loader: F, max_retries: usize) -> Result<(), OrigaError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(), OrigaError>>,
{
    let mut last_err = None;
    for attempt in 0..=max_retries {
        match loader().await {
            Ok(()) => return Ok(()),
            Err(e) => {
                if attempt < max_retries {
                    tracing::info!("Retrying after error: {e}");
                }
                last_err = Some(e);
            },
        }
    }
    Err(last_err.expect("at least one attempt was made"))
}

pub fn start_dictionary_loading(
    auth_store: AuthStore,
    repository: HybridUserRepository,
    connectivity: ConnectivityStore,
    offline_store: OfflineBundleStore,
) {
    spawn_local(async move {
        // Phase A: manifest check
        if let Err(e) = crate::repository::cache_manager::check_and_invalidate().await {
            tracing::warn!("Cache manifest check failed: {e}");
        }

        // Phase B: staged loading to minimize peak WASM linear memory.
        //
        // The iOS WKWebView process has a ~1.5 GB jetsam limit. Loading all
        // resources simultaneously via futures::join! caused ~27 concurrent
        // HTTP requests and ~144 MB of response bodies in the JS heap at once,
        // plus dictionary deflate-decompression and JSON parsing
        // — pushing peak memory past the limit and killing the process with an
        // OOM jetsam kill (not a Rust panic the hook can catch).
        //
        // Stages are ordered heaviest-first so the most memory-intensive work
        // happens while the JS heap is relatively empty. After each loader
        // returns, its raw bytes are consumed into static OnceLock structures
        // and the intermediate Vec/String buffers are dropped.
        //
        // Sizes (CDN, compressed):
        //   Stage 1 — dictionary: largest single inflate
        //   Stage 2 — vocab+phrases+furigana+pitch: ~91 MB text
        //   Stage 3 — kanji+grammar+radicals: ~4 MB text

        // Stage 1: tokenizer dictionary — solo, dominates memory usage.
        if let Err(e) = load_dictionary().await {
            tracing::error!("Failed to load dictionary: {e}");
        }
        auth_store.is_dictionary_loaded.set(true);

        // Stage 2: medium resources in parallel.
        let (vocab_r, phrases_r, furigana_r, pitch_r) = futures::join!(
            load_vocabulary(),
            load_phrases(),
            load_furigana_dict(),
            load_pitch_audio(),
        );

        if let Err(e) = vocab_r {
            tracing::error!("Failed to load vocabulary: {e}");
        }
        auth_store.is_vocabulary_loaded.set(true);

        if let Err(e) = phrases_r {
            tracing::error!("Failed to load phrases: {e}");
        }
        auth_store.is_phrases_loaded.set(true);

        if let Err(e) = furigana_r {
            tracing::warn!("Failed to load furigana: {e}");
        }
        auth_store.is_furigana_loaded.set(true);

        if let Err(e) = pitch_r {
            tracing::warn!("Failed to load pitch audio: {e}");
        }
        auth_store.is_pitch_audio_loaded.set(true);

        // Stage 3: light resources in parallel.
        let (kanji_r, grammar_r, radicals_r) = futures::join!(
            load_with_retry(load_kanji, 1),
            load_grammar(),
            load_with_retry(load_radicals, 1),
        );

        if let Err(e) = kanji_r {
            tracing::error!("Failed to load kanji: {e}");
        }
        auth_store.is_kanji_loaded.set(true);

        if let Err(e) = grammar_r {
            tracing::error!("Failed to load grammar: {e}");
        }
        auth_store.is_grammar_loaded.set(true);

        if let Err(e) = radicals_r {
            tracing::error!("Failed to load radicals: {e}");
        }
        auth_store.is_radicals_loaded.set(true);

        // Phase C: jlpt_content (depends on kanji + grammar)
        if let Err(e) = load_with_retry(load_jlpt_content, 1).await {
            tracing::error!("Failed to load jlpt_content: {e}");
        }

        // Phase D: post-load migrations (BEFORE signaling completion)
        let seed_use_case = SeedReadyPhrasesUseCase::new(&repository);
        if let Err(e) = seed_use_case.execute().await {
            tracing::warn!("Failed to seed ready phrases: {e}");
        }

        let migrate_kanji = MigrateKanjiCompanionsUseCase::new(&repository);
        if let Err(e) = migrate_kanji.execute().await {
            tracing::warn!("Failed to migrate kanji companions: {e}");
        }

        // Phase E: auto per-card pre-cache (background, only when online)
        if connectivity.is_online.get_untracked() {
            if let Some(user) = auth_store.user.get_untracked() {
                // Validate cache if marked as complete
                let cache_state = offline_store.card_cache_state.get_untracked();
                if cache_state == crate::store::offline_bundle_store::CardCacheState::Complete {
                    const CARD_CACHE_MARKER_KEY: &str = "/__origa_card_cache_complete__";
                    let is_cache_valid =
                        crate::repository::cdn_provider::is_cached(CARD_CACHE_MARKER_KEY).await;
                    if !is_cache_valid {
                        tracing::warn!("Card cache marker not found, resetting to Idle");
                        offline_store.set_card_cache_state(
                            crate::store::offline_bundle_store::CardCacheState::Idle,
                        );
                    }
                }

                let cards: Vec<origa::domain::StudyCard> = user
                    .knowledge_set()
                    .study_cards()
                    .values()
                    .cloned()
                    .collect();

                if !cards.is_empty() {
                    crate::loaders::card_precache_loader::start_card_precache(cards, offline_store);
                }
            }
        }

        // Signal completion only after all migrations finish
        auth_store.is_jlpt_content_loaded.set(true);
    });
}

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");

    let is_authenticated = auth_store.is_authenticated();
    let is_all_data_loaded = auth_store.is_all_data_loaded();
    let is_checking = auth_store.is_checking_session;

    // Guideline 4.2.3(ii) consent gate: the mandatory ~230 MB resource fetch
    // only starts after the user approves it on the consent screen. Both
    // trigger points are gated — this effect (auto-start) and the view branch
    // below (consent screen vs loading overlay).
    let download_consented = RwSignal::new(is_resource_download_consented());

    let repository =
        use_context::<HybridUserRepository>().expect("repository context not provided");
    let connectivity = use_context::<ConnectivityStore>().expect("ConnectivityStore not provided");
    let offline_bundle =
        use_context::<OfflineBundleStore>().expect("OfflineBundleStore not provided");

    Effect::new({
        let auth_store = auth_store.clone();
        let repository = repository.clone();
        let connectivity = connectivity.clone();
        let offline_bundle = offline_bundle.clone();
        move |_| {
            if !download_consented.get() {
                return;
            }
            if !is_checking.get()
                && is_authenticated.get()
                && !is_all_data_loaded.get()
                && !auth_store.is_data_loading_started.get()
            {
                auth_store.is_data_loading_started.set(true);
                start_dictionary_loading(
                    auth_store.clone(),
                    repository.clone(),
                    connectivity.clone(),
                    offline_bundle.clone(),
                );
            }
        }
    });

    move || {
        if auth_store.is_loading().get() {
            let loading_msg: Signal<String> = Signal::derive(move || {
                crate::i18n::use_i18n()
                    .get_keys()
                    .common()
                    .loading()
                    .inner()
                    .to_string()
            });
            view! {
                <LoadingOverlay message=loading_msg />
            }
            .into_any()
        } else if is_authenticated.get() && !is_all_data_loaded.get() && !download_consented.get() {
            let auth_store_for_start = auth_store.clone();
            let repository_for_start = repository.clone();
            let connectivity_for_start = connectivity.clone();
            let offline_bundle_for_start = offline_bundle.clone();
            view! {
                <ResourceDownloadConsent
                    on_start=move |_| {
                        // The consent screen persists the approval (its click
                        // handler) and hands off to the standard loading
                        // overlay. `is_data_loading_started` is set first so
                        // the auto-start effect (reacting to the consent flip)
                        // does not launch a second fetch.
                        auth_store_for_start.is_data_loading_started.set(true);
                        download_consented.set(true);
                        start_dictionary_loading(
                            auth_store_for_start.clone(),
                            repository_for_start.clone(),
                            connectivity_for_start.clone(),
                            offline_bundle_for_start.clone(),
                        );
                    }
                />
            }
            .into_any()
        } else if is_authenticated.get() && !is_all_data_loaded.get() && download_consented.get() {
            let store = auth_store.clone();
            let loading_msg: Signal<String> = Signal::derive(move || {
                let i18n = crate::i18n::use_i18n();
                let flags = LoadingFlags {
                    vocabulary: store.is_vocabulary_loaded.get(),
                    kanji: store.is_kanji_loaded.get(),
                    grammar: store.is_grammar_loaded.get(),
                    radicals: store.is_radicals_loaded.get(),
                    phrases: store.is_phrases_loaded.get(),
                    pitch_audio: store.is_pitch_audio_loaded.get(),
                    dictionary: store.is_dictionary_loaded.get(),
                    furigana: store.is_furigana_loaded.get(),
                    jlpt_content: store.is_jlpt_content_loaded.get(),
                };
                let state = loading_message_state(&flags);
                let fetching_template = i18n
                    .get_keys()
                    .ui()
                    .loading_fetching_progress()
                    .inner()
                    .to_string();
                let finalizing_template = i18n
                    .get_keys()
                    .ui()
                    .loading_finalizing_progress()
                    .inner()
                    .to_string();
                format_loading_message(state, &fetching_template, &finalizing_template)
            });
            // Progress bar mirrors the "X of Y" message (Guideline 4.2.3(ii)).
            let progress: Signal<Option<(u32, u32)>> = Signal::derive(move || {
                let completed = [
                    store.is_vocabulary_loaded.get(),
                    store.is_kanji_loaded.get(),
                    store.is_grammar_loaded.get(),
                    store.is_radicals_loaded.get(),
                    store.is_phrases_loaded.get(),
                    store.is_pitch_audio_loaded.get(),
                    store.is_dictionary_loaded.get(),
                    store.is_furigana_loaded.get(),
                    store.is_jlpt_content_loaded.get(),
                ]
                .into_iter()
                .filter(|loaded| *loaded)
                .count() as u32;
                Some((completed, LOADING_RESOURCES_TOTAL as u32))
            });
            view! {
                <LoadingOverlay
                    message=loading_msg
                    progress
                    test_id="app-loading-overlay"
                />
            }
            .into_any()
        } else if is_authenticated.get() {
            children().into_any()
        } else {
            view! { <Login/> }.into_any()
        }
    }
}

#[component]
pub fn AppRoutes() -> impl IntoView {
    let auth_store = use_context::<AuthStore>().expect("AuthStore not provided");
    let repository = auth_store.repository().clone();
    let current_user: RwSignal<Option<User>> = RwSignal::new(None);
    let location = use_location();

    Effect::new({
        let repository = repository.clone();
        let auth_store_for_effect = auth_store.clone();
        move |_| {
            let _ = auth_store_for_effect.is_authenticated().get();
            let repository = repository.clone();
            spawn_local(async move {
                if let Ok(Some(user)) = repository.get_current_user().await {
                    current_user.set(Some(user));
                }
            });
        }
    });

    let sidebar_visible = Signal::derive(move || {
        let authenticated = auth_store.is_authenticated().get();
        let path = location.pathname.get();
        let hidden_path = path == "/lesson" || path == "/onboarding";
        let has_user = current_user.with(|u| u.is_some());
        authenticated && !hidden_path && has_user
    });

    // Non-sidebar <main> is a flex column so pages can fill the shell via flex-1
    // (e.g., lesson card centering). See ADR-027; height + safe-area stay on the shell.
    let main_class = move || {
        if sidebar_visible.get() {
            "paper-texture main-with-sidebar pt-safe-t-half pb-20 lg:pb-0".to_string()
        } else {
            "paper-texture pt-safe-t-half min-h-[100dvh] flex flex-col".to_string()
        }
    };

    view! {
        <Show when=move || sidebar_visible.get()>
            <Sidebar current_user test_id="sidebar" />
        </Show>
        <main class=main_class>
            <Routes fallback=|| view! { <Login/> }>
                <Route path=path!("/") view=|| view! { <ProtectedRoute><Home/></ProtectedRoute> } />
                <Route path=path!("login") view=Login />
                <Route path=path!("onboarding") view=|| view! { <ProtectedRoute><Onboarding/></ProtectedRoute> } />
                <Route path=path!("home") view=|| view! { <ProtectedRoute><Home/></ProtectedRoute> } />
                <Route path=path!("profile") view=|| view! { <ProtectedRoute><Profile/></ProtectedRoute> } />
                <Route path=path!("words") view=|| view! { <ProtectedRoute><Words/></ProtectedRoute> } />
                <Route path=path!("grammar/:id") view=|| view! { <ProtectedRoute><GrammarDetail/></ProtectedRoute> } />
                <Route path=path!("grammar") view=|| view! { <ProtectedRoute><Grammar/></ProtectedRoute> } />
                <Route path=path!("phrases") view=|| view! { <ProtectedRoute><Phrases/></ProtectedRoute> } />
                <Route path=path!("kanji/:id") view=|| view! { <ProtectedRoute><KanjiDetail/></ProtectedRoute> } />
                <Route path=path!("kanji") view=|| view! { <ProtectedRoute><Kanji/></ProtectedRoute> } />
                <Route path=path!("lesson") view=|| view! { <ProtectedRoute><Lesson/></ProtectedRoute> } />
                <Route path=path!("sets") view=|| view! { <ProtectedRoute><Sets/></ProtectedRoute> } />
            </Routes>
            <BottomTabBar test_id="bottom-tab" />
        </main>
    }
}
