//! Shared mount helpers and domain fixtures for WASM component tests.
//!
//! Extracted from `component_wasm_tests.rs` / `component_i18n_wasm_tests.rs`
//! (commit 48dd9dc) so every `*_wasm_tests.rs` file reuses one mount API.
//!
//! Mount helpers:
//! - [`mount_to_wrapper`] — leak mount (the 48dd9dc pattern). For components
//!   WITHOUT global listeners only.
//! - [`mount_disposable`] — RAII mount: the view unmounts when the returned
//!   handle drops at the end of the test. Required for components that
//!   install global listeners (e.g. `<Router>` → history/popstate).
//! - [`mount_with_i18n`] — leak mount + i18n context (for `use_i18n()`).
//! - [`mount_with_router`] — disposable mount inside a real `<Router>` +
//!   i18n context. `RouterContext` is `pub(crate)` in leptos_router 0.8, so
//!   `A` / `use_navigate` need a real router; disposal removes the stale
//!   router's listeners so later tests are unaffected.
//! - [`mount_with_stores`] — leak mount + i18n + `AuthStore` +
//!   `ConnectivityStore`. Stores are created inside the mount closure
//!   (owner-scoped) and passed to the caller's closure; their `RwSignal`
//!   fields are `Copy`, so a test can capture a signal (e.g. into an
//!   `Option` local) to drive state after `tick()`.
//!
//! Router children caveat: `TypedChildren` requires `FnOnce() -> C + Send`,
//! but erased `AnyView` (an `Rc`) is not `Send`. Wrapping the caller's
//! closure into the `<Router>` view via the `view!` macro does not help —
//! the whole children closure must be `Send`. [`mount_with_router`] instead
//! mounts a plain (non-generic) view: the test renders the router-dependent
//! component through a `ChildrenFn` prop of an intermediate component whose
//! view type is concrete at that point.

#![cfg(all(target_arch = "wasm32", test))]

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::i18n::Locale;
use crate::store::{AuthStore, ConnectivityStore};

// ─── Wrappers ──────────────────────────────────────────────────────────

/// Creates a `<div>` appended to `<body>` for test isolation.
pub(crate) fn create_wrapper() -> web_sys::Element {
    console_error_panic_hook::set_once();
    let document = web_sys::window().unwrap().document().unwrap();
    let wrapper = document.create_element("div").unwrap();
    let _ = document.body().unwrap().append_child(&wrapper);
    wrapper
}

// ─── Mounts ────────────────────────────────────────────────────────────

/// Mount a view closure into `wrapper`'s reactive scope. The `UnmountHandle`
/// is leaked to keep the component alive for the test (48dd9dc pattern).
pub(crate) fn mount_to_wrapper<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce() -> AnyView + 'static,
{
    let dispose = leptos::mount::mount_to(wrapper.clone().unchecked_into(), f);
    std::mem::forget(dispose);
}

/// Mount whose view unmounts automatically when the returned handle drops at
/// the end of the test (`UnmountHandle` implements `Drop`). Unlike
/// [`mount_to_wrapper`], the reactive owner is disposed too, removing global
/// listeners — use for `<Router>`-wrapping mounts.
pub(crate) fn mount_disposable<F, V>(
    wrapper: &web_sys::Element,
    f: F,
) -> leptos::mount::UnmountHandle<V::State>
where
    F: FnOnce() -> V + 'static,
    V: IntoView,
{
    leptos::mount::mount_to(wrapper.clone().unchecked_into(), f)
}

/// Mount a component that needs i18n context. The context is provided inside
/// the reactive scope before rendering the component.
pub(crate) fn mount_with_i18n<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce() -> AnyView + 'static,
{
    mount_to_wrapper(wrapper, move || {
        leptos_i18n::provide_i18n_context::<Locale>();
        f()
    });
}

// ─── Async wait helpers ────────────────────────────────────────────────

/// Polls `f` until it returns `true` or `attempts` run out (one attempt per
/// `interval_ms`). Replaces fixed `TimeoutFuture` sleeps in animation tests
/// — polls succeed as soon as the state settles instead of waiting out a
/// worst-case delay, removing the CI-runner flakiness risk.
pub(crate) async fn wait_until<F>(f: F, attempts: usize, interval_ms: u32) -> bool
where
    F: Fn() -> bool,
{
    for _ in 0..attempts {
        if f() {
            return true;
        }
        gloo_timers::future::TimeoutFuture::new(interval_ms).await;
    }
    f()
}

/// Restores the document pathname after a navigation test: pushes the
/// original URL back via the History API so later Router mounts observe the
/// same deterministic starting route (see `mount_with_router` docs).
pub(crate) fn restore_pathname(original: &str) {
    let window = web_sys::window().unwrap();
    let history = window.history().unwrap();
    let state = wasm_bindgen::JsValue::NULL;
    let _ = history.push_state_with_url(&state, "", Some(original));
}

// ─── Signal capture ────────────────────────────────────────────────────

/// Shared slot type for capturing a `Copy` value (e.g. a signal) out of a
/// mount closure.
pub(crate) type SharedCell<T> = std::rc::Rc<std::cell::Cell<Option<T>>>;

/// Creates a sender/receiver pair for [`SharedCell`]. The mount closure sets
/// the value via the first handle; the test reads it after mount:
///
/// ```ignore
/// let (set, get) = shared_cell::<RwSignal<u32>>();
/// mount_to_wrapper(&wrapper, move || {
///     let value = RwSignal::new(10u32);
///     set.set(Some(value));
///     view! { <ProgressBar value=value /> }.into_any()
/// });
/// let value = get.get().expect("captured");
/// ```
///
/// This is required because signals must be created inside the mount's
/// reactive scope (an `Owner`), while the test body runs outside it.
pub(crate) fn shared_cell<T>() -> (SharedCell<T>, SharedCell<T>) {
    let cell: SharedCell<T> = std::rc::Rc::new(std::cell::Cell::new(None));
    (cell.clone(), cell)
}

/// Leak mount with i18n, `AuthStore` and `ConnectivityStore` provided.
///
/// Stores are constructed inside the reactive scope (owner-scoped context)
/// and handed to `f`, which may set initial signal values before rendering.
/// To drive state after mount, capture a store signal — they are `Copy`.
pub(crate) fn mount_with_stores<F>(wrapper: &web_sys::Element, f: F)
where
    F: FnOnce(&AuthStore, &ConnectivityStore) -> AnyView + 'static,
{
    mount_to_wrapper(wrapper, move || {
        leptos_i18n::provide_i18n_context::<Locale>();
        let auth = AuthStore::new();
        let connectivity = ConnectivityStore::new();
        provide_context(auth.clone());
        provide_context(connectivity.clone());
        f(&auth, &connectivity)
    });
}

// ─── Router mounts ─────────────────────────────────────────────────────

/// Intermediate view that provides i18n context and renders a real
/// `<Router>` around the component-under-test. Being a plain `#[component]`
/// with a `ChildrenFn` prop, the router's children type is inferred here
/// where the view is concrete — sidestepping the `Send` bound issue of
/// erased closures (see module docs).
#[component]
fn TestRouterHost(children: ChildrenFn) -> impl IntoView {
    leptos_i18n::provide_i18n_context::<Locale>();
    view! {
        <leptos_router::components::Router>
            {children()}
        </leptos_router::components::Router>
    }
}

/// Disposable mount inside a real `<Router>` with i18n context.
///
/// `RouterContext` is `pub(crate)` in leptos_router 0.8 — the only way to
/// satisfy `A` / `use_navigate` is a real router. The component closure
/// runs inside the router's children slot via [`TestRouterHost`], so hooks
/// like `use_navigate()` find the context during their setup phase. Keep the
/// returned handle alive until the assertions are done (`let _mount = …`);
/// on drop the router's history/popstate listeners are removed, so routers
/// of earlier tests cannot observe later navigation. Each test runs on the
/// same document pathname (the wasm-bindgen test runner serves one page),
/// giving every router the same deterministic starting route.
pub(crate) fn mount_with_router<F>(
    wrapper: &web_sys::Element,
    f: F,
) -> leptos::mount::UnmountHandle<leptos::tachys::view::any_view::AnyViewState>
where
    F: Fn() -> AnyView + 'static,
{
    mount_disposable(wrapper, move || {
        let children = StoredValue::new_local(f);
        view! {
            <TestRouterHost>
                {move || children.with_value(|g| g())}
            </TestRouterHost>
        }
        .into_any()
    })
}

/// Disposable mount combining a real `<Router>`, i18n context and the app
/// stores (`AuthStore`, `ConnectivityStore`) — for components that mix
/// routing hooks with store context (e.g. `Sidebar`, `BottomTabBar`).
/// The user signal of the fresh `AuthStore` is set to `Some(user)` before
/// the component renders, so authenticated state is the default here.
/// The `HybridUserRepository` is also provided (mirroring `app.rs`) so
/// components reading the repository context work out of the box.
pub(crate) fn mount_with_router_and_stores<F>(
    wrapper: &web_sys::Element,
    user: Option<origa::domain::User>,
    f: F,
) -> leptos::mount::UnmountHandle<leptos::tachys::view::any_view::AnyViewState>
where
    F: Fn() -> AnyView + 'static,
{
    mount_with_router(wrapper, move || {
        let auth = AuthStore::new();
        auth.user.set(user.clone());
        provide_context(auth.repository().clone());
        provide_context(auth);
        provide_context(ConnectivityStore::new());
        f()
    })
}

// ─── Domain fixtures ───────────────────────────────────────────────────

use origa::domain::{Card, Question, StudyCard, VocabularyCard};

/// A study card wrapping a vocabulary card, without dictionary access
/// (POS bypassed via [`VocabularyCard::new_with_pos`]).
pub(crate) fn vocabulary_study_card(word: &str) -> StudyCard {
    let vocab = VocabularyCard::new_with_pos(Question::new(word.to_string()).unwrap(), None, None);
    StudyCard::new(Card::Vocabulary(vocab))
}

// NOTE: no kanji study-card fixture — `KanjiCard::new` requires the kradfile
// radicals dictionary (CDN-loaded), so kanji fixtures are unavailable in
// component tests. Layout-level tests use [`vocabulary_study_card`]
// instead (grouping and list rendering only consume `card_id` + level
// index, not the card type).

/// A domain user with the given username (email local part).
pub(crate) fn test_user(name: &str) -> origa::domain::User {
    origa::domain::User::new(
        format!("{name}@example.com"),
        origa::domain::NativeLanguage::Russian,
        None,
    )
}
