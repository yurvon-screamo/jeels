use std::collections::HashMap;

use origa::domain::{CardType, JapaneseLevel, StudyCard};
use ulid::Ulid;

/// Controls whether `card_list_view` renders a flat list or splits cards into
/// JLPT-level groups (N5 first, Other last).
///
/// `ByJlptLevel` is currently unused at call sites — JLPT grouping was reverted
/// from `/grammar` and `/kanji` because the `GroupedGrid` snapshot pattern
/// goes stale on a favorite-toggle + reload cycle. The variant and the
/// surrounding infrastructure (this file, `grouped_grid.rs`, the `ByJlptLevel`
/// branch in `card_list_view`) are kept so a follow-up PR can ship a reactive
/// `GroupedGrid` without rebuilding the scaffolding.
#[derive(Clone, Copy)]
pub enum ListGrouping {
    Flat,
    #[allow(dead_code)]
    ByJlptLevel {
        card_type: CardType,
    },
}

/// Stable sort rank for a card's JLPT level. Lower rank renders first.
/// Cards whose level could not be determined (`None`) go last as "Other".
pub fn group_rank(level: &Option<JapaneseLevel>) -> u8 {
    match level {
        Some(JapaneseLevel::N5) => 0,
        Some(JapaneseLevel::N4) => 1,
        Some(JapaneseLevel::N3) => 2,
        Some(JapaneseLevel::N2) => 3,
        Some(JapaneseLevel::N1) => 4,
        None => 5,
    }
}

/// Type alias for the card_id -> Optional level lookup built once per page
/// load and reused across filter/search recomputations.
pub type LevelIndex = HashMap<Ulid, Option<JapaneseLevel>>;

/// Abstraction over anything that exposes a `Ulid` card id, so the sorter
/// works for `StudyCard` as well as test stubs.
pub trait CardIdLike {
    fn card_id(&self) -> Ulid;
}

impl CardIdLike for StudyCard {
    fn card_id(&self) -> Ulid {
        *StudyCard::card_id(self)
    }
}

/// Orders `cards` by group first (N5 -> Other) then by `card_id` within a
/// group. Pure function — no Leptos primitives, fully unit-testable.
///
/// `index` must contain an entry for every `card_id` in `cards`; missing
/// entries are treated as "Other" (rank 5).
pub fn order_cards_by_group<C>(cards: &[C], index: &LevelIndex) -> Vec<C>
where
    C: Clone + CardIdLike,
{
    let mut sorted: Vec<C> = cards.to_vec();
    sorted.sort_by(|a, b| {
        // `index.get` returns `Option<&Option<JL>>`; deref to `&Option<JL>`
        // explicitly so the types align with `group_rank`.
        let rank_a = index.get(&a.card_id()).map_or(5, group_rank);
        let rank_b = index.get(&b.card_id()).map_or(5, group_rank);
        rank_a
            .cmp(&rank_b)
            .then_with(|| a.card_id().cmp(&b.card_id()))
    });
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct Stub {
        id: Ulid,
    }
    impl CardIdLike for Stub {
        fn card_id(&self) -> Ulid {
            self.id
        }
    }

    fn stub(byte: u8) -> Stub {
        // Build a deterministic Ulid from a single byte so card_ids compare
        // the same way as the byte — keeps the within-group ordering obvious.
        let mut bytes = [0u8; 16];
        bytes[15] = byte;
        Stub {
            id: Ulid::from_bytes(bytes),
        }
    }

    fn index_of(items: &[(u8, Option<JapaneseLevel>)]) -> LevelIndex {
        items
            .iter()
            .map(|(b, level)| (stub(*b).id, *level))
            .collect()
    }

    #[test]
    fn group_rank_orders_n5_through_n1_with_other_last() {
        // Assert — strictly ascending ranks per documented order
        assert_eq!(group_rank(&Some(JapaneseLevel::N5)), 0);
        assert_eq!(group_rank(&Some(JapaneseLevel::N4)), 1);
        assert_eq!(group_rank(&Some(JapaneseLevel::N3)), 2);
        assert_eq!(group_rank(&Some(JapaneseLevel::N2)), 3);
        assert_eq!(group_rank(&Some(JapaneseLevel::N1)), 4);
        assert_eq!(group_rank(&None), 5);
    }

    #[test]
    fn order_cards_by_group_sorts_by_level_first() {
        // Arrange — N1 card comes first in source, must move to the end
        let cards = vec![stub(1), stub(2)];
        let index = index_of(&[(1, Some(JapaneseLevel::N1)), (2, Some(JapaneseLevel::N5))]);

        // Act
        let ordered = order_cards_by_group(&cards, &index);

        // Assert — N5 (rank 0) before N1 (rank 4)
        assert_eq!(ordered[0].id, stub(2).id);
        assert_eq!(ordered[1].id, stub(1).id);
    }

    #[test]
    fn order_cards_by_group_preserves_card_id_order_within_group() {
        // Arrange — two N5 cards, ids 10 and 20, source order reversed
        let cards = vec![stub(20), stub(10)];
        let index = index_of(&[(10, Some(JapaneseLevel::N5)), (20, Some(JapaneseLevel::N5))]);

        // Act
        let ordered = order_cards_by_group(&cards, &index);

        // Assert — within N5, smaller card_id first
        assert_eq!(ordered[0].id, stub(10).id);
        assert_eq!(ordered[1].id, stub(20).id);
    }

    #[test]
    fn order_cards_by_group_other_goes_last() {
        // Arrange — mix of all levels including None (Other)
        let cards = vec![stub(1), stub(2), stub(3), stub(4), stub(5), stub(6)];
        let index = index_of(&[
            (1, None),
            (2, Some(JapaneseLevel::N1)),
            (3, Some(JapaneseLevel::N5)),
            (4, Some(JapaneseLevel::N4)),
            (5, None),
            (6, Some(JapaneseLevel::N3)),
        ]);

        // Act
        let ordered = order_cards_by_group(&cards, &index);

        // Assert — order: N5, N4, N3, N1, None, None
        let ordered_ids: Vec<u8> = ordered.iter().map(|s| s.id.to_bytes()[15]).collect();
        assert_eq!(ordered_ids, vec![3, 4, 6, 2, 1, 5]);
    }

    #[test]
    fn order_cards_by_group_missing_index_entry_treated_as_other() {
        // Arrange — card_id absent from index must fall into Other group
        let cards = vec![stub(1), stub(2)];
        let index = index_of(&[(1, Some(JapaneseLevel::N5))]); // 2 is missing

        // Act
        let ordered = order_cards_by_group(&cards, &index);

        // Assert — N5 (1) first, missing (2) last as Other
        assert_eq!(ordered[0].id, stub(1).id);
        assert_eq!(ordered[1].id, stub(2).id);
    }
}
