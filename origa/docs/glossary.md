# Глоссарий доменной модели Origa

Канонический словарь терминов предметной области. Кодовые имена (в квадратных скобках) — единственно допустимые идентификаторы в коде, тестах и документации. При появлении нового доменного понятия — добавьте запись сюда ДО написания кода.

---

## SRS — Интервальное повторение

Оценка [Rating] — ответ пользователя при повторении карточки. Четыре значения:

- Again [Rating::Again] — не вспомнил, сброс интервала.
- Hard [Rating::Hard] — вспомнил с трудом, интервал уменьшается.
- Good [Rating::Good] — нормальное вспоминание, интервал растёт штатно.
- Easy [Rating::Easy] — легко, интервал растёт ускоренно.

НЕ называть «grade», «score», «answer» — только `Rating`.

Стабильность [Stability] — прочность удержания карточки в памяти. Измеряется в днях: чем выше, тем длиннее интервал до следующего повторения. Value object: `Stability::new(f64)`, не может быть отрицательной. В коде: `domain::memory::Stability`.

Сложность [Difficulty] — насколько тяжело карточка даётся пользователю. Чем выше, тем короче интервал при той же оценке. Value object: `Difficulty::new(f64)`, отклоняет только отрицательные значения. FSRS-алгоритм внутри клемпит в [1.0, 10.0], но value-object этого не делает. В коде: `domain::memory::Difficulty`.

Состояние памяти [MemoryState] — снимок SRS-параметров карточки на момент последнего повторения: stability, difficulty, next_review_date, card_state. В коде: `domain::memory::MemoryState`.

Состояние карточки [CardState] — этап жизненного цикла карточки в FSRS:

- New [CardState::New] — карточка ни разу не показывалась.
- Learning [CardState::Learning] — начальное заучивание, короткие интервалы.
- Review [CardState::Review] — карточка в регулярном повторении. Значение по умолчанию (`#[default]`).
- Relearning [CardState::Relearning] — забыта (Again на Review), вернулась в заучивание.

История памяти [MemoryHistory] — последовательность всех повторений карточки: текущий MemoryState + список ReviewLog. Используется для merge между устройствами и расчёта SRS. В коде: `domain::memory::MemoryHistory`.

Лог повторения [ReviewLog] — запись одного повторения: rating + interval + timestamp + ULID. В коде: `domain::memory::ReviewLog`.

Режим оценки [RateMode] — контекст, в котором выставляется оценка. Определяет FSRS-параметры (retention, max_interval, enable_fuzz):

- StandardLesson [RateMode::StandardLesson] — обычный урок.
- ShortTerm [RateMode::ShortTerm] — краткосрочное заучивание (сериализуется как `"FixationLesson"` для backward-compat).
- PhraseReview [RateMode::PhraseReview] — повторение карточки-фразы.
- OnboardingScoring [RateMode::OnboardingScoring] — оценивание при онбординге.
- GrammarReview [RateMode::GrammarReview] — повторение грамматической карточки.
- KanjiReview [RateMode::KanjiReview] — повторение кандзи-карточки.

В коде: `domain::srs::RateMode`.

Следующее повторение [NextReview] — результат расчёта FSRS: интервал + новое MemoryState. Возвращается функцией `rate_memory(mode, rating, memory_history)`. В коде: `domain::srs::NextReview`.

Порог известности [KNOWN_CARD_STABILITY_THRESHOLD] — константа `21.0` (дней). Карточка считается изученной, если её stability выше этого порога. В коде: `domain::memory::KNOWN_CARD_STABILITY_THRESHOLD`.

Порог сложности [HIGH_DIFFICULTY_THRESHOLD] — константа `7.0`. Карточка считается сложной, если difficulty ≥ порога И stability < `HIGH_DIFFICULTY_STABILITY_CAP` (7.0).

---

## Карточки

Карточка [Card] — единица изучения. Enum с четырьмя вариантами:

- Vocabulary [Card::Vocabulary] — слово.
- Kanji [Card::Kanji] — иероглиф.
- Grammar [Card::Grammar] — грамматическое правило.
- Phrase [Card::Phrase] — фраза.

В коде: `domain::knowledge::Card`.

Карточка изучения [StudyCard] — обёртка над Card, добавляющая SRS-контекст: ULID, MemoryHistory, is_favorite, favorite_easy_streak, favorite_changed_at. Это то, что хранится в KnowledgeSet и сериализуется на диск. В коде: `domain::knowledge::StudyCard`.

Слово-карточка [VocabularyCard] — карточка для изучения слова. Содержит Question (само слово). В коде: `domain::knowledge::VocabularyCard`.

Кандзи-карточка [KanjiCard] — карточка для изучения иероглифа. В коде: `domain::knowledge::KanjiCard`.

Карточка грамматики [GrammarRuleCard] — ссылка на грамматическое правило по rule_id (ULID). Само правило хранится в CDN grammar-JSON. В коде: `domain::knowledge::GrammarRuleCard`.

Фраза-карточка [PhraseCard] — ссылка на фразу по phrase_id (ULID). Сама фраза хранится в phrase index. В коде: `domain::knowledge::PhraseCard`.

Тип карточки [CardType] — классификация карточки для статистики и построения урока. Выводится из Card enum. В коде: `domain::knowledge::CardType`.

Вопрос [Question] — лицевая сторона карточки (текст, который видит пользователь). Value object: не может быть пустым после trim. В коде: `domain::value_objects::Question`.

Ответ [CardAnswer] — оборотная сторона карточки. Варианты: Text (простой текст), Vocabulary (список переводов + описание). В коде: `domain::value_objects::CardAnswer`.

Избранное [is_favorite] — флаг StudyCard. Карточка в избранном появляется в уроке чаще. Снимается автоматически после 5 оценок Easy подряд (`favorite_easy_streak`).

---

## Урок

Урок [LessonData] — набор карточек для одного занятия. Содержит: cards (Vec<(Ulid, LessonCard)>), core_count, daily_new_limit. Строится функцией `KnowledgeSet::cards_to_lesson()`. В коде: `domain::knowledge::LessonData`.

Карточка урока [LessonCard] — отображение Card в контексте урока: card_id, сам Card, quiz (опционально). В коде: `domain::knowledge::LessonCard`.

Представление карточки [LessonCardView] — view-слой для UI: Normal (простая карточка), YesNo (да/нет Quiz), Quiz (множественный выбор). Генерируется `LessonViewGenerator`. В коде: `domain::knowledge::LessonCardView`.

Генератор представлений [LessonViewGenerator] — преобразует LessonCard в LessonCardView, добавляя quiz/distractors на основе словарей. В коде: `domain::knowledge::LessonViewGenerator`.

Викторина [QuizCard] — карточка с вопросом и вариантами ответа. В коде: `domain::knowledge::QuizCard`.

Да/Нет карточка [YesNoCard] — утверждение, на которое пользователь отвечает да/нет. Генерируется из Vocabulary карточек с использованием distractors. В коде: `domain::knowledge::YesNoCard`.

---

## Набор знаний

Набор знаний [KnowledgeSet] — коллекция всех карточек пользователя: study_cards (HashMap<Ulid, StudyCard>), deleted_cards (tombstones), deleted_companion_words (blocklist), stats (StatsTracker). Сериализуется как часть User. В коде: `domain::knowledge::KnowledgeSet`.

Tombstone [deleted_cards] — множество ULID удалённых карточек. Используется при merge между устройствами: карточка, удалённая на одном устройстве, не восстанавливается с другого.

Blocklist companion-слов [deleted_companion_words] — множество слов (поверхностных форм), удалённых пользователем. Запрещает автоматическое повторное создание companion-карточек для кандзи. Ручное создание через `create_card` очищает запись из blocklist'а.

Карточки-компаньоны [companion vocab cards] — словарные карточки, автоматически создаваемые для кандзи (популярные слова, содержащие этот иероглиф). Максимум `MAX_COMPANION_WORDS` (3) на кандзи; слова из removal-списка аудита словаря не создаются никогда (guard в `KnowledgeSet::create_companion_vocab_cards`).

---

## Статистика

История по дням [DailyHistoryItem] — снимок статистики за один день: lessons_completed, total/known/new/in_progress/high_difficulty words, positive/negative/total ratings, new/phrase cards studied, avg stability/difficulty. В коде: `domain::knowledge::DailyHistoryItem`.

Обновление статистики [DailyStatsUpdate] — входной параметр для методов `update()`/`update_stats()` класса DailyHistoryItem. Передаётся из StatsTracker при каждом изменении. В коде: `domain::knowledge::daily_history::DailyStatsUpdate` (standalone struct, не поле DailyHistoryItem).

Оценка даты завершения [estimate_completion_date] — функция, прогнозирующая дату изучения всех карточек на основе истории новых карточек за день. В коде: `domain::knowledge::estimate_completion_date`.

Сводка за сегодня [TodayOverview] — агрегированная статистика за текущий день для UI. В коде: `domain::stats::TodayOverview`.

Доля оценок [RatingRatio] — отношение положительных/отрицательных оценок. В коде: `domain::stats::RatingRatio`.

Трекер статистики [StatsTracker] — внутреннее поле KnowledgeSet, отвечающее за daily_history и счётчики new_cards_studied_today/phrase_cards_studied_today. В коде: `domain::knowledge::StatsTracker`.

---

## Японский язык

Уровень JLPT [JapaneseLevel] — уровень экзамена JLPT: N5 (начальный) → N1 (продвинутый). Используется для приоритизации новых карточек в уроке. В коде: `domain::value_objects::JapaneseLevel`.

Родной язык [NativeLanguage] — язык интерфейса/переводов пользователя: English или Russian. Влияет на выбор переводов в карточках и фразах. В коде: `domain::value_objects::NativeLanguage`.

Фуригана [FuriganaSegment] — сегмент текста с аннотацией чтения (хирагана над кандзи). В коде: `domain::furigana::FuriganaSegment`. Функции: `furiganize_text`, `furiganize_text_html`.

Часть речи [PartOfSpeech] — грамматическая классификация токена (существительное, глагол, и т.д.). Используется при токенизации и генерации quiz. В коде: `domain::tokenizer::PartOfSpeech`.

Токен [TokenInfo] — результат токенизации японского текста: поверхностная форма + часть речи + базовая форма. В коде: `domain::tokenizer::TokenInfo`.

Перевод токена [TokenTranslation] — перевод токена на родной язык пользователя. В коде: `domain::tokenizer::TokenTranslation`.

Словарные данные [DictionaryData] — предзагруженный lindera-словарь + rkyv-сериализованные переводы. Глобальный singleton, инициализируется через `init_dictionary` или `init_dictionary_from_rkyv`.

---

## Пользователь

Пользователь [User] — агрегат верхнего уровня: email + native_language + knowledge_set + (опционально) jlpt_progress. Сериализуется и хранится на сервере (TrailBase). В коде: `domain::user::User`.

Знание слова [WordKnowledge] — оценка пользователем своего знания конкретного слова при онбординге. В коде: `domain::user::WordKnowledge`.

Нагрузка [DailyLoad] — предпочтение пользователя по количеству новых карточек в день: Minimal (7) → Maximum (42). Значения кратны размеру руки знакомства (7): темп задаёт число полных рук в день, хвостовых «обрубков» лимита не бывает. В коде: `domain::value_objects::DailyLoad`.

Дневной бюджет [DailyBudget] — производные от Нагрузки лимиты одного урока: сколько новых карточек допускается в день (`new_cards_per_day`, кратно 7) и сколько новых якорных фраз допускается в один урок (`new_phrases_per_lesson` = 2 × new_cards_per_day). Остаток дневного лимита гейтит только открытие руки знакомства, не её размер. Бюджет фраз привязан к уроку, а не к суткам: каждый урок получает полный лимит. В коде: `domain::value_objects::DailyBudget`.

Рука [AcquaintanceHand] — партия новых карт (до 7), проводимая через показ и тренировку до критерия; атомарна и эфемерна (состояние не персистится). Всегда набирается полностью, пока в пуле есть карты; кандзи руки тянет в ту же руку до 3 новых слов-компаньонов со своим знаком (кластер). В коде: `domain::acquaintance::AcquaintanceHand`, `use_cases::SelectAcquaintanceHandUseCase`.

---

## JLPT

Контент JLPT [JlptContent] — слова, кандзи и грамматика, сгруппированные по уровням JLPT (N5–N1). Загружается из CDN. Используется для приоритизации карточек в уроке и прогресса. В коде: `domain::jlpt_content::JlptContent`.

Прогресс JLPT [JlptProgress] — сколько слов/кандзи/грамматики каждого уровня JLPT изучено пользователем. В коде: `domain::jlpt_progress::JlptProgress`.

---

## Известные наборы

Известный набор [WellKnownSet] — предсобранный набор слов/кандзи для учебника или медиа (Minna no Nihongo, Spy Family, и т.д.). Хранится в CDN. В коде: `domain::well_known_set::WellKnownSet`.

Метаданные набора [WellKnownSetMeta] — id, set_type, level, title (RU/EN), description, word_count. В коде: `domain::well_known_set::WellKnownSetMeta`.

Тип набора [SetType] — классификатор набора (MinnaNoNihongo, SpyFamily, Custom, ...). Type alias на `String`, не enum — значения не типизированы и открыты. В коде: `domain::well_known_set::SetType`.
