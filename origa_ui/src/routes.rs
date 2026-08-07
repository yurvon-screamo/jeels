use crate::loaders::{
    data_loader::{load_grammar, load_kanji, load_radicals, load_vocabulary},
    dictionary::load_dictionary,
    furigana_dict_loader::load_furigana_dict,
    jlpt_content_loader::load_jlpt_content,
    loading_message::{LoadingFlags, format_loading_message, loading_message_state},
    phrase_loader::load_phrases,
    pitch_audio_loader::load_pitch_audio,
};
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
use origa::use_cases::{
    MigrateKanjiCompanionsUseCase, MigrateVocabularyPartOfSpeechUseCase, SeedReadyPhrasesUseCase,
};

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
        // plus UniDic deflate-decompression (~61 MB → ~200 MB) and JSON parsing
        // — pushing peak memory past the limit and killing the process with an
        // OOM jetsam kill (not a Rust panic the hook can catch).
        //
        // Stages are ordered heaviest-first so the most memory-intensive work
        // happens while the JS heap is relatively empty. After each loader
        // returns, its raw bytes are consumed into static OnceLock structures
        // and the intermediate Vec/String buffers are dropped.
        //
        // Sizes (CDN, compressed):
        //   Stage 1 — dictionary:  ~61 MB (→ ~200 MB decompressed)
        //   Stage 2 — vocab+phrases+furigana+pitch: ~91 MB text
        //   Stage 3 — kanji+grammar+radicals: ~4 MB text

        // Stage 1: UniDic dictionary — solo, dominates memory usage.
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

        let migrate_vocab_pos = MigrateVocabularyPartOfSpeechUseCase::new(&repository);
        if let Err(e) = migrate_vocab_pos.execute().await {
            tracing::warn!("Failed to migrate vocabulary part of speech: {e}");
        }

        // Phase E: auto per-card pre-cache (background, only when online)
        if connectivity.is_online.get_untracked() {
            if let Some(user) = auth_store.user.get_untracked() {
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

    Effect::new({
        let auth_store = auth_store.clone();
        move |_| {
            if !is_checking.get()
                && is_authenticated.get()
                && !is_all_data_loaded.get()
                && !auth_store.is_data_loading_started.get()
            {
                auth_store.is_data_loading_started.set(true);
                start_dictionary_loading(
                    auth_store.clone(),
                    use_context::<HybridUserRepository>().expect("repository context not provided"),
                    use_context::<ConnectivityStore>().expect("ConnectivityStore not provided"),
                    use_context::<OfflineBundleStore>().expect("OfflineBundleStore not provided"),
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
        } else if is_authenticated.get() && !is_all_data_loaded.get() {
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
            view! {
                <LoadingOverlay message=loading_msg />
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
