//! Режим знакомства с новыми картами (см. docs/acquaintance-mode.md).
//!
//! Рука — партия незнакомых карт, проводимая через показ и тренировку до
//! критерия, после чего вся рука целиком входит в SRS с первым ревью
//! назавтра. Рука эфемерна: состояние не персистится, прерывание — это
//! отсутствие записи.

mod hand;
mod seed;

pub use hand::{AcquaintanceEntry, AcquaintanceHand, AcquaintanceSubphase, AnswerOutcome};
pub use seed::seed_first_review;
