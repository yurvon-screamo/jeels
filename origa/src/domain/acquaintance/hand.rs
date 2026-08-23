//! Рука знакомства: атомарная партия незнакомых карт одного урока.
//!
//! Правила поведения — docs/acquaintance-mode.md, правило «Тренировка»:
//! полные ротации случайного порядка (порядок витка принадлежит UI),
//! критерий `CRITERION_SUCCESSSES` успехов на карту; подфазы яп→рус → рус→яп
//! относятся только к словам, несловесные карты копят единый счётчик сквозь
//! обе подфазы. Закрывшие критерий продолжают отвечаться с замороженным
//! прогрессом.

use crate::domain::{CardType, OrigaError};
use ulid::Ulid;

use super::entry::AcquaintanceEntry;
use super::phase::{AcquaintanceSubphase, AnswerOutcome};

/// Число успешных ответов, требуемое карте: в каждой подфазе (слова) /
/// за тренировку (несловесные карты).
pub const CRITERION_SUCCESSSES: u8 = 3;

/// Максимальный размер руки знакомства (docs/acquaintance-mode.md §6).
pub const HAND_MAX_SIZE: usize = 7;

/// Рука знакомства: партия незнакомых карт одного урока.
#[derive(Debug)]
pub struct AcquaintanceHand {
    entries: Vec<AcquaintanceEntry>,
    subphase: Option<AcquaintanceSubphase>,
}

impl AcquaintanceHand {
    /// Собирает руку из уже сгруппированного показательного порядка
    /// (кандзи первым, его слова этой руки рядом — группирует вызывающий).
    /// Фразы в режиме знакомства не участвуют.
    pub fn new(cards: Vec<(Ulid, CardType)>) -> Result<Self, OrigaError> {
        if cards.is_empty() {
            return Err(OrigaError::InvalidAcquaintanceHand {
                reason: "hand is empty".to_string(),
            });
        }

        let mut seen = std::collections::HashSet::new();
        for (card_id, card_type) in &cards {
            if *card_type == CardType::Phrase {
                return Err(OrigaError::InvalidAcquaintanceHand {
                    reason: format!(
                        "phrase card {card_id} does not participate in acquaintance mode"
                    ),
                });
            }
            if !seen.insert(*card_id) {
                return Err(OrigaError::InvalidAcquaintanceHand {
                    reason: format!("duplicate card {card_id}"),
                });
            }
        }

        let has_words = cards
            .iter()
            .any(|(_, card_type)| *card_type == CardType::Vocabulary);
        Ok(Self {
            entries: cards
                .into_iter()
                .map(|(card_id, card_type)| AcquaintanceEntry {
                    card_id,
                    card_type,
                    forward_successes: 0,
                    reverse_successes: 0,
                })
                .collect(),
            subphase: has_words.then_some(AcquaintanceSubphase::Forward),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_test(
        entries: Vec<(Ulid, CardType, u8, u8)>,
        subphase: Option<AcquaintanceSubphase>,
    ) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(card_id, card_type, forward, reverse)| AcquaintanceEntry {
                    card_id,
                    card_type,
                    forward_successes: forward,
                    reverse_successes: reverse,
                })
                .collect(),
            subphase,
        }
    }

    /// Состав руки в показательном порядке (кандзи первым, его слова рядом).
    /// Ротационный порядок витков принадлежит UI.
    pub fn presentation_order(&self) -> Vec<Ulid> {
        self.entries.iter().map(|entry| entry.card_id).collect()
    }

    pub fn entry(&self, card_id: Ulid) -> Option<&AcquaintanceEntry> {
        self.entries.iter().find(|entry| entry.card_id == card_id)
    }

    /// Текущая подфаза тренировки слов; `None`, если слов в руке нет.
    pub fn subphase(&self) -> Option<AcquaintanceSubphase> {
        self.subphase
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Обрабатывает ответ юзера по карте текущего витка.
    ///
    /// Успех продвигает видимый счётчик карты и при необходимости
    /// переключает подфазу или закрывает руку; провал и ответы по картам,
    /// закрывшим критерий текущей подфазы (или общий — для несловесных),
    /// прогресс не меняют.
    pub fn record_answer(
        &mut self,
        card_id: Ulid,
        remembered: bool,
    ) -> Result<AnswerOutcome, OrigaError> {
        let entry_index = self
            .entries
            .iter()
            .position(|entry| entry.card_id == card_id)
            .ok_or(OrigaError::CardNotFound { card_id })?;

        let subphase = self.subphase;
        if self.entries[entry_index].progress_in(subphase) >= CRITERION_SUCCESSSES {
            // Закрывшие критерий продолжают отвечаться с замороженным
            // прогрессом — и на успех, и на провал.
            return Ok(AnswerOutcome::ProgressFrozen);
        }
        if !remembered {
            return Ok(AnswerOutcome::Failed);
        }

        self.entries[entry_index].record_success(subphase);

        if let Some(AcquaintanceSubphase::Forward) = self.subphase {
            let answered_is_word = self.entries[entry_index].is_word();
            let all_words_closed_forward = self
                .entries
                .iter()
                .filter(|entry| entry.is_word())
                .all(|word| word.forward_successes >= CRITERION_SUCCESSSES);
            if answered_is_word && all_words_closed_forward {
                self.subphase = Some(AcquaintanceSubphase::Reverse);
                return Ok(AnswerOutcome::SubphaseAdvanced);
            }
        }

        if self
            .entries
            .iter()
            .all(|entry| entry.criterion_met(self.subphase))
        {
            return Ok(AnswerOutcome::HandCompleted);
        }

        Ok(AnswerOutcome::Counted {
            progress: self.entries[entry_index].progress_in(subphase),
        })
    }
}
