"""Shared codec and merge logic for Origa knowledge_set blobs (ADR-034).

Used by:
  - migrate_reviews_to_counters.py — one-time reviews→counters conversion
  - sync_user_to_domain_user.py — rolling-deployment merge (user → domain_user)

All merge functions mirror the Rust domain contracts exactly:
  - MemoryHistory::merge     (origa/src/domain/memory/mod.rs)
  - StudyCard::merge          (origa/src/domain/knowledge/card.rs)
  - KnowledgeSet::merge       (origa/src/domain/knowledge/mod.rs)
  - StatsTracker::merge       (origa/src/domain/knowledge/stats_tracker.rs)
  - DailyHistoryItem::merge_with (origa/src/domain/knowledge/daily_history.rs)
"""

from __future__ import annotations

import base64
import copy
import json
import zlib
from typing import Any, cast

# ─── ADR-034 codec constants ──────────────────────────────────────────
DEFLATE_PREFIX = "DEFLATE;"

# Rating enum values as serialized by serde (string variants)
RATING_EASY = "Easy"
RATING_GOOD = "Good"
RATING_HARD = "Hard"
RATING_AGAIN = "Again"

# Serialized field name for favorite_easy_streak (serde rename in StudyCard)
PERFECT_STREAK_FIELD = "perfect_streak_since_known"


# ─── Encode / Decode ──────────────────────────────────────────────────


def decode_knowledge_set(raw: str) -> dict[str, Any]:
    """Decode knowledge_set wire blob to a dict.

    Handles both legacy plain-JSON and DEFLATE;<base64> formats (ADR-034).
    Recovering: corrupt input → empty KnowledgeSet.
    """
    if raw.startswith(DEFLATE_PREFIX):
        b64_data = raw[len(DEFLATE_PREFIX) :]
        try:
            deflated = base64.b64decode(b64_data)
            json_bytes = zlib.decompress(deflated)
            return cast(dict[str, Any], json.loads(json_bytes))
        except Exception:
            return _empty_knowledge_set()
    else:
        # Legacy plain JSON
        try:
            return cast(dict[str, Any], json.loads(raw))
        except Exception:
            return _empty_knowledge_set()


def encode_knowledge_set(ks: dict[str, Any]) -> str:
    """Encode KnowledgeSet dict to DEFLATE;<base64> wire format."""
    json_str = json.dumps(ks, separators=(",", ":"), ensure_ascii=False)
    deflated = zlib.compress(json_str.encode("utf-8"), level=6)
    return DEFLATE_PREFIX + base64.b64encode(deflated).decode("ascii")


def _empty_knowledge_set() -> dict[str, Any]:
    return {"study_cards": {}, "lesson_history": []}


# ─── Reviews → Counters conversion ────────────────────────────────────


def migrate_memory_history(memory: dict[str, Any]) -> dict[str, Any]:
    """Replace reviews array with scalar counters in a single MemoryHistory.

    Input shape (old):
        {"current_state": {...} | null, "reviews": [...]}
    Output shape (new):
        {"current_state": {...} | null, "reps": N, "lapses": N,
         "easy_count": N, "good_count": N,
         "last_review_date": "..." | null, "last_rating": "..." | null}
    """
    reviews = memory.get("reviews", [])
    current_state = memory.get("current_state")

    reps = len(reviews)
    lapses = sum(1 for r in reviews if r.get("rating") == RATING_AGAIN)
    easy_count = sum(1 for r in reviews if r.get("rating") == RATING_EASY)
    good_count = sum(1 for r in reviews if r.get("rating") == RATING_GOOD)

    last_review_date = None
    last_rating = None
    if reviews:
        sorted_reviews = sorted(reviews, key=lambda r: r.get("timestamp", ""))
        last = sorted_reviews[-1]
        last_review_date = last.get("timestamp")
        last_rating = last.get("rating")

    return {
        "current_state": current_state,
        "reps": reps,
        "lapses": lapses,
        "easy_count": easy_count,
        "good_count": good_count,
        "last_review_date": last_review_date,
        "last_rating": last_rating,
    }


def migrate_knowledge_set(ks: dict[str, Any]) -> tuple[dict[str, Any], int]:
    """Migrate all StudyCards from reviews to counters.

    Returns (migrated_ks, migrated_count). Idempotent: cards already in
    counter format (no "reviews" key) are skipped.
    """
    study_cards = ks.get("study_cards", {})
    migrated_count = 0

    for study_card in study_cards.values():
        memory = study_card.get("memory_history")
        if memory is None:
            continue
        if "reviews" not in memory:
            continue
        study_card["memory_history"] = migrate_memory_history(memory)
        migrated_count += 1

    return ks, migrated_count


# ─── Merge logic (mirrors Rust domain) ────────────────────────────────


def _select_later_state(
    left: dict[str, Any] | None,
    right: dict[str, Any] | None,
    left_last_review: str | None,
    right_last_review: str | None,
) -> dict[str, Any] | None:
    """Mirror select_later_state() — LWW by last_review_date."""
    if left is None and right is None:
        return None
    if left is not None and right is None:
        return copy.deepcopy(left)
    if left is None and right is not None:
        return copy.deepcopy(right)
    # Both present — compare last_review_date
    if left_last_review is None and right_last_review is None:
        return copy.deepcopy(right)
    if left_last_review is not None and right_last_review is None:
        return copy.deepcopy(left)
    if left_last_review is None and right_last_review is not None:
        return copy.deepcopy(right)
    # Both have timestamps — RFC3339 UTC strings compare lexicographically
    if right_last_review >= left_last_review:
        return copy.deepcopy(right)
    return copy.deepcopy(left)


def merge_memory_history(local: dict[str, Any], remote: dict[str, Any]) -> dict[str, Any]:
    """Mirror MemoryHistory::merge — merge remote into local."""
    local_lrd = local.get("last_review_date")
    remote_lrd = remote.get("last_review_date")

    current_state = _select_later_state(
        local.get("current_state"),
        remote.get("current_state"),
        local_lrd,
        remote_lrd,
    )

    # Counters: max()
    reps = max(local.get("reps", 0), remote.get("reps", 0))
    lapses = max(local.get("lapses", 0), remote.get("lapses", 0))
    easy_count = max(local.get("easy_count", 0), remote.get("easy_count", 0))
    good_count = max(local.get("good_count", 0), remote.get("good_count", 0))

    # last_review_date + last_rating: from whichever side is newer
    last_review_date = local_lrd
    last_rating = local.get("last_rating")
    if remote_lrd:
        if not local_lrd or remote_lrd >= local_lrd:
            last_review_date = remote_lrd
            last_rating = remote.get("last_rating")

    return {
        "current_state": current_state,
        "reps": reps,
        "lapses": lapses,
        "easy_count": easy_count,
        "good_count": good_count,
        "last_review_date": last_review_date,
        "last_rating": last_rating,
    }


def merge_study_card(local: dict[str, Any], remote: dict[str, Any]) -> dict[str, Any]:
    """Mirror StudyCard::merge — merge remote into local."""
    result = copy.deepcopy(local)

    # Merge memory_history
    result["memory_history"] = merge_memory_history(
        local.get("memory_history", {}),
        remote.get("memory_history", {}),
    )

    # Favorite merge — LWW by favorite_changed_at
    local_fca = local.get("favorite_changed_at")
    remote_fca = remote.get("favorite_changed_at")

    if local_fca and remote_fca:
        if remote_fca > local_fca:
            result["is_favorite"] = remote.get("is_favorite", False)
            result["favorite_changed_at"] = remote_fca
            if not result["is_favorite"]:
                result[PERFECT_STREAK_FIELD] = 0
    elif not local_fca and remote_fca:
        result["is_favorite"] = remote.get("is_favorite", False)
        result["favorite_changed_at"] = remote_fca
        if not result["is_favorite"]:
            result[PERFECT_STREAK_FIELD] = 0
    elif not local_fca and not remote_fca:
        # Legacy: no timestamps — OR fallback
        result["is_favorite"] = local.get("is_favorite", False) or remote.get(
            "is_favorite", False
        )
    # else: local has fca, remote doesn't → keep local (already in result)

    # favorite_easy_streak: max(result, remote) — result may have been reset
    # to 0 above by an unfavorite-wins branch, mirroring Rust's
    # self.favorite_easy_streak.max(other.favorite_easy_streak) after the
    # favorite LWW branch already mutated self.favorite_easy_streak.
    result[PERFECT_STREAK_FIELD] = max(
        result.get(PERFECT_STREAK_FIELD, 0),
        remote.get(PERFECT_STREAK_FIELD, 0),
    )

    return result


def merge_daily_history_item(self_item: dict[str, Any], other: dict[str, Any]) -> None:
    """Mirror DailyHistoryItem::merge_with — in-place on self_item."""
    self_item["lessons_completed"] = max(
        self_item.get("lessons_completed", 0), other.get("lessons_completed", 0)
    )
    self_item["positive_ratings"] = max(
        self_item.get("positive_ratings", 0), other.get("positive_ratings", 0)
    )
    self_item["negative_ratings"] = max(
        self_item.get("negative_ratings", 0), other.get("negative_ratings", 0)
    )
    self_item["total_ratings"] = max(
        self_item.get("total_ratings", 0), other.get("total_ratings", 0)
    )
    self_item["new_cards_studied_today"] = max(
        self_item.get("new_cards_studied_today", 0),
        other.get("new_cards_studied_today", 0),
    )
    self_item["phrase_cards_studied_today"] = max(
        self_item.get("phrase_cards_studied_today", 0),
        other.get("phrase_cards_studied_today", 0),
    )

    other_ts = other.get("timestamp", "")
    self_ts = self_item.get("timestamp", "")
    if other_ts > self_ts:
        self_item["timestamp"] = other_ts
        self_item["avg_stability"] = other.get("avg_stability")
        self_item["avg_difficulty"] = other.get("avg_difficulty")


def _date_part(timestamp: str) -> str:
    """Extract date (YYYY-MM-DD) from RFC3339 timestamp string."""
    return timestamp[:10] if timestamp else ""


def merge_lesson_history(
    local: list[dict[str, Any]], remote: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    """Mirror StatsTracker::merge — merge remote lesson_history into local."""
    result = [copy.deepcopy(item) for item in local]

    for remote_item in remote:
        remote_date = _date_part(remote_item.get("timestamp", ""))
        found = False
        for existing in result:
            if _date_part(existing.get("timestamp", "")) == remote_date:
                merge_daily_history_item(existing, remote_item)
                found = True
                break
        if not found:
            result.append(copy.deepcopy(remote_item))

    result.sort(key=lambda x: x.get("timestamp", ""))
    return result


def _card_content_key(card: dict[str, Any]) -> str | None:
    """Extract the content key from a serialized Card enum.

    Mirrors Card::content_key() in Rust. The serialized shape is
    {"Variant": {...}} (externally tagged serde enum).
    """
    if not isinstance(card, dict) or len(card) != 1:
        return None
    (variant, inner), = card.items()
    match variant:
        case "Vocabulary":
            return _nested_text(inner, "word")
        case "Kanji":
            return _nested_text(inner, "kanji")
        case "Grammar":
            return inner.get("rule_id") if isinstance(inner, dict) else None
        case "Phrase":
            return inner.get("phrase_id") if isinstance(inner, dict) else None
        case _:
            return None


def _nested_text(inner: Any, field: str) -> str | None:
    """Extract inner[field]["text"] from a serialized card variant."""
    if not isinstance(inner, dict):
        return None
    nested = inner.get(field)
    if not isinstance(nested, dict):
        return None
    return nested.get("text")


def _validate_unique_card(
    study_cards: dict[str, Any], new_card: dict[str, Any]
) -> bool:
    """Mirror KnowledgeSet::validate_unique_card.

    Returns True if the card passes (no duplicate), False otherwise.
    Checks content_key uniqueness (word, kanji, rule_id, phrase_id).
    """
    new_key = _card_content_key(new_card)
    if new_key is None:
        # Unknown card shape — allow (can't validate)
        return True

    for existing_card in study_cards.values():
        existing = existing_card.get("card") if isinstance(existing_card, dict) else None
        if existing is None:
            continue
        existing_key = _card_content_key(existing)
        if existing_key == new_key:
            return False

    return True


def merge_knowledge_sets(
    local_ks: dict[str, Any], remote_ks: dict[str, Any]
) -> dict[str, Any]:
    """Mirror KnowledgeSet::merge — merge remote into local.

    Idempotent: LWW + max() + union semantics ensure repeated runs produce
    the same result if inputs don't change.
    """
    result = copy.deepcopy(local_ks)

    # Union deleted_cards (tombstones) — remove from study_cards
    result_deleted = set(result.get("deleted_cards", []))
    for deleted_id in remote_ks.get("deleted_cards", []):
        result_deleted.add(deleted_id)
        result.get("study_cards", {}).pop(deleted_id, None)
    result["deleted_cards"] = list(result_deleted)

    # Union deleted_companion_words
    result_companion = set(result.get("deleted_companion_words", []))
    result_companion.update(remote_ks.get("deleted_companion_words", []))
    result["deleted_companion_words"] = list(result_companion)

    # Per-card merge
    result.setdefault("study_cards", {})
    for card_id, remote_card in remote_ks.get("study_cards", {}).items():
        if card_id in result_deleted:
            continue
        if card_id in result["study_cards"]:
            result["study_cards"][card_id] = merge_study_card(
                result["study_cards"][card_id], remote_card
            )
        elif _validate_unique_card(result["study_cards"], remote_card.get("card", {})):
            # New card — add only if content_key is unique (mirrors Rust
            # KnowledgeSet::merge validate_unique_card gate)
            result["study_cards"][card_id] = copy.deepcopy(remote_card)

    # Merge lesson_history (StatsTracker)
    result["lesson_history"] = merge_lesson_history(
        result.get("lesson_history", []),
        remote_ks.get("lesson_history", []),
    )

    return result
