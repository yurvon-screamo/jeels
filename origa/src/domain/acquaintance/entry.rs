//! Карта внутри руки и её прогресс.

use crate::domain::CardType;
use ulid::Ulid;

use super::hand::CRITERION_SUCCESSSES;
use super::phase::AcquaintanceSubphase;

/// Карта внутри руки. Слова ведут два счётчика (по подфазам), несловесные
/// карты — один (`forward_successes`), действующий во всех витках.
#[derive(Debug)]
pub struct AcquaintanceEntry {
    pub(super) card_id: Ulid,
    pub(super) card_type: CardType,
    pub(super) forward_successes: u8,
    pub(super) reverse_successes: u8,
}

impl AcquaintanceEntry {
    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn card_type(&self) -> CardType {
        self.card_type
    }

    /// Видимый прогресс карты: счётчик текущей подфазы для слов, единый
    /// счётчик для несловесных карт.
    pub fn progress_in(&self, subphase: Option<AcquaintanceSubphase>) -> u8 {
        match subphase {
            Some(AcquaintanceSubphase::Reverse) if self.is_word() => self.reverse_successes,
            _ => self.forward_successes,
        }
    }

    /// Закрыла ли карта свой критерий полностью (для слов — в обеих подфазах).
    pub fn criterion_met(&self, subphase: Option<AcquaintanceSubphase>) -> bool {
        self.progress_in(subphase) >= CRITERION_SUCCESSSES
            && (!self.is_word() || self.reverse_successes >= CRITERION_SUCCESSSES)
    }

    pub(super) fn is_word(&self) -> bool {
        self.card_type == CardType::Vocabulary
    }

    pub(super) fn record_success(&mut self, subphase: Option<AcquaintanceSubphase>) {
        match subphase {
            Some(AcquaintanceSubphase::Reverse) if self.is_word() => {
                self.reverse_successes += 1;
            },
            _ => self.forward_successes += 1,
        }
    }
}
