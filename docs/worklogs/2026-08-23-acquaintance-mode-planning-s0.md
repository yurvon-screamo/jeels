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

## Следующий шаг

Срез S1: полная доменная машина руки (витки, подфазы, критерии) + rstest.
