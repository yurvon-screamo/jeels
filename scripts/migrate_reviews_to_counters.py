#!/usr/bin/env python3
"""
Migrate domain_user.knowledge_set from reviews-array format to denormalized counters.

Reads each row in domain_user, decodes the knowledge_set blob (deflate+base64 per
ADR-034), walks every StudyCard's MemoryHistory.reviews array, computes scalar
counters (reps, lapses, easy_count, good_count, last_review_date, last_rating),
removes the reviews array, and writes the row back.

Usage:
    TRAILBASE_URL=https://app.origa.uwuwu.net \
    TRAILBASE_ADMIN_TOKEN=... \
    python scripts/migrate_reviews_to_counters.py [--dry-run]

The script reads/writes via the TrailBase admin records API. It does NOT touch
the `user` table — only `domain_user`.

Prerequisites:
    pip install requests
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
from datetime import datetime, timezone

import requests

# ─── ADR-034 codec constants ──────────────────────────────────────────
DEFLATE_PREFIX = "DEFLATE;"

# Rating enum values as serialized by serde (string variants)
RATING_EASY = "Easy"
RATING_GOOD = "Good"
RATING_HARD = "Hard"
RATING_AGAIN = "Again"


def decode_knowledge_set(raw: str) -> dict:
    """Decode knowledge_set wire blob to a dict.

    Handles both legacy plain-JSON and DEFLATE;<base64> formats (ADR-034).
    Recovering: corrupt input → empty KnowledgeSet.
    """
    if raw.startswith(DEFLATE_PREFIX):
        import zlib

        b64_data = raw[len(DEFLATE_PREFIX) :]
        try:
            deflated = base64.b64decode(b64_data)
            json_bytes = zlib.deinflate(deflated)
            return json.loads(json_bytes)
        except Exception as e:
            print(f"  WARN: decode failed ({e}), returning empty KnowledgeSet", file=sys.stderr)
            return {"study_cards": {}, "lesson_history": []}
    else:
        # Legacy plain JSON
        try:
            return json.loads(raw)
        except Exception as e:
            print(f"  WARN: JSON decode failed ({e}), returning empty", file=sys.stderr)
            return {"study_cards": {}, "lesson_history": []}


def encode_knowledge_set(ks: dict) -> str:
    """Encode KnowledgeSet dict to DEFLATE;<base64> wire format."""
    import zlib

    json_str = json.dumps(ks, separators=(",", ":"), ensure_ascii=False)
    deflated = zlib.compress(json_str.encode("utf-8"), level=6)
    return DEFLATE_PREFIX + base64.b64encode(deflated).decode("ascii")


def migrate_memory_history(memory: dict) -> dict:
    """Replace reviews array with scalar counters in a single MemoryHistory.

    Input shape (old):
        {
            "current_state": {...} | null,
            "reviews": [
                {"id": "...", "rating": "Good", "timestamp": "...", "interval": {...}},
                ...
            ]
        }

    Output shape (new):
        {
            "current_state": {...} | null,
            "reps": 3,
            "lapses": 1,
            "easy_count": 1,
            "good_count": 1,
            "last_review_date": "2025-01-01T12:00:00Z" | null,
            "last_rating": "Good" | null
        }
    """
    reviews = memory.get("reviews", [])
    current_state = memory.get("current_state")

    reps = len(reviews)
    lapses = sum(1 for r in reviews if r.get("rating") == RATING_AGAIN)
    easy_count = sum(1 for r in reviews if r.get("rating") == RATING_EASY)
    good_count = sum(1 for r in reviews if r.get("rating") == RATING_GOOD)

    # last_review_date: timestamp of the last review (chronologically)
    last_review_date = None
    last_rating = None
    if reviews:
        # Reviews are appended chronologically; last element is most recent.
        # But sort by timestamp defensively in case of disorder.
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


def migrate_knowledge_set(ks: dict) -> dict:
    """Migrate all StudyCards in a KnowledgeSet from reviews to counters."""
    study_cards = ks.get("study_cards", {})
    migrated_count = 0

    for card_id, study_card in study_cards.items():
        memory = study_card.get("memory_history")
        if memory is None:
            continue
        if "reviews" not in memory:
            # Already migrated or no reviews — skip
            continue
        study_card["memory_history"] = migrate_memory_history(memory)
        migrated_count += 1

    return ks, migrated_count


def main():
    parser = argparse.ArgumentParser(
        description="Migrate domain_user knowledge_set from reviews array to scalar counters"
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="Decode and show what would change, but don't write"
    )
    parser.add_argument(
        "--table", default="domain_user", help="TrailBase table name (default: domain_user)"
    )
    args = parser.parse_args()

    base_url = os.environ.get("TRAILBASE_URL")
    admin_token = os.environ.get("TRAILBASE_ADMIN_TOKEN")

    if not base_url or not admin_token:
        print("ERROR: TRAILBASE_URL and TRAILBASE_ADMIN_TOKEN must be set", file=sys.stderr)
        sys.exit(1)

    session = requests.Session()
    session.headers.update(
        {
            "Authorization": f"Bearer {admin_token}",
            "Content-Type": "application/json",
        }
    )

    # List all records
    list_url = f"{base_url}/api/records/v1/{args.table}?limit=1024"
    resp = session.get(list_url)
    resp.raise_for_status()
    data = resp.json()

    records = data.get("records", [])
    print(f"Found {len(records)} records in {args.table}")

    cursor = data.get("cursor")
    while cursor:
        next_url = f"{base_url}/api/records/v1/{args.table}?limit=1024&cursor={cursor}"
        resp = session.get(next_url)
        resp.raise_for_status()
        next_data = resp.json()
        records.extend(next_data.get("records", []))
        cursor = next_data.get("cursor")
        print(f"  Fetched {len(records)} records total...")

    total_cards_migrated = 0
    total_rows_updated = 0

    for record in records:
        record_id = record.get("id")
        email = record.get("email", "?")
        ks_raw = record.get("knowledge_set")

        if not ks_raw:
            print(f"  [{email}] No knowledge_set, skipping")
            continue

        # Skip if already default
        if ks_raw == '{"study_cards":{},"lesson_history":[]}':
            print(f"  [{email}] Empty knowledge_set, skipping")
            continue

        ks = decode_knowledge_set(ks_raw)
        card_count = len(ks.get("study_cards", {}))

        ks, migrated_count = migrate_knowledge_set(ks)
        total_cards_migrated += migrated_count

        new_ks_raw = encode_knowledge_set(ks)
        old_size = len(ks_raw)
        new_size = len(new_ks_raw)
        ratio = old_size / new_size if new_size > 0 else 0

        print(
            f"  [{email}] {card_count} cards, {migrated_count} migrated, "
            f"{old_size} → {new_size} bytes ({ratio:.1f}x reduction)"
        )

        if args.dry_run:
            print(f"    DRY RUN — not writing")
            continue

        # Write back
        update_url = f"{base_url}/api/records/v1/{args.table}/{record_id}"
        resp = session.patch(update_url, json={"knowledge_set": new_ks_raw})
        resp.raise_for_status()
        total_rows_updated += 1
        print(f"    Written ✓")

    print(f"\nDone. {total_cards_migrated} cards migrated, {total_rows_updated} rows updated.")
    if args.dry_run:
        print("(dry run — no changes written)")


if __name__ == "__main__":
    main()
