# ADR-019: Многократный показ карточек урока по FSRS-статусу и кэш части речи

## Status

Accepted

## Date

2026-06-28

## Context

Процесс обучения был слишком поверхностным: каждая карточка показывалась в уроке один раз. Трудные карты (`is_high_difficulty`) получали тот же единственный показ, что и хорошо изученные, хотя именно они нуждаются в расширенном повторении (expanding rehearsal) внутри сессии. Также на карточке слова не было части речи — пользователь не видел, существительное это или глагол.

### Проблемы

1. Трудные и in-progress карты не получали внутри-сессионного закрепления.
2. Контракт `LessonData` (ключ `Vec<(Ulid, LessonCard)>` = slot/card id) делал невозможным повторный показ одной карты: `LessonData::cards_map`/`get` и фронтовый `cards: HashMap<Ulid, LessonCard>` в `lesson_state` предполагают уникальность ключа-слота.
3. POS вычислялся через lindera-токенизацию на лету при каждом обращении (`VocabularyCard::part_of_speech() -> Result<PartOfSpeech, OrigaError>`) — дорого и нестабильно.

## Decision

### 1. Slot id vs card id в LessonCard

Введено поле `LessonCard.card_id` (настоящий StudyCard id для FSRS-rate; `#[serde(default = "Ulid::nil")]`). Ключ `LessonData.cards: Vec<(Ulid, LessonCard)>` теперь трактуется как slot id (уникальный per показ):

- single-show: slot id == card id (как в `build_core_lesson_cards`);
- multi-show: slot id = `Ulid::new()`, общий `card_id`.

Backward-compat: кастомный `Deserialize` для `LessonData` через локальную `Wire`-структуру с backfill `card_id` из slot-ключа для старых данных (nil → slot id). Roundtrip-тест `lesson_data_roundtrip_preserves_all_fields` пинирует сохранение всех полей; `lesson_data_deserialize_backfills_nil_card_id_from_slot` — ветку backfill.

### 2. expand_repeated_views в конце pipeline

Шаг `expand_repeated_views` выполняется последним в `KnowledgeSet::cards_to_lesson`, после всех этапов сборки:

`build_lesson_core → add_kanji_companions → add_interleaved_phrases → add_tail_phrases → expand_repeated_views`

Все upstream-шаги отрабатывают по slot id без изменений. Умножаются только primary non-phrase слоты: `build_lesson_core` возвращает `primary_card_ids` (favorites + selected + padding), в который НЕ попадают companions (они добавляются позже через `add_kanji_companions`) → companions exempt (всегда 1 показ), иначе размер урока взрывался бы.

### 3. Цели показов по статусу + clamp

Функция `target_showings`:

- `is_high_difficulty` → 2;
- иначе (new/in-progress/known) → 1.

Применяется к Vocabulary/Kanji/Grammar; Phrase не умножается (`candidate_views_for_repeat` для Phrase возвращает пустой вектор). «Разные вью» = строго разные дискриминанты `LessonCardView` (дедуп через `std::mem::discriminant` в `build_distinct_views`). `compute_expansion_views` обрезает кандидатов до цели и возвращает пустой вектор, если доступных различных вью меньше 2 (в этом случае показ остаётся единичным). Если доступных различных типов (с учётом guards) меньше цели → clamp к числу доступных (НЕ fallback к 1). Каждый показ рейтится отдельно по `card_id` → N оценок FSRS.

Матрица фактически достигаемых показов: Vocabulary hard → 2, Kanji hard → 2, Grammar hard → 2; Vocabulary/Kanji/Grammar new/in-progress/known → 1 (Grammar new также 1 — только `Normal`).

> **Изм. от 2026-08-04:** ранее high-difficulty имел target=3 (Kanji hard достигал 3 показов); снижено до 2 — уроки получались слишком длинными. В той же правке new/in-progress временно остались на target=2; позже снижены до 1 — повторный показ нужен только трудным картам, новые и в стадии закрепления достаточно одного показа за урок.

### 4. POS кэш + миграция

`VocabularyCard.pos: Option<PartOfSpeech>` (`#[serde(default)]`) — вычисляется один раз при создании карты (`from_known_word`/`from_text`). `part_of_speech()` читает из кэша с fallback на `tokenize_text` для legacy; `pos()` возвращает `Option<PartOfSpeech>` без токенизации. POS пробрасывается в `revert()` и `with_grammar_rule()` — мутированная/развёрнутая форма сохраняет POS исходного слова.

In-place миграция `MigrateVocabularyPartOfSpeechUseCase`: идемпотентна, перебирает vocabulary-карты с `pos().is_none()`, проставляет POS через `with_pos` и `update_card_content` → `replace_card` (замена только поля `card`, `MemoryHistory` сохраняется). Early-return без `save_sync`, когда `migrated == 0` (steady-state, чтобы не делать блокирующую запись на каждом холодном старте).

### 5. Spacing best-effort

`MIN_REPEAT_SPACING = 3` карты между последовательными показами одного `card_id`. Контракт: spacing гарантируется (`drain_pending`/`distribute_pending_with_spacing`), когда размер core-секции позволяет; на слишком коротком уроке (anchor в конце core, нет буфера) целевой индекс клемпится к `core.len()` и копии кластеризуются в конце core — best-effort. Tail-фразы остаются в самом конце урока (копии вставляются строго внутри core-секции). Контракт документирован и покрыт edge-case rstest-тестами.

### 6. POS Tag UI

Локализованный тег POS (RU/EN, все варианты `PartOfSpeech`) рядом с тегом типа карты во всех вью слова (`LessonCardHeader`). POS читается через `Card::vocabulary_part_of_speech()` и рендерится через `part_of_speech_label`. Тег использует `TagVariant::Default` (без явного variant → default) — Tertiary tier по DESIGN.md для вторичных метаданных; цветные variants (`Olive`/`Terracotta`/`Sage`/`Filled`) зарезервированы для card types (`CardType::tag_variant`).

## Alternatives Considered

### Ручная пользовательская метка «сложная»

Отвергнуто: FSRS-производные статусы (`is_high_difficulty`/`is_in_progress`/`is_known_card`) уже автоматически отражают сложность; ручная метка требует миграции данных, новой UI-ручки и дублирует семантику FSRS.

### Fallback к 1 показу при нехватке типов вью

Отвергнуто: лишило бы фичу самую ценную часть — трудные грамматика/слова остались бы с 1 показом. Clamp сохраняет усиление.

### Хранить только slot id, rate по позиции

Отвергнуто: ломает rate-flow (rate идёт по `card_id`) и проверку дедупликации в downstream-шагах (`already_in_lesson` в `add_kanji_companions`, `in_lesson` в `add_interleaved_phrases`).

### Soft-cap размера урока

Отвергнуто пользователем: потолок не вводится; защита от взрыва — companions exempt (не умножаются) и clamp по доступным вью.

## Consequences

- Позитивные: трудные карты закрепляются внутри сессии; POS виден сразу, без токенизации в рантайме урока.
- Backward-compat: старые данные десериализуются без потерь (backfill `card_id` из slot id; `pos` по умолчанию `None`).
- FSRS: каждый показ — отдельная оценка; stability сдвигается N раз (по контракту).
- Размер урока не ограничен сверху (по решению пользователя); защита от взрыва — companions exempt.
- Поддержка: при добавлении поля в `LessonData` — обновить `Wire`-структуру (есть guard-тест `lesson_data_roundtrip_preserves_all_fields`).
