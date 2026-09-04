# ADR-044: Android JNI context ownership (ndk-context publication)

## Status

**Accepted** (2026-09-03).

## Date

2026-09-03.

## Context

`init_platform_verifier_android` (added in #355) reads the JavaVM and the
Activity context from the `ndk-context` crate global and relies on the
windowing layer to populate it before Rust startup code runs. That contract
was implicit: `tao 0.34` — the windowing library beneath wry beneath tauri —
called `ndk_context::initialize_android_context()` while starting the
activity, and our code read what it left behind. Nothing in `Cargo.toml`
declared the dependency on that side effect.

Tauri 2.11 (#443) pulled in `tao 0.35`, whose mobile multi-window refactor
removed the `ndk-context` dependency entirely. The moment it landed, the
first read on Android aborted the process on every launch:

```text
PANIC at ndk-context/src/lib.rs:72: android context was not initialized
```

Nothing in our code changed; an undocumented side effect of a transitive
dependency disappeared. The same break hit every Tauri 2.11 app reading
`ndk-context` (android-native-keyring-store#21, plugins-workspace#2900).
CI did not catch it: host clippy/test never compiles
`#[cfg(target_os = "android")]` code, the release pipeline builds the APK
without running it, and Playwright e2e drives the web build in a desktop
browser — no CI layer executed the app on Android.

## Decision

We own the Android context invariant instead of assuming it:

1. `tauri/src/android_context.rs` (Android-only) captures the JavaVM in
   `JNI_OnLoad` — the JVM invokes it on `System.loadLibrary`; neither tao,
   wry nor tauri define that symbol, so there is no collision. The
   Application context is resolved via `ActivityThread.currentApplication()`
   (a hidden but de-facto stable API, standard practice upstream) and pinned
   as a leaked global ref for the process lifetime.
2. Both are published into `ndk-context::initialize_android_context` under a
   CAS guard (ndk-context asserts on re-publication; a second `JNI_OnLoad`
   is possible on classloader splits). Publishing into the shared global —
   rather than private statics — fixes every reader, including transitive
   ones (`tauri-plugin-tts` depends on `ndk-context`).
3. The Application context (not the Activity) is published: an Activity can
   be torn down and recreated at any time (the deep-link recreate flow,
   ADR-010), while the Application object outlives every Activity — and it
   is what the verifier keeps a reference to for its entire lifetime.
4. Failure policy: every step degrades to `false` + `tracing::error!` +
   logcat (via `android.util.Log` JNI calls). Nothing in the module panics —
   it runs across the FFI boundary and release profiles build with
   `panic = "abort"`.
5. Single-publisher assumption: only this module calls
   `initialize_android_context`. The CAS guard protects against our own
   re-entry; a hypothetical foreign publisher trips ndk-context's assert
   (abort in release, caught by `catch_unwind` in unwind-enabled profiles).
6. Observability: two startup markers are emitted to logcat —
   `[android-context] published JavaVM+Application to ndk-context` and
   `[rustls] platform-verifier initialized for Android` — because tracing on
   Android has no logcat backend (Sentry-only) and native stdout goes to
   /dev/null. The `tracing::info!` counterparts are kept for Sentry.
7. CI net: the `android-smoke` job (ci.yml) builds a debug APK, boots an
   emulator, launches the app and asserts process liveness plus both
   markers, and fails on the ndk-context panic text. It also runs
   `cargo check -p origa-app --lib --target x86_64-linux-android` as a
   minutes-scale fail-fast for code host clippy never sees.

## Alternatives considered

| Alternative | Verdict |
| --- | --- |
| Pin tao < 0.35 / Tauri < 2.11 | Blocks security updates; the break is a supported upstream configuration, not a bug to freeze out. |
| `webpki-roots` instead of the platform verifier | Removes JNI entirely, but changes the TLS trust model (no user/enterprise CAs, roots frozen per release) and requires workspace-wide reqwest feature surgery. |
| `RuntimeHandle::run_on_android_context` | The sanctioned dispatch path, but timing-fragile (panics when the activity is not registered yet) and fixes only our reader while `tauri-plugin-tts` keeps reading an unpublished global. |

## Consequences

- The first read of `ndk_context::android_context()` on Android is now
  guaranteed populated by code we own, not by a transitive side effect.
- The smoke job adds ~20–30 min to cold PR runs touching `core`/`tauri`
  (workspace build with ort/lindera + emulator image + boot; warm runs are
  materially cheaper via the rust cache). Accepted trade-off: it is the only
  automated net for Android startup crashes.
- The smoke test runs the x86_64 ABI (emulator), not the production aarch64;
  the fixed code is ABI-independent (pure JNI, no ISA-specific paths), and
  production builds stay aarch64 via `_build-tauri.yml`.
- If Tauri later restores ndk-context publication upstream, OUR publication
  must be removed at the same time: with `panic = "abort"` release profiles
  a second publisher aborts the process on ndk-context's assert regardless
  of pointer validity — coexistence is not safe by construction.
