//! Режим знакомства с новыми картами (см. docs/acquaintance-mode.md).
//!
//! Рука — партия незнакомых карт, проводимая через показ и тренировку до
//! критерия, после чего вся рука целиком входит в SRS с первым ревью
//! назавтра. Рука эфемерна: состояние не персистится, прерывание — это
//! отсутствие записи.

#[cfg(test)]
mod completion_tests;
mod entry;
mod hand;
#[cfg(test)]
mod hand_tests;
mod phase;
mod seed;
#[cfg(test)]
mod training_tests;

pub use entry::AcquaintanceEntry;
pub use hand::{AcquaintanceHand, CRITERION_SUCCESSSES};
pub use phase::{AcquaintanceSubphase, AnswerOutcome};
pub use seed::seed_first_review;
