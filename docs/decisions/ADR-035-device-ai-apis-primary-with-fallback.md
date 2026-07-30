# ADR-035: device-ai-apis as primary ASR/OCR/TTS with fallback stack

## Status

Accepted

## Date

2026-07-30

## Context

Origa's ASR/OCR/TTS previously ran entirely in-browser via WASM: Whisper
(encoder/decoder/tokenizer downloaded from CDN, ~100 MB, ONNX inference in
`ort`/`ort-web`), NDLOCR-Lite (DEIM + PARseq cascade), and `speechSynthesis`/
`tauri-plugin-tts`. This works cross-platform but pays a heavy cost in model
download size, inference latency, and battery on every platform that *also*
ships capable native AI APIs (Apple Speech/Vision, Android ML Kit,
Windows.Media.*).

The user requested `tauri-plugin-device-ai-apis` (hypothesi/tauri-plugin-device-
ai-apis) be adopted as the **primary** source for ASR, OCR, and TTS on every
platform, keeping the current stack as a fallback (web, unsupported devices).

Investigation surfaced hard constraints that shape the decision:

1. **The plugin does not compile on Windows.** Its `crates/device-ai/src/
   windows.rs` is written against a `windows-rs` API that no longer exists in
   the resolved `windows = "0.58"` — `SpeechRecognizer::CreateWithLanguage` is
   gone, `HRESULT`/`Error` handling drifted. Even if it compiled, the README
   states Windows TTS *"completes synthesis, but does not yet play the generated
   stream."*
2. **Linux has no native backend.** `device-ai` returns `FeatureNotAvailable`
   for every feature on Linux.
3. **The plugin's JS `speech_recognize` accepts only live-microphone input.**
   File-based recognition requires calling the `device-ai` Rust crate directly,
   and that crate's speech backend only implements macOS and Windows.
4. **Upstream mobile bridge did not compile** (discovered when Origa's CI first
   built Android). The plugin's own `test.yml` only runs `cargo fmt/clippy/test`
   on `ubuntu-latest` — it never builds the Android/iOS targets — so two errors
   went unnoticed from the initial commit: `commands::get_capabilities` calls a
   method absent on the mobile `DeviceAiApis` impl (defined only on desktop),
   and `init()` used `?` on `register_android_plugin`/`register_ios_plugin`
   without a `From<PluginInvokeError>` impl. The README still advertises
   iOS/Android support; the code did not build.

   **Resolution:** depend on a fork (`yurvon-screamo/tauri-plugin-device-ai-apis`,
   rev `e5709f6`) that fixes all three mobile-bridge layers (Rust `mobile.rs`,
   Android Kotlin, iOS Swift — see Alternatives). The fixes will be upstreamed;
   until merged, the fork carries them.

## Decision

Adopt device-ai as primary on **macOS, iOS, Android** only, with a strict
fallback stack that is never a regression.

### Compilation scoping

- `tauri-plugin-device-ai-apis` and `device-ai` are declared under
  `cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))` in
  `tauri/Cargo.toml`. Windows and Linux do not compile or link them.
- A compile-time kill-switch feature `disable-device-ai` drops the plugin and
  the `device_ai_recognize_file` command, forcing the fallback stack on every
  platform. Use `cargo ... --features disable-device-ai`.

### Routing matrix (runtime, capabilities-first)

The frontend is one WASM binary shared across every Tauri host, so `target_os`
is always `unknown` there. Routing is therefore **runtime** via
`device_ai::available(feature)`, backed by a cached `get_capabilities` query:

| Platform        | ASR (file)        | ASR (live)        | OCR               | TTS                |
|-----------------|-------------------|-------------------|-------------------|--------------------|
| macOS (Tauri)   | device-ai (crate) | device-ai (JS)    | device-ai         | device-ai          |
| iOS/Android     | Whisper WASM      | device-ai (JS)    | device-ai         | device-ai          |
| Windows (Tauri) | Whisper WASM      | Whisper WASM      | NDLOCR WASM       | tauri-plugin-tts   |
| Linux (Tauri)   | Whisper WASM      | Whisper WASM      | NDLOCR WASM       | tauri-plugin-tts   |
| Web             | Whisper WASM      | — (file only)     | NDLOCR WASM       | speechSynthesis    |

Any device-ai failure (capability unavailable, invoke error, timeout) collapses
to the matching fallback in the same row.

### Key invariants

- **Capabilities are cached only on success.** A transient query failure
  (plugin warming up, timeout) returns `all_unavailable` for the current call
  but is *not* cached, so the next call retries — a single early failure cannot
  permanently disable device-ai for the session.
- **`tauri-plugin-tts` is retained** and remains the TTS backend on
  Windows/Linux (and is best-effort-stop elsewhere). Removing it would regress
  Windows, where device-ai TTS cannot play audio.
- **Capabilities are ACL-gated** in `capabilities/device-ai.json` (macOS) and
  `capabilities/mobile.json` (iOS/Android) with explicit permission identifiers;
  Windows/Linux capabilities deliberately omit them so the build does not fail
  on the unregistered plugin.
- `NSSpeechRecognitionUsageDescription` added for macOS/iOS; the speech
  recognition entitlement does not exist in the sandbox list, so only the usage
  description is required.

## Alternatives Considered

### Fork the plugin (mobile bridge fix) — ADOPTED

The upstream plugin never compiled its mobile targets in CI (`test.yml` only
runs `cargo fmt/clippy/test` on `ubuntu-latest`), so the mobile bridge was
broken at three layers, all surfaced when Origa's CI first built Android/iOS:

1. **Rust `mobile.rs`** — `get_capabilities` was defined only on the desktop
   `DeviceAiApis` impl, and `init()` used `?` on `register_*_plugin` without
   `From<PluginInvokeError>`. Fixed by adding `get_capabilities` (returning the
   documented iOS/Android feature matrix) and applying the existing
   `mobile_invoke_error` mapper.
2. **Android Kotlin `DeviceAiPlugin.kt`** — four array-returning commands used
   `Invoke.resolve(JSONArray)`, which takes `JSObject`. Switched to
   `Invoke.resolveObject` (matches tauri-apps/plugins-workspace notification).
3. **iOS Swift `DeviceAiPlugin.swift`** — `Invoke.reject` takes a `String` but
   was passed the `Encodable` `PluginError` (~20 sites); added an
   `Invoke.reject(_ error: PluginError)` overload that JSON-encodes it.
   `VNRecognizeTextRequest.recognitionLevel` assignments qualified as
   `VNRequestTextRecognitionLevel.fast/.accurate`; `UIImage(data:)` wrapped
   `[UInt8]` in `Data(...)`; iOS 26 FoundationModels `LanguageModelSession`
   `@InstructionsBuilder` mismatch resolved by dropping system-prompt
   instructions at the two LLM call sites (Origa does not use LLM).

The fork `yurvon-screamo/tauri-plugin-device-ai-apis` (rev `e5709f6`) carries
all of the above. The mobile build was the main reason for adopting device-ai,
so fixing the bridge was mandatory rather than excluding mobile. The fixes
will be upstreamed via a PR to `hypothesi/...`; until merged, Origa depends on
the fork.

### Fork the plugin and fix `windows.rs`

- Pros: Windows could use device-ai too.
- Cons: Substantial WinRT-API spelunking (~9 distinct compile errors), and
  Windows TTS playback is still unimplemented upstream — OCR/ASR-only gain for
  a platform already served by a working stack.
- Rejected now; left as future work. The kill-switch makes reverting trivial.

### Route device-ai on Windows via capabilities only

- Pros: One code path.
- Cons: The crate does not compile, so it cannot even be linked.
- Rejected.

## Consequences

- **Windows and Linux keep their current, working AI stack.** This is the
  non-regression guarantee: those platforms are byte-for-byte unaffected by
  device-ai's absence.
- **macOS/iOS/Android get native ASR/OCR/TTS** with no model download when the
  capability is present.
- **Known limitations carried forward** (documented in code):
  - device-ai `synthesize` is blocking and **not interruptible** —
    `stop_speech` is a no-op on the device-ai path (best-effort on plugin:tts).
  - **Mobile file-ASR falls back to Whisper WASM** — the plugin JS API accepts
    only live-microphone input, and the device-ai Rust speech backend has no
    iOS/Android implementation. Live-mic on mobile is native.
  - **Web has no live-microphone ASR** — MediaRecorder→Whisper PCM resampling
    is a separate enhancement.
- **Runtime quality on macOS/iOS/Android is not locally verifiable** (no such
  dev hardware). Compile-time verification: the `Build Tauri / Build Android`
  matrix job builds the mobile (Android) device-ai path, and the macOS/iOS
  paths were validated in CI before the temporary device-ai-* jobs were
  removed. Runtime smoke on a real device remains the owner's responsibility;
  the kill-switch is the escape hatch.
- The `device-ai` crate compiles into the macOS/iOS/Android binary even under
  `--features disable-device-ai` (Cargo cannot combine `[target.cfg]` deps with
  feature gates without `optional = true`). This is accepted binary bloat for
  the kill-switch build.
