"""Unit tests for _knowledge_set_codec merge logic.

Each test mirrors a Rust domain test, verifying that the Python merge
produces the same result as the corresponding Rust implementation:

  - migrate_memory_history  ← MemoryHistory counters
  - merge_memory_history    ← MemoryHistory::merge
  - merge_study_card        ← StudyCard::merge
  - merge_knowledge_sets    ← KnowledgeSet::merge
  - merge_lesson_history    ← StatsTracker::merge / DailyHistoryItem::merge_with

Run: python scripts/test_sync_logic.py
"""

from __future__ import annotations

import os
import sys

sys.path.insert(0, os.path.dirname(__file__))
from _knowledge_set_codec import (  # noqa: E402
    decode_knowledge_set,
    encode_knowledge_set,
    merge_lesson_history,
    migrate_memory_history,
    merge_daily_history_item,
    merge_knowledge_sets,
    merge_memory_history,
    merge_study_card,
)

# ─── Test helpers ─────────────────────────────────────────────────────

NOW = "2026-08-10T12:00:00Z"
YESTERDAY = "2026-08-09T12:00:00Z"
TOMORROW = "2026-08-11T12:00:00Z"


def make_memory_with_counters(
    reps: int = 0,
    lapses: int = 0,
    easy_count: int = 0,
    good_count: int = 0,
    last_review_date: str | None = None,
    last_rating: str | None = None,
    stability: float | None = None,
    difficulty: float | None = None,
) -> dict:
    """Create a counter-format memory_history (new client format)."""
    mem = {
        "current_state": None,
        "reps": reps,
        "lapses": lapses,
        "easy_count": easy_count,
        "good_count": good_count,
        "last_review_date": last_review_date,
        "last_rating": last_rating,
    }
    if stability is not None and difficulty is not None:
        mem["current_state"] = {
            "stability": {"value": stability},
            "difficulty": {"value": difficulty},
            "next_review_date": last_review_date or NOW,
            "card_state": "Review",
        }
    return mem


def make_memory_with_reviews(ratings: list[tuple[str, str]]) -> dict:
    """Create a reviews-format memory_history (old client format).

    ratings: list of (timestamp, rating) pairs, chronological.
    """
    reviews = []
    for i, (ts, rating) in enumerate(ratings):
        reviews.append(
            {
                "id": f"01J000000000000000000000{i:02d}",
                "rating": rating,
                "timestamp": ts,
                "interval": {"secs": 86400, "nanos": 0},
            }
        )
    return {
        "current_state": {
            "stability": {"value": 5.0},
            "difficulty": {"value": 3.0},
            "next_review_date": ratings[-1][0] if ratings else NOW,
            "card_state": "Review",
        }
        if ratings
        else None,
        "reviews": reviews,
    }


def make_study_card(
    card_id: str = "01J0000000000000000000001A",
    memory: dict | None = None,
    is_favorite: bool = False,
    favorite_changed_at: str | None = None,
    streak: int = 0,
    word: str = "猫",
) -> dict:
    return {
        "card_id": card_id,
        "card": {"Vocabulary": {"word": {"text": word}}},
        "memory_history": memory or make_memory_with_counters(),
        "is_favorite": is_favorite,
        "perfect_streak_since_known": streak,
        "favorite_changed_at": favorite_changed_at,
    }


# ─── Reviews → Counters ──────────────────────────────────────────────


def test_migrate_basic():
    """Reviews array correctly converts to counters."""
    mem = make_memory_with_reviews(
        [(YESTERDAY, "Good"), (NOW, "Easy"), (NOW, "Again"), (NOW, "Good")]
    )
    result = migrate_memory_history(mem)

    assert result["reps"] == 4
    assert result["lapses"] == 1
    assert result["easy_count"] == 1
    assert result["good_count"] == 2
    assert result["last_rating"] == "Good"
    assert result["current_state"] is not None


def test_migrate_empty_reviews():
    """Empty reviews array produces zero counters."""
    mem = {"current_state": None, "reviews": []}
    result = migrate_memory_history(mem)

    assert result["reps"] == 0
    assert result["lapses"] == 0
    assert result["last_review_date"] is None
    assert result["last_rating"] is None


def test_migrate_preserves_current_state():
    """current_state is passed through unchanged."""
    state = {
        "stability": {"value": 10.0},
        "difficulty": {"value": 2.0},
        "next_review_date": NOW,
        "card_state": "Review",
    }
    mem = {"current_state": state, "reviews": []}
    result = migrate_memory_history(mem)

    assert result["current_state"] == state


def test_migrate_idempotent():
    """Double-migrating produces the same result."""
    mem = make_memory_with_reviews(
        [(YESTERDAY, "Good"), (NOW, "Again"), (NOW, "Easy")]
    )
    once = migrate_memory_history(mem)
    # No "reviews" key → migrate_memory_history should handle gracefully
    twice = {
        "current_state": once["current_state"],
    }
    twice.update({k: v for k, v in once.items() if k != "current_state"})

    assert once == twice


# ─── MemoryHistory merge ─────────────────────────────────────────────


def test_merge_memory_history_max_counters():
    """Counters merge via max() — mirrors merge_combines_counters_via_max."""
    local = make_memory_with_counters(
        reps=3, lapses=1, easy_count=1, good_count=2, last_review_date=YESTERDAY
    )
    remote = make_memory_with_counters(
        reps=1, lapses=0, easy_count=1, good_count=0, last_review_date=NOW
    )

    result = merge_memory_history(local, remote)

    assert result["reps"] == 3  # max(3, 1)
    assert result["lapses"] == 1  # max(1, 0)
    assert result["easy_count"] == 1  # max(1, 1)
    assert result["good_count"] == 2  # max(2, 0)


def test_merge_memory_history_takes_newer_rating():
    """last_rating from the newer side — mirrors merge_takes_last_rating_from_newer_side."""
    local = make_memory_with_counters(
        last_review_date=YESTERDAY, last_rating="Good", stability=3.0, difficulty=2.0
    )
    remote = make_memory_with_counters(
        last_review_date=NOW, last_rating="Easy", stability=5.0, difficulty=4.0
    )

    result = merge_memory_history(local, remote)

    assert result["last_rating"] == "Easy"
    assert result["current_state"]["stability"]["value"] == 5.0


def test_merge_memory_history_keeps_older_if_local_newer():
    """If local is newer, local state is preserved."""
    local = make_memory_with_counters(
        last_review_date=NOW,
        last_rating="Good",
        stability=10.0,
        difficulty=5.0,
    )
    remote = make_memory_with_counters(
        last_review_date=YESTERDAY,
        last_rating="Easy",
        stability=2.0,
        difficulty=1.0,
    )

    result = merge_memory_history(local, remote)

    assert result["last_rating"] == "Good"
    assert result["current_state"]["stability"]["value"] == 10.0


def test_merge_memory_history_idempotent():
    """Double-merge = single merge."""
    local = make_memory_with_counters(
        reps=3, lapses=1, easy_count=1, good_count=2,
        last_review_date=YESTERDAY, last_rating="Good",
        stability=5.0, difficulty=3.0,
    )
    remote = make_memory_with_counters(
        reps=1, lapses=0, easy_count=1, good_count=0,
        last_review_date=NOW, last_rating="Easy",
        stability=10.0, difficulty=7.0,
    )

    once = merge_memory_history(local, remote)
    twice = merge_memory_history(once, remote)

    assert once == twice


# ─── StudyCard merge ─────────────────────────────────────────────────


def test_merge_study_card_favorite_lww():
    """Favorite: LWW by favorite_changed_at — unfavorite wins with newer ts."""
    local = make_study_card(
        is_favorite=True, favorite_changed_at=YESTERDAY, streak=2
    )
    remote = make_study_card(
        is_favorite=False, favorite_changed_at=NOW, streak=0
    )

    result = merge_study_card(local, remote)

    assert result["is_favorite"] is False
    assert result["favorite_changed_at"] == NOW
    assert result["perfect_streak_since_known"] == 0  # unfavorite resets streak


def test_merge_study_card_favorite_keeps_newer_favorite():
    """Newer favorite=true wins over older favorite=false."""
    local = make_study_card(
        is_favorite=False, favorite_changed_at=YESTERDAY, streak=0
    )
    remote = make_study_card(
        is_favorite=True, favorite_changed_at=NOW, streak=3
    )

    result = merge_study_card(local, remote)

    assert result["is_favorite"] is True
    assert result["perfect_streak_since_known"] == 3  # max(0, 3)


def test_merge_study_card_streak_max():
    """favorite_easy_streak takes max."""
    local = make_study_card(is_favorite=True, streak=4)
    remote = make_study_card(is_favorite=True, streak=2)

    result = merge_study_card(local, remote)

    assert result["perfect_streak_since_known"] == 4  # max(4, 2)


def test_merge_study_card_legacy_no_timestamps():
    """No timestamps → OR fallback for is_favorite."""
    local = make_study_card(is_favorite=False, favorite_changed_at=None)
    remote = make_study_card(is_favorite=True, favorite_changed_at=None)

    result = merge_study_card(local, remote)

    assert result["is_favorite"] is True  # False OR True


# ─── KnowledgeSet merge ──────────────────────────────────────────────


def test_merge_knowledge_set_new_cards_added():
    """Cards in remote that aren't in local are added (unique content_key)."""
    local_ks = {
        "study_cards": {"card_a": make_study_card("card_a")},
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {
            "card_a": make_study_card("card_a"),
            "card_b": make_study_card("card_b", word="犬"),
        },
        "lesson_history": [],
    }

    result = merge_knowledge_sets(local_ks, remote_ks)

    assert "card_a" in result["study_cards"]
    assert "card_b" in result["study_cards"]


def test_merge_knowledge_set_duplicate_content_blocked():
    """A new card with same content_key as existing is NOT added.

    Mirrors Rust KnowledgeSet::merge → validate_unique_card gate.
    Both cards are Vocabulary with word "猫", but different card_ids.
    """
    local_ks = {
        "study_cards": {"card_a": make_study_card("card_a")},
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {
            "card_b": make_study_card("card_b"),  # same word "猫"
        },
        "lesson_history": [],
    }

    result = merge_knowledge_sets(local_ks, remote_ks)

    # card_b NOT added — duplicate word "猫"
    assert "card_a" in result["study_cards"]
    assert "card_b" not in result["study_cards"]


def test_merge_knowledge_set_tombstone_removes_card():
    """deleted_cards union removes matching study_cards."""
    local_ks = {
        "study_cards": {
            "card_a": make_study_card("card_a"),
            "card_b": make_study_card("card_b"),
        },
        "deleted_cards": [],
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {},
        "deleted_cards": ["card_a"],
        "lesson_history": [],
    }

    result = merge_knowledge_sets(local_ks, remote_ks)

    assert "card_a" not in result["study_cards"]
    assert "card_b" in result["study_cards"]
    assert "card_a" in result["deleted_cards"]


def test_merge_knowledge_set_tombstone_prevents_re_add():
    """A card in deleted_cards is not re-added from remote study_cards."""
    local_ks = {
        "study_cards": {},
        "deleted_cards": ["card_x"],
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {"card_x": make_study_card("card_x")},
        "deleted_cards": [],
        "lesson_history": [],
    }

    result = merge_knowledge_sets(local_ks, remote_ks)

    assert "card_x" not in result["study_cards"]


def test_merge_knowledge_set_companion_words_union():
    """deleted_companion_words are unioned."""
    local_ks = {
        "study_cards": {},
        "deleted_companion_words": ["犬"],
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {},
        "deleted_companion_words": ["猫", "犬"],
        "lesson_history": [],
    }

    result = merge_knowledge_sets(local_ks, remote_ks)

    assert set(result["deleted_companion_words"]) == {"犬", "猫"}


def test_merge_knowledge_set_idempotent():
    """Double-merge = single merge — critical for repeatable sync."""
    local_ks = {
        "study_cards": {
            "card_a": make_study_card(
                "card_a",
                memory=make_memory_with_counters(
                    reps=5, lapses=1, last_review_date=YESTERDAY,
                    last_rating="Good", stability=3.0, difficulty=2.0,
                ),
            ),
        },
        "deleted_cards": [],
        "deleted_companion_words": ["犬"],
        "lesson_history": [],
    }
    remote_ks = {
        "study_cards": {
            "card_a": make_study_card(
                "card_a",
                memory=make_memory_with_counters(
                    reps=3, lapses=0, last_review_date=NOW,
                    last_rating="Easy", stability=10.0, difficulty=5.0,
                ),
            ),
            "card_b": make_study_card("card_b", word="犬"),
        },
        "deleted_cards": [],
        "deleted_companion_words": ["猫"],
        "lesson_history": [],
    }

    once = merge_knowledge_sets(local_ks, remote_ks)
    twice = merge_knowledge_sets(once, remote_ks)

    assert once == twice


# ─── Lesson history merge ────────────────────────────────────────────


def test_merge_lesson_history_different_days():
    """History items from different days are both kept."""
    local = [
        {"timestamp": NOW, "lessons_completed": 3, "positive_ratings": 10},
    ]
    remote = [
        {"timestamp": YESTERDAY, "lessons_completed": 2, "positive_ratings": 5},
    ]

    result = merge_lesson_history(local, remote)

    assert len(result) == 2
    timestamps = [item["timestamp"] for item in result]
    assert timestamps == sorted(timestamps)


def test_merge_lesson_history_same_day_max():
    """Same-day items merge via max() for counters."""
    local = [
        {
            "timestamp": NOW,
            "lessons_completed": 3,
            "positive_ratings": 10,
            "new_cards_studied_today": 5,
        },
    ]
    remote = [
        {
            "timestamp": NOW,
            "lessons_completed": 5,
            "positive_ratings": 7,
            "new_cards_studied_today": 3,
        },
    ]

    result = merge_lesson_history(local, remote)

    assert len(result) == 1
    assert result[0]["lessons_completed"] == 5  # max(3, 5)
    assert result[0]["positive_ratings"] == 10  # max(10, 7)
    assert result[0]["new_cards_studied_today"] == 5  # max(5, 3)


def test_merge_daily_history_snapshot_lww():
    """avg_stability/avg_difficulty: LWW by timestamp."""
    local = {
        "timestamp": YESTERDAY,
        "avg_stability": 3.0,
        "avg_difficulty": 5.0,
        "lessons_completed": 2,
    }
    other = {
        "timestamp": NOW,
        "avg_stability": 10.0,
        "avg_difficulty": 2.0,
        "lessons_completed": 1,
    }

    merge_daily_history_item(local, other)

    assert local["avg_stability"] == 10.0  # newer wins
    assert local["avg_difficulty"] == 2.0
    assert local["lessons_completed"] == 2  # max(2, 1)


# ─── Codec round-trip ────────────────────────────────────────────────


def test_codec_roundtrip():
    """Encode → decode preserves data."""
    ks = {
        "study_cards": {"card_a": make_study_card("card_a")},
        "lesson_history": [{"timestamp": NOW, "lessons_completed": 1}],
        "deleted_cards": ["card_x"],
    }
    encoded = encode_knowledge_set(ks)
    decoded = decode_knowledge_set(encoded)

    assert decoded == ks


def test_codec_deflate_prefix():
    """Encoded blob starts with DEFLATE; prefix."""
    encoded = encode_knowledge_set({"study_cards": {}, "lesson_history": []})
    assert encoded.startswith("DEFLATE;")


def test_decode_corrupt_returns_empty():
    """Corrupt blob → empty KnowledgeSet (recovering policy)."""
    result = decode_knowledge_set("DEFLATE;!!!corrupt!!!")
    assert result == {"study_cards": {}, "lesson_history": []}


# ─── Cross-format merge (old user + new domain_user) ─────────────────


def test_cross_format_merge():
    """User has reviews format, domain_user has counters → merge works.

    This is the core sync scenario: old client wrote reviews, new client
    has counters. The sync script converts user's reviews → counters
    first (via migrate_knowledge_set), then merges.
    """
    from _knowledge_set_codec import migrate_knowledge_set

    # domain_user (new format): 1 card with 5 reps
    domain_ks = {
        "study_cards": {
            "card_a": make_study_card(
                "card_a",
                memory=make_memory_with_counters(
                    reps=5, lapses=0, easy_count=2, good_count=3,
                    last_review_date=NOW, last_rating="Good",
                    stability=10.0, difficulty=4.0,
                ),
            ),
        },
        "lesson_history": [],
    }

    # user (old format): same card with 3 reviews, 1 lapse, newer last_review
    user_ks = {
        "study_cards": {
            "card_a": {
                "card_id": "card_a",
                "card": {"Vocabulary": {"word": {"text": "猫"}}},
                "memory_history": make_memory_with_reviews(
                    [(YESTERDAY, "Good"), (YESTERDAY, "Again"), (TOMORROW, "Easy")]
                ),
                "is_favorite": False,
                "perfect_streak_since_known": 0,
                "favorite_changed_at": None,
            },
        },
        "lesson_history": [],
    }

    # Convert user's reviews → counters
    user_ks_converted, migrated = migrate_knowledge_set(user_ks)
    assert migrated == 1

    # Merge
    result = merge_knowledge_sets(domain_ks, user_ks_converted)
    card = result["study_cards"]["card_a"]
    mem = card["memory_history"]

    # Counters: max(5, 3) = 5 reps
    assert mem["reps"] == 5
    # Lapses: max(0, 1) = 1
    assert mem["lapses"] == 1
    # easy_count: max(2, 1) = 2
    assert mem["easy_count"] == 2
    # good_count: max(3, 1) = 3
    assert mem["good_count"] == 3
    # last_review_date: TOMORROW is newer than NOW → remote wins
    assert mem["last_review_date"] == TOMORROW
    assert mem["last_rating"] == "Easy"


# ─── Run ─────────────────────────────────────────────────────────────

ALL_TESTS = [
    v for k, v in sorted(globals().items()) if k.startswith("test_") and callable(v)
]


def main() -> None:
    passed = 0
    failed = 0
    for test in ALL_TESTS:
        try:
            test()
            print(f"  ✓ {test.__name__}")
            passed += 1
        except AssertionError as e:
            print(f"  ✗ {test.__name__}: {e}")
            failed += 1
        except Exception as e:
            print(f"  ✗ {test.__name__}: {type(e).__name__}: {e}")
            failed += 1

    print(f"\n{passed} passed, {failed} failed")
    if failed:
        sys.exit(1)


if __name__ == "__main__":
    main()
