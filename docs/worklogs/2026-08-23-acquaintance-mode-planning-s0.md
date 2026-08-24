# Worklog: Режим знакомства с новыми картами — планирование + срез S0

- Дата: 2026-08-23
- Фича: новые карты попадают в SRS только через контролируемое знакомство
  (рука ≤7 карт: показ → тренировка ротациями до критерия → первый ревью
  назавтра). Поведение основано на successive relearning.
- Спека и план: [docs/acquaintance-mode.md](../acquaintance-mode.md).

## Что сделано

1. **Планирование** (без кода): ландшафт практик запоминания (successive
   relearning, expanding retrieval, criterion-based learning); поведенческая
   спека Gherkin v4.1 (12 правил, ~35 сценариев) — 2 раунда ревью до `ready`;
   UI-спека внутри страницы `/lesson` (слайды трёх типов, полоса руки,
   inline-confirm «Уже знаю», кнопки «Не помню»/«Помню»); техплан с контрактами
   (`AcquaintanceHand`, `AnswerOutcome`, `NewCardPolicy`) и срезами S0–S7 —
   3 раунда ревью до `ready`.
2. **S0**: доменный модуль `origa/src/domain/acquaintance/` (mod/seed/hand):
   - `seed_first_review(history, first_due)` — сидирование первого ревью через
     `MemoryState::with_card_state(Stability(3), Difficulty(5), due, Review)`
     без семантики ревью; новый `MemoryHistory::seed` (pub(crate));
   - каркас типов руки (`AcquaintanceSubphase`, `AnswerOutcome`,
     `AcquaintanceEntry`, `AcquaintanceHand` c геттерами);
   - feature `acquaintance_mode = []` в `origa_ui/Cargo.toml` (пока не
     потребляется — расцветёт в S4–S6);
   - 8 юнит-тестов: seed-инварианты (!is_new / due=завтра / !known /
     !high_difficulty), нетронутость журнала ревью, эволюция после Good
     (интервал ≥1 дня, ≤60 дней, остаётся Review).
3. Ревью кода: `approve` (1 Common — seed → pub(crate), исправлено; 2 Low —
   тест разделён по одному концепту, Result оставлен осознанно).

## Ключевые решения

- Тренировочные ответы НЕ пишут в MemoryHistory; планирование первого ревью —
  ровно один раз при закрытии руки (сидирование, не рейтинг).
- Рука атомарна и эфемерна: прерывание = отсутствие записи, отдельного use case
  нет.
- Флаг компиляции живёт только в origa_ui; секция features в crate origa не
  заводится; билдер получит параметр NewCardPolicy в S3 (не cfg).
- Экзаменационная фаза удалена из спеки (полная ротация делает её дубликатом),
  выбывание карт («тающая рука») отклонено — хвост даёт массированное
  повторение и иллюзию знания.

## Gotchas окружения

- Локальный `cargo test --workspace` на Windows/nightly падает в bin
  `origa_ui_bin` (`queries overflow the depth limit`): бин у потолка
  recursion_limit=128 (ADR-027/ADR-031), унификация фич leptos с SSR-landing
  усугубляет. Воспроизводится на чистом мастере (проверено git stash).
  Рабочие локальные ворота: `--exclude origa_landing -j 2` + отдельно
  `-p origa_landing`; CI Linux покрывает полный воркспейс. Зафиксировано в
  docs/acquaintance-mode.md §9.3.
- Диск переполнялся при линковке (target разросся до 81 GB): лечится
  `cargo clean -p <верхние крейсы>` (34 GB) или полным clean; после сбоя по
  диску инкрементальный кэш может остаться битым → CARGO_INCREMENTAL=0.

## Ворота

fmt ✅ · clippy workspace -D warnings ✅ · тесты: workspace(excl landing)
2107/0 + landing 58+3/0 + origa целиком 1662/0 (8 новых acquaintance-тестов) ✅

---

# Срез S1 (2026-08-23)

## Что сделано

1. **Машина руки** (`hand.rs` 167 строк + `entry.rs` 55 + `phase.rs` 30):
   - `AcquaintanceHand::new` — валидации (пустота/дубликаты/Phrase →
     `InvalidAcquaintanceHand`), стартовая подфаза Forward при наличии слов;
   - `record_answer(card_id, remembered) -> Result<AnswerOutcome>` —
     подсчёт до критерия 3, заморозка закрывших (по прогрессу текущей
     подфазы / общему для несловесных), провал = честный no-op исход,
     авто-смена Forward→Reverse когда все слова закрыли Forward,
     `HandCompleted` на последнем критерии;
   - два счётчика у слов (forward/reverse) вместо сброса одного —
     несловесные карты получают иммунитет от смены подфаз бесплатно.
2. **Ошибка**: `OrigaError::InvalidAcquaintanceHand { reason }` (Domain).
3. **Тесты**: 17 юнитов в трёх файлах (hand_tests/completion_tests/
   training_tests) — все пути правила «Тренировка»: подсчёт, провал,
   заморозка (слово в подфазе при незакрытых соседях; несловесная после
   критерия), смена подфаз со сбросом видимого прогресса, накопление
   неслова сквозь подфазы, завершение смешанной руки/руки без слов/
   вырожденной из одной карты, unknown id → CardNotFound.

## Отклонения от буквы плана (приняты ревьювером)

- `AnswerOutcome::Failed` добавлен (план не имел no-op исхода для провала).
- Заморозка по прогрессу текущей подфазы, полная завершимость — по обоим
  счётчикам слова.

## Ворота

fmt ✅ · clippy workspace -D warnings ✅ · workspace(excl landing)
2122/0 + landing 61/0 + origa 1678/0 ✅ · файлы ≤167 строк ✅
Ревью: `approve` (1 Common — два недостающих теста, доложены сразу).

## Следующий шаг

Срез S2: use cases SelectAcquaintanceHandUseCase +
CompleteAcquaintanceHandUseCase + journeys.

---

# Срезы S2–S4 (2026-08-23)

## S2 — use cases (f6a43db5)

SelectAcquaintanceHandUseCase (детерминированный JLPT-сорт через
JlptContent::find_level вместо reuse distribute_new_cards — rng ломал бы
контракт «та же рука»; принято ревьювером) + CompleteAcquaintanceHandUseCase
(сидирование due=завтра всем ещё новым картам руки + списание лимита одной
операцией, идемпотентный пропуск известных) + 10 journeys.
Ревью: approve; 3 Common закрыты (тесты группировки показа, journey на
идемпотентность смешанного завершения, jlpt_sort_key консолидация).

## S3 — инварианты билдера (db3e8195)

NewCardPolicy {Inject, Exclude}; Exclude гейтит впрыск + фильтрует
избранных незнакомых + drop_new_cards choke point после companions
(корректировка core_count); фразы освобождены предикатом (rstest 6 осей);
jlpt_sort_key консолидация; distribute_new_cards откат в private;
build_seeded_memory_state убран из pub API.
Ревью: approve c 0 High/0 Common; оба Low закрыты (единый источник фактов
в drop_new_cards + параметр политики).

## S4 — UI фазы показа (e7910303 → 6fc7e1d6)

AcquaintanceState/Context/SlideData; AcquaintanceView: Tag фазы +
HandProgressStrip; типозависимые слайды (word=FuriganaText+translations,
kanji=reuse KanjiCardDetails, grammar=все поля сразу с Show when
non-empty); action bar «Уже знаю» Ghost→inline-confirm→MarkKnown +
«Дальше» (скрыт при открытом confirm — анти-гонка); Training-заглушка до
S5; content.rs: hand-select до review-select, рендер двумя
взаимоисключающими ветками.

### Уроки S4 (важно!)

- Leptos: cfg-атрибуты внутри view! НЕ обрабатываются — все cfg-ветки
  делать statement'ами ДО view! (замыкания, возвращающие into_any).
- python-патчи с якорями после cargo fmt молча не применяются — каждый
  шаг верифицировать grep'ом сразу.
- git add -A захватил чужие untracked landing-файлы — исправлено
  reset --soft + restore --staged; впредь добавлять файлы явно.

## Ворота S4

fmt/clippy/workspace 2143/0 ✅ · обе конфигурации флага компилируются ✅
wasm-тесты: нерабочий файл удалён честно, переносится в S7 e2e вместе с
ask-first CI-правкой. Ревью раунд 2 в процессе (фоновая задача).

## Осталось

S5 тренировка (+извлечение оркестрации из LessonContent), S6 итоговый
экран, S7 e2e + CI ask-first правка.

---

# Срезы S5–S7 (2026-08-24)

## S5+S6 — тренировка и итог (8fad1d82)

TrainingBody: последовательная ротация presentation_order (шафл витка
отложен), фронты по типу×подфазе, рейтинг → AcquaintanceHand::record_answer,
HandCompleted → CompleteAcquaintanceHandUseCase → Summary. Полоса из
entry.progress_in(subphase); тег фазы с направлением. S6: штамп +
«К ревью» → stage=Inactive → обычный урок.

Уроки: код писать ТОЛЬКО после чтения фактических сигнатур домена
(record_answer -> Result<AnswerOutcome>, execute(Vec<Ulid>)) — попытка
«по памяти» дала три несобираемых файла подряд.

## S7 — e2e (f9916382)

acquaintance_flow.spec.ts за env-гейтом ACQUAINTANCE_MODE=1: happy path
и «Уже знаю». Дефолтный CI пропускает; включение прогона под флагом —
ask-first правка ci.yml (документировано в end2end/AGENTS.md).

## Осталось

Ревью S5–S7 (запущено), пуш коммитов, ручной smoke на реальной сборке.
