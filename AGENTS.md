# AGENTS.md — Origa

**Origa** — приложение для изучения японского языка (FSRS, OCR, STT, токенизация).
Репозиторий: <https://github.com/yurvon-screamo/origa>

## Стек

| Слой           | Технология                                                                |
|----------------|---------------------------------------------------------------------------|
| Workspace      | Rust 2024 edition, id `net.uwuwu.origa`                                   |
| Бизнес-логика  | `origa/` — Clean Architecture (Use Cases → Domain → Traits)               |
| Frontend       | `origa_ui/` — Leptos 0.8, CSR/WASM, trunk                                 |
| Landing        | `origa_landing/` — Leptos 0.8, SSR/Axum, i18n (EN+RU)                     |
| Desktop        | `tauri/` — Tauri v2 (Windows, Linux, macOS)                               |
| E2E            | `end2end/` — Playwright (TypeScript)                                      |
| CDN / Storage  | Tigris (S3-compatible, user-owned); bucket `origa-cdn`                    |
| Утилиты        | `utils/`, `scripts/` (Python)                                             |

## Структура проекта

```text
origa/          — domain, use_cases, traits, ocr, stt, dictionary
origa_ui/       — Leptos 0.8 frontend (WASM)
origa_landing/  — SSR landing site (Leptos 0.8 + Axum)
tauri/          — Tauri v2 desktop app
end2end/     — Playwright E2E тесты
utils/       — CLI утилиты
cdn/         — статический контент (dictionaries, grammar, kanji_animations, ndlocr, phrases, pitch, well_known_set)
scripts/     — Python скрипты обработки данных
docs/        — документация (decisions/)
models/      — ML модели
```

## Среда разработки

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"  # ОБЯЗАТЕЛЬНО
cd tauri && cargo tauri dev
```

### Android dev (эмулятор)

Debug WASM-бандл раздувается до ~300 МБ и крашит Android WebView кучу
(`OutOfMemoryError` в `RustWebViewClient.shouldInterceptRequest`,
tauri-apps/tauri#13554). Для `cargo tauri android dev` обязательно
использовать профиль `wasm-dev` (определён в корневом `Cargo.toml`):
стрип debug-символов + opt-level=1, компилируется быстро, WASM ~8–19 МБ.

```powershell
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"
$env:TRUNK_BUILD_CARGO_PROFILE = "wasm-dev"
cd tauri && cargo tauri android dev
```

### Переменные окружения (compile-time, `build.rs`)

Обязательные: `ORIGA_CDN_BASE_URL`; для сборки `origa_landing` (clippy/test) — также `ORIGA_APP_BASE_URL` (или пара `ORIGA_BASE_URI` + `ORIGA_APP_URI_PREFIX`), без неё `origa_landing/build.rs` паникует.
Опциональные: `ORIGA_CDN_REGION`, `ORIGA_VERSION`, `ORIGA_COMMIT`, `ORIGA_BUILD_DATE`, `TRAILBASE_URL`, `ORIGA_LANDING_BASE_URL` (дефолт `https://origa.uwuwu.net`), `SENTRY_DSN`, `SENTRY_ENVIRONMENT`.

**Sentry** (ADR-036): единый `SENTRY_DSN` пробрасывается во все build-скрипты; пустой/не задан = Sentry отключен. `SENTRY_ENVIRONMENT` маппится в CI из `version_type` (`stable`→`production`, `prerelease`→`staging`, иначе `development`). `SENTRY_RELEASE` выводится из `ORIGA_VERSION` (отдельная CI-переменная не нужна). Локально для теста Sentry:

```powershell
$env:SENTRY_DSN = "https://<public_key>@o<orgid>.ingest.sentry.io/<projectid>"
$env:SENTRY_ENVIRONMENT = "development"
$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"
cd tauri && cargo tauri dev
```

**DNS naming scheme** (CI/CD production):

- `ORIGA_BASE_URI` — base domain (e.g. `origa.uwuwu.net`)
- `ORIGA_CDN_URI_PREFIX` — CDN subdomain prefix (e.g. `cdn` → `s3.origa.uwuwu.net`)
- `ORIGA_APP_URI_PREFIX` — app subdomain prefix (e.g. `app` → `app.origa.uwuwu.net`)
- Landing = base domain (no prefix)

**Local dev:** `$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"` (production CDN endpoint — read-only, safe to use directly; cache policy is tiered, see CDN / S3 below)
**Landing dev:** `$env:ORIGA_CDN_BASE_URL = "https://s3.origa.uwuwu.net"; $env:ORIGA_APP_BASE_URL = "https://app.origa.uwuwu.net"` (`ORIGA_LANDING_BASE_URL` необязателен — дефолт `https://origa.uwuwu.net`)

## Команды

```powershell
cargo test --workspace                              # все тесты
cargo test -p origa -- --nocapture                  # с выводом
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check && cargo fmt
```

Тесты: `rstest` (параметризованные). Конфиги: `.rustfmt.toml` (max_width=100), `clippy.toml` (complexity=25).

## Ключевые зависимости

`rs-fsrs` (FSRS), `ort` + NDLOCR-Lite (OCR), `ort` + `rustfft` (Whisper STT),
`lindera 5` + SudachiDict (токенизация; сборка: `scripts/build_sudachidict.py`), `serde`/`bincode`/`rkyv` (сериализация),
`rusqlite` (БД), Leptos 0.8 + `leptos_router`/`leptos-use`/`leptos_i18n` (frontend),
`sha2`/`hmac` (TrailBase auth), `tracing`/`tracing-wasm` (логирование).
Плагины: opener, tts, deep-link (`origa://`), single-instance, updater, process.

## CDN / S3

Tigris object storage (S3-compatible, endpoint `t3.storageapi.dev`) под Railway — bucket `adaptable-foodbox-ucep7wx`, раздача через Railway s3-proxy + edge caching: URL `https://s3.origa.uwuwu.net` вшивается через `build.rs`. Трейт: `origa/src/traits/cdn_provider.rs`, реализация: `origa_ui/src/repository/cdn_provider.rs`. Миграция на user-owned Tigris (ADR-037) была откачена в #372 (DPI-throttle на Cloudflare-роутинге для РФ); orphaned-ресурсы той миграции (user-Tigris bucket `origa-cdn` ~4 GB, R2, Worker) ждут cleanup.

Профиль `~/.aws/credentials [origa]` — для `deploy_cdn.py` / `refresh_cache_control.py`. Контракт кредов: env `AWS_ACCESS_KEY_ID` при наличии приоритетнее профиля — CI передаёт scoped-ключи через env.

Все объекты — статические, но кэшируются по-разному в зависимости от частоты изменений. Политика в `scripts/_cdn_cache.py`, применяется в `deploy_cdn.py`.

- **Truly-static** (`public, max-age=31536000, immutable`): ML-модели (`ndlocr/`, `whisper/`), kanji SVG/frames (`kanji_animations/`, `kanji_frames/`), audio фраз (`phrases/audio/`), системный словарь lindera (`dictionaries/`)
- **Release-updated** (`public, max-age=300, must-revalidate`): контент-JSON — `grammar/`, `dictionary/`, `phrases/phrase_index.json`, `phrases/data/`, `pitch/`, `well_known_set/`
- **Always-fresh** (`no-cache`): `manifest.json`

immutable уместен только для truly-static файлов. `grammar`/`phrases`/`dictionary` обновляются каждый релиз (W-11, P-3, L-4, S-3) — для них immutable означал CDN edge-cache poisoning (PR #182): S3 обновлялся, а edge держал годовой кэш и отдавал устаревшую версию, пока кэш не сбросили вручную.

```powershell
python scripts/deploy_cdn.py            # генерация манифеста + инкрементальный деплой (по политике)
python scripts/deploy_cdn.py --dry-run  # показать что будет залито + Cache-Control каждого файла
```

Манифест (`manifest.json`) содержит SHA256 хеши версионных файлов и позволяет клиенту обнаруживать обновления. Деплоится с `Cache-Control: no-cache`.

### Microsoft Store (MSIX, ADR-042)

Store-дистрибуция — **MSIX-пакет**: Store сам подписывает и хостит его, CA-сертификат не нужен (linked-EXE путь ADR-041 с CDN-зеркалом `releases/` удалён — Store отклонял неподписанный EXE по 10.2.9, а подписка недоступна). Сборка: `tauri/scripts/build-msix.ps1` (единственный источник: store-build `ORIGA_APP_STORE=1` + стейджинг + MakeAppx/signtool с эфемерной самоподписью). CI: job `build-windows-store` в отдельном reusable `_build-windows-store.yml` — stable-теги + `workflow_dispatch` input `force_store_msix` (smoke → версия `0.0.0.<run>`, НЕ сабмитить). Артефакт `.msix` + `.pfx` + пароль живёт 7 дней в Actions.

```powershell
# локальная smoke-сборка и установка:
cd tauri; ./scripts/build-msix.ps1 -Version 0.7.0
# импортировать tauri/target/msix/Origa.msix.pfx в Cert:\CurrentUser\Trusted People (пароль в .password.txt)
Add-AppxPackage -Path tauri/target/msix/Origa_0.7.0_x64.msix
```

Updater в store-сборке скомпилирован out (`not(app_store)` гейт, политика 10.2.5); прямые загрузки NSIS обновляются через Tauri updater как раньше. Identity-плейсхолдеры `__PARTNER_CENTER_*__` в манифесте заполняются после создания «MSIX or PWA app» продукта в Partner Center. Пересабмит = бамп версии. Runbook: ADR-042.

Обновить Cache-Control на существующих объектах (one-time, после смены политики — новые upload'ы уже корректны, но старые объекты хранят прежний заголовок):

```powershell
python scripts/refresh_cache_control.py --dry-run  # read-only: HEAD на каждый объект, показывает что изменится
python scripts/refresh_cache_control.py             # применить (server-side copy-object с REPLACE metadata)
```

Идемпотентен: обновляются только объекты с неверным Cache-Control. При сбое на середине объект пропускается, скрипт продолжает; повторный запуск дозаполнит остальное. Ключи с shell-метасимволами (потенциальная инъекция через `pwsh -Command`) отбрасываются с предупреждением; CJK-имена kanji-файлов (`一.svg` и т.п.) обрабатываются корректно.

## CI/CD

Workflows: `ci.yml`, `docker.yml`, `tauri.yml`, `cleanup-cache.yml`.
CI: lint + test + e2e + docker build (2 images: landing + ui).
CD: 2 Docker images (GHCR) + Railway deploy (2 services).
Targets: Windows x86_64, Linux x86_64, macOS aarch64. Релиз при push `master` + tag `v*.*.*`.

## Границы

### ✅ ВСЕГДА

- `cargo clippy --workspace --all-targets -- -D warnings` + `cargo fmt` + `cargo test --workspace` перед коммитом
- `ORIGA_CDN_BASE_URL` установлена перед сборкой

### ⚠️ СПРОСИТЕ СНАЧАЛА

- Изменения в `Cargo.toml` (workspace deps), `.github/workflows/`, `origa/src/domain/`, линтер-конфигах

### 🚫 НИКОГДА

- Коммит без тестов / `unwrap()` в production / `#[async_trait]` / `#[allow(dead_code)]`
- `println!` / `console.log` в production / удаление тестов
- Sans-serif шрифты (только Cormorant Garamond + DM Mono)
- `border-radius` на основных UI / `box-shadow` с blur (только жёсткие offset-тени)

## Документация лендинга (`/docs/*`)

Раздел `/docs` — markdown-driven (как `content/blog/`), контент в `content/docs/{en,ru}/*.md`,
2 языка (EN+RU). Landing = единственный LLM-facing surface для Origa (GPTBot/ClaudeBot/PerplexityBot
не индексируют GitHub для японских запросов), поэтому критична актуальность и точность.

### Tone rules (наследуются от лендинга + дополнения)

- Те же правила, что в `docs/landing-content-plan.md §10`: без pricing, open source, license, BSL,
  без superlatives, без humor, прямое обращение "you"/"вы".
- **Без сравнений с конкурентами.** `/compare` уже это закрывает. docs = "как работает + как пользоваться",
  не "чем лучше".
- **Без конкретных чисел**, которые могут меняться между релизами: cards/day, FSRS-параметры, размеры
  корпусов в точных значениях, latency в мс. Качественные описания вместо цифр.
- Числа разрешены только если они стабильны: JLPT levels N5–N1, форматы файлов (`.apk`, `.deb`),
  версии моделей.

### Definition of Done (релиз)

Если в релизе меняется behavior фичи, упомянутой в `/docs/*`, контент соответствующей doc-страницы
обновляется в том же PR. Markdown-файлы коммитятся вместе с кодом.

## Git

Коммиты на английском. Ветка: `master`. Теги: `v*.*.*` для релизов.
