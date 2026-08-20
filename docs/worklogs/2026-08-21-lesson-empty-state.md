# Worklog: Умное пустое состояние урока + пер-урок лимит фраз

- Дата: 2026-08-20 — 2026-08-21
- Задача: пустой урок должен объяснять причины (исчерпание колоды / дневной
  лимит / ближайшее повторение) и предлагать действия; новые якорные фразы
  отвязать от дневного бюджета (пер-урок кап из DailyLoad).
- Решение зафиксировано в [ADR-040](decisions/ADR-040-per-lesson-phrase-budget.md).

## Что сделано

1. **BDD-first** (`end2end`): `lesson_empty_state.feature` (5 сценариев),
   шаги, локаторы LessonPage; шаг «начинается новый урок или ошибка»
   переименован в «…или пустое состояние» + dead-step grep.
2. **Domain** (`origa`): `DailyBudget` VO (`value_objects.rs`),
   `DailyLoad::new_phrases_per_lesson = new_cards_per_day × 2`
   (паритет первого урока дня), `cards_to_lesson(budget)` вместо `usize`
   (~27 вызовов), удалён `compute_phrase_new_budget`/`PHRASE_NEW_RATIO`,
   переписаны док-комментарии (Option alpha → пер-урок кап).
   Новый `empty_diagnosis.rs`: `diagnose_empty_lesson()` + rstest.
3. **UI** (`origa_ui`): `LessonEmptyState` (CTA → /sets, /profile,
   next-review), `content.rs` интеграция, i18n en+ru (empty_*),
   `lesson.no_cards` оставлен только для grammar-practice (gated).
4. **Landing**: `fsrs` добавлен в `SIDEBAR_SLUGS` (после `lesson`) —
   страница существовала, но не попадала в сайдбар.
5. **Тесты**: rstest 6 уровней DailyLoad, паритет верхней границы,
   дискриминирующий тест второго урока (sanity: бюджет ≥ кап, rate_card),
   фикстура фразового индекса расширена 8 → 15.

## Gotchas

- **recursion_limit bin-крейта** (`origa_ui_bin`, потолок 128): первый
  вариант (`<Show>` + `Signal::derive` на call-site) ронял bin. Фикс:
  guard внутрь компонента + `-> AnyView` (плоский тип на call-site).
  `cargo test -p origa_ui --bins` мог проходить, а `--workspace` падать —
  CI использует `--workspace --exclude origa_landing`, проверять надо
  ровно CI-команду.
- **ULID в тестовых фикстурах**: Crockford Base32 исключает I, L, O, U —
  `...EZ03HU` невалиден, заменил на `...EZ03J0` и др.
- **Диск**: `target/debug/incremental` распух до 28 ГБ и залил D: —
  LLVM падал с `no space on device`, маскируясь под ошибку компиляции.
- **cargo test --workspace с origa_landing** требует `ORIGA_APP_BASE_URL`,
  иначе build.rs паникует; вместе с tauri-unification может переваливать
  bin — CI обходит через `--exclude origa_landing`.
- **e2e: существующие шаги не самодостаточны.** «выбирает минимальную
  нагрузку» НЕ открывает /profile — в feature перед ним обязателен
  «пользователь открывает страницу профиля», иначе таймаут на
  `profile-load-minimal`.
- **e2e: `completeLessonFlexible` не переваривал vocab-quiz карточки**:
  quiz_card.rs скрывает опции после ответа (`Show when !show_result`),
  helper ждал anyInteractive перед проверкой NextCard → зависание на
  feedback-фазе. Фикс: проверка `lessonCardNextBtn` ДО `anyInteractive`
  (для phrase-quiz опции остаются видимыми — порядок критичен). Тест
  мог проходить/падать случайно: `apply_view` выбирает тип карточки
  вероятностно (show-answer vs quiz vs yesno) — один и тот же сценарий
  флейкил между прогонами.

## Верификация

- `cargo clippy --workspace --all-targets -- -D warnings` ✅
- `cargo test --workspace --exclude origa_landing` ✅ (CI-команда)
- `cargo fmt --all` ✅
- `npm run typecheck` ✅
- e2e: см. статус PR
