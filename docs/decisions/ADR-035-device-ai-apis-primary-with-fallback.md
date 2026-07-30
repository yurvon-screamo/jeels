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

Investigation surfaced three hard constraints that shape the decision:

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
  dev hardware). Verification is compile-time only: a `device-ai-build` CI job
  on `macos-latest` checks the primary path compiles. Runtime smoke is the
  owner's responsibility; the kill-switch is the escape hatch.
- The `device-ai` crate compiles into the macOS/iOS/Android binary even under
  `--features disable-device-ai` (Cargo cannot combine `[target.cfg]` deps with
  feature gates without `optional = true`). This is accepted binary bloat for
  the kill-switch build.
