# ADR-039: Migrate the tokenizer to SudachiDict on lindera 5

- Дата: 2026-08-18
- Статус: Accepted (реализовано в ветке `feat/sudachidict-lindera5`)

## Контекст

Токенайзер Origa работал на lindera 2.3.4 + UniDic 2.1.2. UniDic — словарь
«коротких единиц» (短単位): у составных слов нет собственных лемм, и Viterbi
резал их на Noun + Suffix. `PartOfSpeech::Suffix` не входит в
`is_vocabulary_word()`, поэтому хвост сплита молча выбрасывался при
создании карточек — пользователь видел только лидирующий фрагмент
(自動車 → 自動). Пострадали и well-known сеты (исторические «чистки
токенайзером» вырезали ~10% слов из migii), и фразовый индекс.

Дополнительный слой — предсобранный rkyv-кеш (`cached-lindera.bin`, 213 МБ
на CDN) — существовал ради lindera 2.x, где загрузка словаря из сырых байт
была дорогой (десериализация daachorse-автомата, transpose матрицы).

## Решение

1. **Словарь: SudachiDict 20260723 (small + core)**, собранный
   `scripts/build_sudachidict.py` (рецепт lindera `docs/src/sudachidict.md`):
   - 46 дополнительных строк в core_lex (連体詞-составные, STT-катакана,
     сленг-орфографии) с реальными контекстами соединения;
   - OOV-плейсхолдеры Sudachi (left=-1/right=-1) переписаны нормальными
     рядами;
   - unk.def выровнен display-колонкой под системную схему.
   На эталоне 11 829 слов well-known сетов: 270 побед / 10 потерь против
   UniDic. Принятые регрессии: числительные-композиты (二十 → 二|十).
2. **lindera 5.3.0**: `dict.trie` ходится in-place, матрица читается
   in-place — дорогая загрузка исчезла, rkyv-кеш-слой удалён целиком
   (213 МБ блоба → 75 МБ deflate на CDN, клиент инфлейтит и кеширует
   per-file в Cache API).
3. **Runtime user dictionary удалён**: extra-лексика запекается в системный
   словарь при сборке. User-dict держал вторую POS-схему — `Token::get`
   разрешал `part_of_speech` по индексу системной схемы, и все user-слова
   получали Unspecified POS (невидимы для карточек).
4. **Версионированный CDN-путь** `dictionaries/sudachidict-20260723/`:
   legacy-файлы UniDic остаются на S3 байт-в-байт (хэши манифеста не
   меняются), старые клиенты не замечают деплой. Удаление legacy —
   отдельный релиз после вымирания старых клиентов.
5. **Пользовательская миграция** `MigrateVocabularyLemmasUseCase`: карточки
   со старыми написаниями лемм (信ずる → 信じる) переименовываются на
   старте с сохранением FSRS-памяти; мульти-сплиты не трогаются.
6. **Данные переиндексированы** тем же пайплайном: сеты —
   `tokenize-well-known`, фразы — `enrich-phrases-with-grammar` (v17)
   - чистка пустых записей; `find-missing` — 0 отсутствующих.

## Последствия

- Токенайзер держит составные слова цельными (自動車, お父さん, 令和 …).
- Пик WASM-памяти при первом старте снижен порядком инфлята (крупнейший
  файл разворачивается первым, буфер преаллоцирован).
- Стартовый кеш словаря в Cache API: 75 МБ вместо 213 МБ.
- Поддержка двух раскладок словаря на CDN до удаления legacy.

## Post-migration cleanup (issue #409)

После вымирания клиентов на pre-SudachiDict сборках (Sentry: ноль событий
с релизов ≤0.6.4 после 2026-08-30; первый стабильный SudachiDict-релиз —
v0.6.5) временные слои удаляются cleanup-PR'ом по issue #409:

- legacy-файлы UniDic исключены из деплоя и CDN-манифеста (JmdictFurigana.txt
  остаётся — используется); само S3-удаление объектов выполняется отдельным
  runbook'ом после мержа (бэкап по SHA256 remote-манифеста, deploy_cdn.py,
  refresh_cache_control.py --dry-run); flat-layout fallback убран из utils
  и тестовых фикстур;
- имя каталога `sudachidict-20260723` вынесено в
  `origa::domain::tokenizer::SUDACHIDICT_DIR` (зеркала: origa_ui/build.rs,
  scripts/deploy_cdn.py);
- `MigrateVocabularyLemmasUseCase` и `MigrateVocabularyPartOfSpeechUseCase`
  ретайрены вместе с `VocabularyCard::with_word()/with_pos()` — карточки
  создаются уже с POS, а `pos=None` легально живёт с ленивой ретокенизацией;
- пустой `origa/build.rs` удалён вместе с мёртвыми `[build-dependencies]`;
- вычищены мёртвые UniDic-кэши CI (cache-степы, keep-правило, gitignore,
  Dockerfile) и починена проверка словаря в `origa_ui/build.rs`, которая
  после миграции смотрела в flat-раскладку без lindera 5 файлов.
