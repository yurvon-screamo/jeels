# Точки замены — точные списки к content-plan v3

Сверено grep'ом по рабочей копии 2026-08-19. Перед переносом в PR перепроверить grep'ом «200» / «перенос» / «transfer» / «tiến độ» / «이전» — нумерация строк могла сдвинуться.

## Раздел 1. Счётчик фраз: 200 000+ → 156 000+

Точное число: 156 228 (`cdn/phrases/phrase_index.json`, поле total). Формат — по канону локали: RU «156 000+», EN «156,000+» и «156K+» (где было «200K+»), KO «15만+»/«15만 개 이상의» (где было «20만»), VI «hơn 156.000»/«156K+» (где было «200K+»).

Лендинг (src/content):

1. ru.rs:34 — home_meta_description — «200 000+ фраз» → «156 000+ фраз»
2. ru.rs:37 — home_hero_subtitle — «более 200 000 фраз» → «более 156 000 фраз»
3. ru.rs:63 — features_meta_description — «200 000+ фраз» → «156 000+ фраз»
4. en.rs:34 — home_meta_description — «200K+ native phrases» → «156K+ native phrases»
5. en.rs:37 — home_hero_subtitle — «200,000+ native phrases» → «156,000+ native phrases»
6. en.rs:42 — home_problem_text — «200,000+ phrases» → «156,000+ phrases»
7. en.rs:63 — features_meta_description — «200K+ phrases» → «156K+ phrases»
8. ko.rs:34 — home_meta_description — «20만+ 원어민 문장» → «15만+ 원어민 문장»
9. ko.rs:37 — home_hero_subtitle — «20만 개 이상의» → «15만 개 이상의»
10. ko.rs:63 — features_meta_description — «20만 개 이상의» → «15만 개 이상의»
11. vi.rs:34 — home_meta_description — «200K+ câu bản ngữ» → «156K+ câu bản ngữ»
12. vi.rs:37 — home_hero_subtitle — «hơn 200.000 câu» → «hơn 156.000 câu»
13. vi.rs:63 — features_meta_description — «hơn 200.000 câu» → «hơn 156.000 câu»

README:

1. README.md:57 — «200,000+ phrases» → «156,000+ phrases»
2. README.ru.md:61 — «более 200 000 фраз» → «более 156 000 фраз»

Блог (best-japanese-learning-app.md, строка 58 во всех локалях):

1. blog/ru/best-japanese-learning-app.md:58 — «более 200 000 фраз» → «более 156 000 фраз»
2. blog/en/best-japanese-learning-app.md:58 — «more than 200,000 phrases» → «more than 156,000 phrases»
3. blog/vi/best-japanese-learning-app.md:58 — «hơn 200.000 cụm từ» → «hơn 156.000 cụm từ»
4. blog/ko/best-japanese-learning-app.md:58 — «20만 개 이상의 구문» → «15만 개 이상의 구문»

НЕ трогать: blog/*/japanese-ai-tutor.md:53,83 — там «$200/год» про цены конкурентов, не про фразы. blog/ko/best-japanese-learning-app.md:33,35 — «2003/2023» и прочие не-счётчики; сверять контекст каждой правки.

## Раздел 2. Н1: «история повторений переносится» — ложь (Anki notes-only)

Код: `origa/src/use_cases/import_anki_pack.rs:95,101,184` — читаются только поля notes; revlog не открывается; карточки создаются `VocabularyCard::from_text` как новые.

Точки (фраза — по локали):

1. blog/ru/best-japanese-learning-app.md:36 — «прогресс переносится»
2. blog/ru/best-japanese-learning-app.md:108 — «колоды и история повторений переносятся»
3. blog/ru/anki-alternative-japanese.md:116 — «существующие колоды и история повторений переносятся»
4. blog/ru/learn-japanese-from-manga.md:71 — «миграция односторонняя и без потерь»
5. blog/en/best-japanese-learning-app.md:36 — «progress transfers»
6. blog/en/best-japanese-learning-app.md:108 — «decks and review history transfer»
7. blog/en/anki-alternative-japanese.md:116 — аналог (проверить grep'ом «review history»)
8. blog/en/learn-japanese-from-manga.md — grep «without loss» (в RU-версии:71 есть; EN проверить)
9. blog/vi/best-japanese-learning-app.md:36 — «tiến độ được chuyển sang»
10. blog/vi/best-japanese-learning-app.md:108 — «bộ bài và lịch sử ôn tập được chuyển»
11. blog/vi/anki-alternative-japanese.md:116 — «bộ bài và lịch sử ôn tập được chuyển»
12. blog/ko/best-japanese-learning-app.md:36 — «진행이 이전됩니다»
13. blog/ko/best-japanese-learning-app.md:108 — «덱과 복습 이력이 이전됩니다»
14. blog/ko/anki-alternative-japanese.md:116 — «기존 덱과 복습 이력이 이전됩니다»

Замена — см. content-plan P1-7 (Н1). RU-шаблон:

> Можно импортировать готовые колоды Anki (`.anki2`, `.anki21`, `.anki21b`) — переносится список слов, Origa создаёт карточки заново. История повторений и интервалы не переносятся: карточки начнутся как новые. Для колод с многолетней историей это заметная потеря — учитывайте это при миграции.

## Раздел 3. Н3: «предложение, в котором слово встретилось» — ложь (нет поля предложения)

Код: `origa/src/domain/knowledge/vocabulary.rs:16` — у VocabularyCard поля предложения/контекста нет.

Подтверждённые точки:

1. blog/ru/japanese-ocr-app.md:61 — «каждое становится карточкой с чтением, переводом, аудио и предложением, в котором оно встретилось»
2. blog/ru/japanese-ocr-app.md:79 — аналог (проверить grep'ом «предложени»)
3. blog/ko/japanese-ocr-app.md:61 — «등장한 문장과 함께 플래시카드가 됩니다» (найдено в этой сессии)
4. blog/en/japanese-ocr-app.md — grep «sentence» (проверить)
5. blog/vi/japanese-ocr-app.md — grep «câu» (проверить)
6. learn-japanese-from-manga (RU/EN/VI/KO) — grep «предложени»/«sentence»/«câu»/«문장» — проверить, описывает ли фраза Origa или общий совет

Замена (RU): «каждое становится карточкой с чтением, переводом и аудио. Изображение не сохраняется.»

НЕ трогать: blog/ru/learn-japanese-from-manga.md:50 — «Добавьте каждое в SRS с предложением, в котором оно встретилось» — общий совет по майнингу (совет автору процесса), не утверждение о продукте.

## Раздел 4. Н2: «FSRS настраивается по колодам» — ложь (retention захардкожен)

Код: `origa/src/domain/srs.rs:36-65` — request_retention по RateMode: ShortTerm 0.95, StandardLesson/OnboardingScoring 0.85, PhraseReview 0.70, GrammarReview 0.90, KanjiReview 0.85; пользовательской настройки нет.

Точка: blog/{ru,en,vi,ko}/anki-alternative-japanese.md — таблица «Как Origa справляется» / «How Origa handles it», строка про FSRS. Замена — см. content-plan P1-7 (Н2).
