use crate::repository::HybridUserRepository;
use leptos::{prelude::*, task::spawn_local};
use origa::domain::{AcquaintanceHand, AcquaintanceSubphase, NativeLanguage};
use origa::use_cases::CompleteAcquaintanceHandUseCase;
use std::collections::HashSet;
use ulid::Ulid;

use super::kanji_card_details::RadicalDisplay;
use crate::ui_components::ReadingItem;

/// Стадии руки знакомства на странице урока (docs/acquaintance-mode.md):
/// показ → тренировка → переходный экран → обычное ревью. `Inactive` —
/// руки нет (пул пуст / лимит исчерпан / юзер продолжил к ревью).
/// `Completed` — тренировка закрыта: одноэкранный переход «теперь к
/// повторению» (H-итерация: без него смена контекста непонятна юзеру).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcquaintanceStage {
    #[default]
    Inactive,
    Presentation,
    Training,
    Completed,
}

/// UI-состояние руки: доменная машина (`AcquaintanceHand`) хранит правила,
/// этот тип — только позицию юзера внутри потока и служебные флаги.
#[derive(Clone, Default)]
pub struct AcquaintanceState {
    pub stage: AcquaintanceStage,
    pub hand: Option<AcquaintanceHand>,
    pub slide_index: usize,
    pub skipped_ids: HashSet<Ulid>,
    /// Рука закрывается: персистенция идёт, экран завершения ещё не
    /// смонтирован (stage станет Completed после коммита записи — защита
    /// бага #462). UI в этом окне замораживает отвеченную карту и прячет
    /// кнопки: без флага после финального ответа пере-показывался вопрос
    /// «лишней картой», которую затем резко перекрывал экран завершения.
    pub hand_finishing: bool,
}

/// Автозвук слова (механизм урока, lesson_card.rs): звучит, когда TTS
/// доступен и звук урока не выключен. Единый предикат для показа и
/// Forward-фронта тренировки — гарды не дублируются.
pub fn should_autoplay_word_audio(is_muted: bool, speech_supported: bool) -> bool {
    speech_supported && !is_muted
}

/// Видимость кнопки озвучки в шапке руки: кнопка озвучивает японскую
/// сторону слова — она доступна, только когда JP на экране. Reverse-фронт
/// показывает перевод (JP скрыта) — кнопка спрятана, чтобы не подсказывать
/// ответ голосом. Несловесные карты озвучивать нечем.
pub fn audio_button_visible(
    stage: AcquaintanceStage,
    subphase: Option<AcquaintanceSubphase>,
    showing_answer: bool,
    is_word: bool,
) -> bool {
    if !is_word {
        return false;
    }
    match stage {
        AcquaintanceStage::Presentation => true,
        AcquaintanceStage::Training => {
            subphase != Some(AcquaintanceSubphase::Reverse) || showing_answer
        },
        AcquaintanceStage::Completed | AcquaintanceStage::Inactive => false,
    }
}

impl AcquaintanceState {
    /// Ре-инициализация под новую руку: reload урока переиспользует тот же
    /// state (компонент не размонтируется), поэтому сбрасывать надо КАЖДОЕ
    /// поле потока. Несброшенный `hand_finishing` прошлой руки ослеплял
    /// вторую руку сессии — кнопки показа/тренировки и клавиатура
    /// гейтились навсегда (ревью PR #498).
    pub fn start_new_hand(&mut self, hand: AcquaintanceHand) {
        self.stage = AcquaintanceStage::Presentation;
        self.hand = Some(hand);
        self.slide_index = 0;
        self.skipped_ids.clear();
        self.hand_finishing = false;
    }

    /// Переходит к следующей непомеченной «Уже знаю» карте.
    /// Возвращает `true`, если показ исчерпан (пора в следующую стадию).
    pub fn advance_presentation(&mut self) -> bool {
        let Some(hand) = &self.hand else {
            return true;
        };
        let order = hand.presentation_order();
        loop {
            self.slide_index += 1;
            if self.slide_index >= order.len() {
                return true;
            }
            if !self.skipped_ids.contains(&order[self.slide_index]) {
                return false;
            }
        }
    }
}

/// Данные слайда показа, разрешённые из StudyCard до рендера
/// (индекс вектора совпадает с `presentation_order` руки).
#[derive(Clone)]
pub enum AcquaintanceSlideData {
    Vocabulary {
        card_id: Ulid,
        word: String,
        pos_label: Option<String>,
        translations: Vec<String>,
    },
    Kanji {
        card_id: Ulid,
        kanji: String,
        name: String,
        radicals: Option<Vec<RadicalDisplay>>,
        example_words: Option<Vec<(String, String)>>,
        on_readings: Option<Vec<ReadingItem>>,
        kun_readings: Option<Vec<ReadingItem>>,
    },
    Grammar {
        card_id: Ulid,
        title: String,
        short_description: String,
        how_to_form: String,
        examples: String,
        explanation: String,
        nuances: String,
    },
}

impl AcquaintanceSlideData {
    pub fn card_id(&self) -> Ulid {
        match self {
            Self::Vocabulary { card_id, .. }
            | Self::Kanji { card_id, .. }
            | Self::Grammar { card_id, .. } => *card_id,
        }
    }

    /// Японское слово слайда (для озвучки из шапки); несловесные карты —
    /// `None`.
    pub fn word(&self) -> Option<&str> {
        match self {
            Self::Vocabulary { word, .. } => Some(word),
            _ => None,
        }
    }

    /// Часть речи слайда слова — тег в шапке.
    pub fn pos_label(&self) -> Option<&str> {
        match self {
            Self::Vocabulary { pos_label, .. } => pos_label.as_deref(),
            _ => None,
        }
    }
}

/// Контекст префазы урока.
#[derive(Clone)]
pub struct AcquaintanceContext {
    pub repository: HybridUserRepository,
    pub state: RwSignal<AcquaintanceState>,
    pub slides: RwSignal<Vec<AcquaintanceSlideData>>,
    pub known_kanji: RwSignal<HashSet<char>>,
    pub native_language: RwSignal<NativeLanguage>,
    /// Текущая карта тренировки — отдельный сигнал: шапка читает его для
    /// тега типа карты, а запись при монтаже TrainingBody не перезапускает
    /// родительские Show (state.update зацикливала бы их перемонтирование).
    pub current_card: RwSignal<Option<Ulid>>,
    /// Раскрыт ли ответ тренировки — шапке нужен для видимости кнопки
    /// озвучки (Reverse-фронт прячет JP-сторону). Отдельный сигнал:
    /// шапка и TrainingBody делят его без чтений общего state, а дерево
    /// префазы стабильно (Memo-гейт content.rs не перемонтирует его).
    pub showing_answer: RwSignal<bool>,
}

impl AcquaintanceContext {
    /// Завершение руки: сидирование первого ревью назавтра + списание лимита
    /// одной операцией (S2), затем сразу ревью урока — без итогового
    /// экрана. Вызывается из конца тренировки (`HandCompleted`) и из
    /// показа, когда выведены все карты.
    ///
    /// Персистенция выполняется ДО перевода стадии в `Completed`: кнопка
    /// «К повторению» и её Space-хендлер существуют только после того, как
    /// запись закоммитилась, поэтому навигация или перезагрузка страницы
    /// физически не могут обогнать сейв (баг #462: сидирование терялось,
    /// когда тест/пользователь перезагружал страницу сразу после
    /// завершения руки — fire-and-forget `spawn_local` не успевал).
    pub fn complete_hand(&self) {
        // Флаг финиша — синхронно, до персистенции: UI сразу замораживает
        // отвеченную карту и прячет кнопки (см. hand_finishing), не дожидаясь
        // коммита записи и монтирования экрана завершения.
        self.state.update(|state| state.hand_finishing = true);

        let ids = self.state.with(|state| {
            state
                .hand
                .as_ref()
                .map(|h| h.presentation_order())
                .unwrap_or_default()
        });
        // Карты руки входят в «Пройдено» урока: экран завершения
        // показывает руку + ревью, а не 0 после урока из одной руки
        // (юзер-репорт). В wasm-тестах AcquaintanceView монтируется
        // без LessonContext — начисление просто пропускается.
        if let Some(lesson_ctx) = use_context::<super::lesson_state::LessonContext>() {
            lesson_ctx
                .lesson_state
                .update(|state| state.review_count += ids.len());
        }
        let repo = self.repository.clone();
        let state = self.state;
        spawn_local(async move {
            // Сидирование выполняет CompleteAcquaintanceHandUseCase (S2):
            // первый ревью назавтра всем картам руки + лимит одной операцией.
            if let Err(e) = CompleteAcquaintanceHandUseCase::new(&repo)
                .execute(ids)
                .await
            {
                // Локальная запись (IndexedDB) падает крайне редко; застрять
                // на тренировке хуже, чем потерять сидирование — завершаем
                // и при ошибке, деградация видна в логах.
                tracing::error!("Acquaintance hand completion failed: {e}");
            }
            state.update(|state| state.stage = AcquaintanceStage::Completed);
        });
    }
}

#[cfg(test)]
mod audio_button_visible_tests {
    use super::*;

    const WORD: bool = true;

    #[rstest::rstest]
    #[case::presentation(true, None, false)]
    #[case::presentation_answer_shown(true, None, true)]
    #[case::forward_front(true, Some(AcquaintanceSubphase::Forward), false)]
    #[case::forward_answer(true, Some(AcquaintanceSubphase::Forward), true)]
    #[case::reverse_front_hidden(false, Some(AcquaintanceSubphase::Reverse), false)]
    #[case::reverse_answer(true, Some(AcquaintanceSubphase::Reverse), true)]
    fn training_visibility_depends_on_jp_side(
        #[case] expected: bool,
        #[case] subphase: Option<AcquaintanceSubphase>,
        #[case] showing_answer: bool,
    ) {
        assert_eq!(
            audio_button_visible(AcquaintanceStage::Training, subphase, showing_answer, WORD),
            expected
        );
    }

    #[rstest::rstest]
    #[case::presentation_word(AcquaintanceStage::Presentation, true, true)]
    #[case::presentation_non_word(AcquaintanceStage::Presentation, false, false)]
    #[case::inactive_word(AcquaintanceStage::Inactive, true, false)]
    #[case::completed_word(AcquaintanceStage::Completed, true, false)]
    fn non_training_stages(
        #[case] stage: AcquaintanceStage,
        #[case] is_word: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(audio_button_visible(stage, None, false, is_word), expected);
    }
}

#[cfg(test)]
mod should_autoplay_word_audio_tests {
    use super::*;

    #[rstest::rstest]
    #[case::muted(true, true, false)]
    #[case::no_tts(false, false, false)]
    #[case::muted_and_no_tts(true, false, false)]
    #[case::ready(false, true, true)]
    fn autoplay_requires_tts_and_unmuted_lesson(
        #[case] is_muted: bool,
        #[case] speech_supported: bool,
        #[case] expected: bool,
    ) {
        assert_eq!(
            should_autoplay_word_audio(is_muted, speech_supported),
            expected
        );
    }
}

#[cfg(test)]
mod advance_presentation_tests {
    use super::*;
    use origa::domain::CardType;
    use std::collections::HashSet;

    fn state_with_hand(count: usize) -> AcquaintanceState {
        let pairs: Vec<(Ulid, CardType)> = (0..count)
            .map(|_| (Ulid::new(), CardType::Vocabulary))
            .collect();
        let hand = AcquaintanceHand::new(pairs).unwrap();
        AcquaintanceState {
            stage: AcquaintanceStage::Presentation,
            hand: Some(hand),
            slide_index: 0,
            skipped_ids: HashSet::new(),
            hand_finishing: false,
        }
    }

    #[test]
    fn advance_moves_through_slides_then_reports_exhausted() {
        // Arrange
        let mut state = state_with_hand(2);

        // Act / Assert
        assert!(!state.advance_presentation());
        assert_eq!(state.slide_index, 1);
        assert!(state.advance_presentation(), "показ исчерпан");
    }

    /// Reload урока переиспользует тот же state: несброшенное окно финиша
    /// прошлой руки ослепляло вторую руку сессии (кнопки/клавиатура
    /// гейтились навсегда) — ревью PR #498.
    #[test]
    fn start_new_hand_resets_finishing_window_and_stream_fields() {
        // Arrange: state закрытой руки — финиш, продвинутый показ, метки
        let mut state = state_with_hand(2);
        state.stage = AcquaintanceStage::Completed;
        state.slide_index = 1;
        state.hand_finishing = true;
        state.skipped_ids.insert(Ulid::new());

        // Act: reload пишет новую руку в тот же state
        let fresh = state_with_hand(1).hand.take().expect("hand built");
        state.start_new_hand(fresh);

        // Assert
        assert_eq!(state.stage, AcquaintanceStage::Presentation);
        assert_eq!(state.slide_index, 0);
        assert!(
            !state.hand_finishing,
            "окно финиша прошлой руки снято — кнопки новой руки работают"
        );
        assert!(state.skipped_ids.is_empty());
    }

    #[test]
    fn advance_skips_known_marked_cards() {
        // Arrange: средняя карта помечена «Уже знаю»
        let [a, b, c] = [Ulid::new(), Ulid::new(), Ulid::new()];
        let pairs = vec![
            (a, CardType::Vocabulary),
            (b, CardType::Vocabulary),
            (c, CardType::Vocabulary),
        ];
        let hand = AcquaintanceHand::new(pairs).unwrap();
        let mut state = AcquaintanceState {
            stage: AcquaintanceStage::Presentation,
            hand: Some(hand),
            slide_index: 0,
            skipped_ids: HashSet::from([b]),
            hand_finishing: false,
        };

        // Act / Assert: первый advance перепрыгивает b и показывает c
        assert!(!state.advance_presentation());
        assert_eq!(state.slide_index, 2);
        // следующий advance исчерпывает показ
        assert!(state.advance_presentation());
    }
}
