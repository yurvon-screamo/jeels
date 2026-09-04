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

`build.rs` handles at compile time: i18n compilation, well-known set
metadata, and env vars (`ORIGA_CDN_BASE_URL` required, plus optional
`ORIGA_CDN_REGION`, `ORIGA_VERSION`, `ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `ORIGA_PUBLIC_BASE_URL`).
The tokenizer dictionary is not built here — `build.rs` only verifies the
pre-built SudachiDict files exist in `cdn/dictionaries/` (fail fast on a
broken checkout; `cdn/` is gitignored).

### `recursion_limit` (bin crate) — RESOLVED, keep the guardrails

Both `lib.rs` and `main.rs` now carry `#![recursion_limit = "512"]` (the bin
used to inherit 128 and overflow with "queries overflow the depth limit!" —
raising the limit alone used to cause mass linker errors from
over-monomorphization, but that escape hatch is gone: dev-profile debuginfo is
capped in the root Cargo.toml `[profile.dev]`, so oversized artifacts no longer
kill link.exe).

The depth pressure itself is real and still applies: tachys encodes every
element's attributes/classes as nested generic type tuples, so piling
**new attributes** onto deep components grows compile times toward the raised
limit. **Guidance:** prefer changing an existing element's class **string**
(type-neutral: `Class<&str>` stays `Class<&str>`) over adding new attributes.
A component **prop** (e.g. `Card`'s `test_id`) is also type-neutral — it is
packed into the Props struct, not added as a view-tree type-param. Splitting
deep views into sub-components remains the structural fix. See ADR-027 §B3,
ADR-029 and PR #441 for concrete cases.

## Development

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"  # REQUIRED
cd tauri && cargo tauri dev          # full app (recommended)
cd origa_ui && trunk serve           # frontend only
```

### `trunk serve` mutates `dist/` — rebuild before serving it statically

`trunk serve` rewrites `dist/index.html` **on disk** with its live-reload
bridge (a WS script to `.well-known/trunk/ws` whose reconnect handler calls
`window.location.reload()`). `trunk build` does NOT embed it. Consequences:

- A `dist/` that was ever served from is poisoned: `npx serve dist -s` (the
  CI topology) will serve the reload bridge, and on any WS reconnect it
  force-reloads the page — aborting WASM loads, interrupting Playwright
  gotos and restarting long restores mid-flight. This exact noise masked
  itself as an app bug during the N1-stress investigation (#492).
- Before any static serving of `dist/` (local e2e against a release build,
  artifact inspection), run a fresh `trunk build --release` — the rebuild
  overwrites the poisoned `index.html`.

### Browserslist warning (`caniuse-lite is outdated`)

Trunk compiles `input.css` with a standalone `tailwindcss` binary it downloads
itself into `~/.cache/trunk/tailwindcss-3.3.5/` (default version hardcoded in
trunk 0.21.14). The binary embeds browserslist + caniuse-lite frozen at its
build time (Oct 2023); browserslist prints

```text
Browserslist: caniuse-lite is outdated. Please run:
  npx update-browserslist-db@latest ...
```

whenever the newest browser release in the embedded DB is ≥ 6 months old.

`npx update-browserslist-db@latest` is a no-op here: it updates `node_modules`,
which this crate does not have — the data lives inside the ELF binary. Pinning
`[tools] tailwindcss = "3.4.17"` in Trunk.toml does not help either (verified:
the 3.4.17 standalone embeds equally old data; the v3 line gets no new
releases). Silence it with the documented browserslist flag — a runtime env
variable read by the tailwindcss CLI, not a build.rs variable:

```bash
export BROWSERSLIST_IGNORE_OLD_DATA=1    # ~/.bashrc (WSL)
```

```powershell
$env:BROWSERSLIST_IGNORE_OLD_DATA = "1"  # $PROFILE (Windows)
```

The flag only disables the warning; the generated CSS is byte-identical.

Notes:

- Version drift: in the Docker image trunk prefers the system
  `npm i -g tailwindcss@3.4.17` (PATH wins over the hardcoded version), and
  the npm package has no browserslist in its deps — no warning there. Do not
  install tailwindcss globally outside Docker: PATH precedence silently
  switches the compiler used by local builds.
- CI steps `Build WASM` (ci.yml) and `Build frontend` (tauri.yml) still print
  the warning; env is not added because `.github/workflows/` changes require
  explicit approval (root AGENTS.md). When approved: per-step env, or a single
  point in `.github/actions/setup-frontend/action.yml`. Does not affect the
  path-filter/gate invariants.

## Testing

```powershell
cargo test -p origa_ui
cargo test -p origa_ui -- --nocapture  # with output
```

Uses `rstest` for parameterized tests.
