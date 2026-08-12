# Plan: macOS Native App Fixes (updater, OIDC, NoResults UX)

## Проблема 1: Отключить автоапдейт для macOS

### Root Cause

`tauri/src/lib.rs:81-93` — updater-плагин и команды регистрируются под gate:

```rust
#[cfg(all(desktop, not(any(feature = "app-store", app_store))))]
```

`desktop` = Windows + macOS + Linux. CI macOS builds уже ставят `ORIGA_APP_STORE=1`,
что отключает updater через `app_store` cfg — но это workaround, а не явное решение.
Локальные dev-builds и не-App-Store builds на macOS всё ещё регистрируют updater.
App Store теперь обрабатывает обновления → updater на macOS не нужен вовсе.

Дополнительно: фронтенд `check_for_updates()` (`app.rs:68`) делает IPC-вызов без
проверки платформы → на macOS получает "command not found", логирует warning.

### Затрагиваемые модули

- `tauri/src/lib.rs` — cfg-gate для updater plugin registration + commands
- `tauri/src/updater_commands.rs` — `#[cfg(desktop)]` → `#[cfg(any(windows, target_os = "linux"))]`
- `tauri/Cargo.toml` — зависимость `tauri-plugin-updater` (target-gate)

### Порядок реализации

1. Заменить `#[cfg(all(desktop, not(any(feature = "app-store", app_store))))]` на
   `#[cfg(any(windows, target_os = "linux"))]` во всех updater-related местах в `lib.rs`:
   - `mod updater_commands` (line 17)
   - `use Manager` import gate (line 22)
   - `use updater_commands::{...}` import (line 28)
   - plugin registration block (lines 81-93)
   - `check_for_update` + `install_update` в `invoke_handler` (lines 137-140)
2. В `updater_commands.rs`: заменить `#[cfg(desktop)]` на
   `#[cfg(any(windows, target_os = "linux"))]` для `check_for_update` и `install_update`
3. В `tauri/Cargo.toml`: добавить target-gate для `tauri-plugin-updater`,
   `tauri-plugin-process`, `tauri-plugin-single-instance` — только Windows + Linux
4. Удалить `app_store` cfg-gate логику для updater (ставится избыточной): строки
   `#[cfg(all(desktop, not(any(feature = "app-store", app_store))))]` везде заменить
   на `#[cfg(any(windows, target_os = "linux"))]`
   - `build.rs` `ORIGA_APP_STORE` updater-patch остаётся (нужен для bundler:
     убирает `createUpdaterArtifacts` и `plugins.updater` из TAURI_CONFIG)

**Важно:** `app_store` cfg в `build.rs` остаётся, т.к. CI iOS тоже использует `ORIGA_APP_STORE`.
Меняется только Rust-side gating updater-плагина.

### Критерии приёмки

- `cargo check` проходит на всех таргетах
- На macOS dev-build updater не регистрируется, IPC-команды `check_for_update`/`install_update` отсутствуют
- Windows/Linux updater работает как прежде
- В `app.rs` фронтенд-проверка `is_tauri()` остаётся (updater::check_for_updates возвращает None при ошибке invoke)

---

## Проблема 2: OIDC не возвращается в приложение на macOS

### Root Cause

ДВА независимых дефекта:

**Дефект A** — `tauri/src/lib.rs:178`:

```rust
#[cfg(any(windows, target_os = "linux"))]
{
    match app.deep_link().register_all() { ... }
}
```

`register_all()` (регистрация custom-scheme в OS) вызывается **только на Windows и Linux**.
macOS пропущена → Launch Services не знает про `origa://`.

**Дефект B** — `tauri/MacOS-Info.plist` **не содержит** `CFBundleURLTypes`:

```xml
<!-- Нет записи о URL scheme "origa" -->
```

На macOS custom-scheme обрабатывается через `CFBundleURLTypes` в Info.plist.
tauri-bundler мерджит `bundle.macOS.infoPlist` (`tauri.conf.json:50`) в финальный Info.plist.
Без `CFBundleURLTypes` macOS не маршрутизирует `origa://auth/callback?code=...` в приложение.

**Цепочка событий:**

1. App → `opener.openUrl(oauth_url)` → системный браузер (Safari/Chrome)
2. Браузер → TrailBase OAuth → redirect на `desktop-callback.html?code=...`
3. `desktop-callback.html` → `window.location.href = 'origa://auth/callback?code=...'`
4. macOS НЕ находит зарегистрированной схемы → открывает Safari с ошибкой или игнорирует
5. App: `get_current_deep_link()` → `None` (симптом Юры: «в ответ приходит Null»)

**Почему iOS работает через ASWebAuth, а desktop-macOS — нет:**
iOS использует `tauri-plugin-aswebauth` (ASWebAuthenticationSession), который перехватывает
callback напрямую. Desktop (вкл. macOS) использует opener + deep-link listener flow.
Deep-link flow на macOS сломан из-за отсутствия регистрации схемы.

### Затрагиваемые модули

- `tauri/src/lib.rs` — cfg-gate `register_all()` (добавить macOS)
- `tauri/MacOS-Info.plist` — добавить `CFBundleURLTypes`

### Контракты

`MacOS-Info.plist` — добавить:

```xml
<key>CFBundleURLTypes</key>
<array>
    <dict>
        <key>CFBundleURLName</key>
        <string>net.uwuwu.origa</string>
        <key>CFBundleURLSchemes</key>
        <array>
            <string>origa</string>
        </array>
    </dict>
</array>
```

### Порядок реализации

1. Добавить `CFBundleURLTypes` в `tauri/MacOS-Info.plist`
2. В `lib.rs:178` заменить `#[cfg(any(windows, target_os = "linux"))]` на
   `#[cfg(any(windows, target_os = "linux", target_os = "macos"))]`
   для `register_all()` блока

### Критерии приёмки

- `cargo check` проходит
- `MacOS-Info.plist` содержит `CFBundleURLTypes` со схемой `origa`
- macOS dev-build: `register_all()` вызывается (видно в логах)
- При OIDC login на macOS: браузер редиректит на `origa://auth/callback?code=...`,
  OS открывает приложение, `deep-link-received` event доходит до WASM

### Риски

- `register_all()` на macOS sandbox: App Store sandbox может блокировать `LSRegisterURL`.
  Но `CFBundleURLTypes` в Info.plist достаточно для sandbox — Launch Services сканирует
  bundle при установке. `register_all()` нужен только для dev builds (не подписанных).
- App Store build всё ещё работает: `CFBundleURLTypes` в Info.plist не конфликтует с sandbox.

---

## Проблема 3: UX dead-end «Слов не найдено»

### Root Cause

`origa_ui/src/pages/words/add_words_preview_modal.rs:170-182` — `AnalysisStage::NoResults`
рендерит только текст «Слов не найдено» + табы, но **не рендерит контролы ввода**.
`has_analyzed` = true навсегда → stage остаётся `NoResults` → пользователь не может:

- Загрузить другой файл
- Переключиться на другой таб и ввести текст
- Закрыть modal (кнопка закрытия только в `PreviewStage`)

Юзер видит dead-end экран без возможности действовать.

### Затрагиваемые модули

- `origa_ui/src/pages/words/add_words_preview_modal.rs` — view-логика
- `origa_ui/src/pages/words/add_words_preview_modal_state.rs` — `analysis_stage()` функция
- `origa_ui/locales/ru.json`, `origa_ui/locales/en.json` — i18n ключи

### Порядок реализации

1. В `add_words_preview_modal_state.rs`: убрать `NoResults` из `analysis_stage()` —
   возвращать `Input` когда слов не найдено. `has_analyzed` сигнал остаётся для
   отображения informational notice.
   - Обновить unit-тесты: `no_results_after_analysis` → `Input`
   - Удалить `NoResults` вариант enum (или оставить как deprecated/unused — предпочтительно удалить)

2. В `add_words_preview_modal.rs`: в `AnalysisStage::Input` добавить
   informational notice когда `has_analyzed && analyzed_words.is_empty()`:

   ```
   <Show when=...>
     <Alert type=Info>words_not_found + hint "try another file or method"</Alert>
   </Show>
   ```

   Рендерится ВЫШЕ контролов ввода (до табов или после — по UX логике после табов, до input).

3. i18n: добавить ключ `words.words_not_found_hint`:
   - EN: "Try a different file or input method."
   - RU: "Попробуйте другой файл или способ ввода."

### UX rationale

После неудачного анализа (0 слов) юзер видит:

1. Сообщение «Слов не найдено» + подсказку
2. Контролы ввода (те же что в `Input`) — сразу можно загрузить другой файл,
   переключить таб, ввести текст вручную
3. Dead-end устранён: юзер может действовать, а не закрывать modal

### Критерии приёмки

- Unit-тест: `analysis_stage(0, true, false) == AnalysisStage::Input`
- После анализа с 0 слов: показывается notice + контролы ввода
- После успешного анализа: показывается Preview (без изменений)
- Переключение табов работает в NoResults-ситуации

### Стратегия верификации

- Unit-тесты `analysis_stage()` обновить
- Cargo test для `add_words_preview_modal_state`
- Manual: загрузить картинку без текста → увидеть notice + контролы → переключить таб → ввести текст

---

## NOTICED BUT NOT TOUCHING

- `build.rs` `ORIGA_APP_STORE` logic для bundler (остаётся как есть — нужна для CI)
- `app_store` cfg flag (остаётся — iOS использует)
- Frontend `is_macos()` helper (не нужен — check_for_updates уже graceful-fails)
- `desktop-callback.html` (используется для desktop OIDC, не ломается)
- iOS ASWebAuth flow (не затрагивается)
