# AGENTS.md — `origa_ui` crate

Leptos 0.8 / WASM frontend for Origa (Japanese learning app). CSR mode only. Rust edition 2024.
For full-app dev: `cd tauri && cargo tauri dev`. Frontend-only: `cd origa_ui && trunk serve`.

## Source Structure

```text
origa_ui/src/
├── lib.rs, main.rs, app.rs, i18n.rs, routes.rs
├── core/              # config, updater, version (build.rs env vars)
├── ui_components/     # 54 components (button, card, modal, sidebar, furigana,
│                      #   kanji_animation, audio_player, toast, skeleton, search...)
├── pages/             # home, login, onboarding, lesson, profile, words,
│                      #   kanji, grammar, phrases, sets, shared
├── repository/        # HybridUserRepository (TrailBase + IndexedDB),
│                      #   CDN provider, dictionary cache, session mgmt
├── store/             # AuthStore (auth state, dict loading, repo ref),
│                      #   connectivity (online/offline)
├── loaders/           # async data init (dictionaries, models, kanji,
│                      #   vocabulary, grammar, phrases, pitch audio)
├── hooks/             # custom Leptos hooks (phrase_checker)
└── utils/             # fetch, file (OPFS), time, drag_drop, yield_
```

## Key Dependencies

| Purpose            | Crate                    |
|--------------------|--------------------------|
| UI framework       | Leptos 0.8 (CSR)         |
| Routing            | leptos_router 0.8        |
| Reactive utilities | leptos-use 0.18          |
| i18n               | leptos_i18n 0.6          |
| Client storage     | `idb` (IndexedDB), OPFS  |
| WASM utilities     | `gloo`, `web-sys`        |
| HTTP client        | TrailBase REST API       |
| Build tool         | `trunk`                  |

## Leptos 0.8 Patterns

```rust
// Signals — core reactivity
let count = RwSignal::new(0);
let derived: Signal<i32> = Signal::derive(move || count.get() * 2);

// Effects — side reactions
Effect::new(move |_| { tracing::info!("Count: {}", count.get()); });

// Async tasks
spawn_local(async move { /* async operations */ });

// Components — ALL interactive components MUST accept test_id
#[component]
pub fn MyComponent(
    #[prop(optional, into)] test_id: Signal<String>,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <div data-testid=move || test_id.get()>
            {children()}
        </div>
    }
}

// Context — global state
let auth = use_context::<AuthStore>().expect("AuthStore not provided");
```

## Conventions

### Props

- All optional props: `#[prop(optional, into)]`; reactive props: `Signal<T>` or `RwSignal<T>`

### Async Data

- `spawn_local` for fire-and-forget async; `create_resource` for reactive data-fetching
- Loader functions in `loaders/` handle async data initialization

### State Management

- `AuthStore` — auth state, dictionary loading status, repository reference
- `RwSignal<T>` for read-write state; `Signal<T>` for derived; `provide_context`/`use_context`

### i18n

```rust
let i18n = crate::i18n::use_i18n();
let text = i18n.get_keys().ui().loading_animation().inner().to_string();
```

Translations compiled at build time by `leptos_i18n_build`.

### Logging

- **Always:** `tracing::info!("Card loaded: {id}");`
- **Never:** `web_sys::console::log_1` or `console_log!`

### Styling

- Read `DESIGN.md` for the complete design system
- No `border-radius` on components; no `box-shadow` with blur (only hard offset shadows)
- Fonts: Cormorant Garamond (headings) + IBM Plex Mono (UI); animation prefix: `anima-*`

## Routing

Routes defined in `routes.rs`: `/` (home), `/login`, `/onboarding`, `/profile`,
`/words`, `/grammar`, `/phrases`, `/kanji`, `/kanji/:id`, `/lesson`, `/sets`.
`ProtectedRoute` wraps authenticated pages — auto-redirects to Login, triggers dictionary loading.

## Build System

`build.rs` handles at compile time: i18n compilation, Lindera dictionary (UniDic),
well-known set metadata, and env vars (`ORIGA_CDN_BASE_URL` required, plus optional
`ORIGA_CDN_REGION`, `ORIGA_VERSION`, `ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `ORIGA_PUBLIC_BASE_URL`).

### `recursion_limit` landmine (bin crate)

`lib.rs` has `#![recursion_limit = "512"]` but `main.rs` (the `origa_ui_bin`
crate) does NOT — it inherits the default 128. The bin mounts the entire `<App/>`
view tree, and tachys encodes every element's attributes/classes as deeply
nested generic type tuples, so the bin is near the 128 query-depth ceiling. Adding
**new attributes** (`class`, `data-testid`, a second class) to a deep component
tips it over → `error: queries overflow the depth limit!` during
layout/monomorphization (platform-independent; fails `cargo test --workspace`).
Raising the bin's limit to 512 lets it compile but produces mass linker errors
from over-monomorphization — so raising the limit is NOT a fix.

**Guidance:** prefer changing an existing element's class **string** (type-neutral:
`Class<&str>` stays `Class<&str>`) over adding new attributes. The structural fix
is to split deep views into sub-components (delegating rendering to the lib crate,
which has the higher limit) or to add the attribute to a shallower element. A
component **prop** (e.g. `Card`'s `test_id`) is also type-neutral — it is packed
into the Props struct, not added as a view-tree type-param, so passing it does NOT
trip this limit (only attributes on a bare HTML element do). See ADR-027 §B3 and
ADR-029 for concrete cases.

## Development

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"  # REQUIRED
cd tauri && cargo tauri dev          # full app (recommended)
cd origa_ui && trunk serve           # frontend only
```

## Testing

```powershell
cargo test -p origa_ui
cargo test -p origa_ui -- --nocapture  # with output
```

Uses `rstest` for parameterized tests.
