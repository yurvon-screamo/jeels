//! Типы руки знакомства: состав, подфазы тренировки слов и исходы ответов.
//!
//! Каркас контракта (docs/acquaintance-mode.md §9.2). Логика витков,
//! подфаз и критериев добавляется срезом S1; порядок показа рука получает
//! уже сгруппированным от `SelectAcquaintanceHandUseCase`.

use crate::domain::CardType;
use ulid::Ulid;

/// Подфаза тренировки слов. Несловесные карты (кандзи, грамматика) имеют
/// единственное направление и в подфазах не участвуют.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcquaintanceSubphase {
    /// яп→рус: фронт — японская сторона.
    Forward,
    /// рус→яп: фронт — перевод.
    Reverse,
}

/// Исход одного ответа в тренировке. Единственный источник истины для
/// перерисовки UI (полоса руки, тег фазы): рассинхрон между доменом и
/// отображением невозможен по построению.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerOutcome {
    /// Успех засчитан; `progress` — число успехов карты в текущей подфазе.
    Counted { progress: u8 },
    /// Карта уже закрыла критерий: ответ не меняет её прогресс.
    ProgressFrozen,
    /// Все слова закрыли подфазу: счётчики слов обнулены, направление сменилось.
    SubphaseAdvanced,
    /// Последняя карта закрыла последний критерий — тренировка завершена.
    HandCompleted,
}

/// Карта внутри руки вместе со счётчиком успехов текущей подфазы
/// (для несловесных карт — общий счётчик единственного направления).
pub struct AcquaintanceEntry {
    card_id: Ulid,
    card_type: CardType,
    successes: u8,
}

impl AcquaintanceEntry {
    pub fn card_id(&self) -> Ulid {
        self.card_id
    }

    pub fn card_type(&self) -> CardType {
        self.card_type
    }

    pub fn successes(&self) -> u8 {
        self.successes
    }
}

/// Рука знакомства: атомарная партия незнакомых карт одного урока.
pub struct AcquaintanceHand {
    entries: Vec<AcquaintanceEntry>,
}

impl AcquaintanceHand {
    #[cfg(test)]
    pub(crate) fn new_test(entries: Vec<(Ulid, CardType, u8)>) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(card_id, card_type, successes)| AcquaintanceEntry {
                    card_id,
                    card_type,
                    successes,
                })
                .collect(),
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

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_order_preserves_given_order() {
        // Arrange
        let (a, b, c) = (Ulid::new(), Ulid::new(), Ulid::new());
        let hand = AcquaintanceHand::new_test(vec![
            (a, CardType::Kanji, 0),
            (b, CardType::Vocabulary, 1),
            (c, CardType::Grammar, 2),
        ]);

        // Act
        let order = hand.presentation_order();

        // Assert
        assert_eq!(order, vec![a, b, c]);
    }

    #[test]
    fn entry_lookup_returns_entry_by_card_id() {
        // Arrange
        let (a, b) = (Ulid::new(), Ulid::new());
        let hand =
            AcquaintanceHand::new_test(vec![(a, CardType::Kanji, 2), (b, CardType::Vocabulary, 0)]);

        // Act / Assert
        assert_eq!(hand.entry(a).map(|entry| entry.successes()), Some(2));
        assert_eq!(
            hand.entry(b).map(|entry| entry.card_type()),
            Some(CardType::Vocabulary)
        );
        assert!(hand.entry(Ulid::new()).is_none());
    }

    #[test]
    fn empty_hand_reports_empty() {
        // Arrange / Act
        let hand = AcquaintanceHand::new_test(vec![]);

        // Assert
        assert!(hand.is_empty());
        assert_eq!(hand.len(), 0);
    }
}
