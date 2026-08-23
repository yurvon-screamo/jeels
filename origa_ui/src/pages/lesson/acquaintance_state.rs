use crate::repository::HybridUserRepository;
use leptos::prelude::*;
use origa::domain::{AcquaintanceHand, NativeLanguage};
use std::collections::HashSet;
use ulid::Ulid;

use super::kanji_card_details::RadicalDisplay;
use crate::ui_components::ReadingItem;

/// Стадии руки знакомства на странице урока (docs/acquaintance-mode.md):
/// показ → тренировка → итог → обычное ревью. `Inactive` — руки нет
/// (пул пуст / лимит исчерпан / рука уже передана в ревью).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcquaintanceStage {
    #[default]
    Inactive,
    Presentation,
    /// Срез S5 заменит заглушку полноценной тренировкой.
    Training,
    // Summary добавляется срезом S6 (итоговый экран руки).
}

/// UI-состояние руки: доменная машина (`AcquaintanceHand`) хранит правила,
/// этот тип — только позицию юзера внутри потока и служебные флаги.
#[derive(Clone, Default)]
pub struct AcquaintanceState {
    pub stage: AcquaintanceStage,
    pub hand: Option<AcquaintanceHand>,
    pub slide_index: usize,
    pub confirm_known: bool,
    pub skipped_ids: HashSet<Ulid>,
}

impl AcquaintanceState {
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
}

/// Контекст префазы урока.
#[derive(Clone)]
pub struct AcquaintanceContext {
    pub repository: HybridUserRepository,
    pub state: RwSignal<AcquaintanceState>,
    pub slides: RwSignal<Vec<AcquaintanceSlideData>>,
    pub known_kanji: RwSignal<HashSet<char>>,
    pub native_language: RwSignal<NativeLanguage>,
}
