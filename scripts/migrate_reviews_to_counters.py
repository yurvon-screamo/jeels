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
import os
import sys

import requests

# Import shared codec + merge logic
sys.path.insert(0, os.path.dirname(__file__))
from _knowledge_set_codec import (  # noqa: E402
    decode_knowledge_set,
    encode_knowledge_set,
    migrate_knowledge_set,
)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Migrate domain_user knowledge_set from reviews array to scalar counters"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Decode and show what would change, but don't write",
    )
    parser.add_argument(
        "--table",
        default="domain_user",
        help="TrailBase table name (default: domain_user)",
    )
    args = parser.parse_args()

    base_url = os.environ.get("TRAILBASE_URL")
    admin_token = os.environ.get("TRAILBASE_ADMIN_TOKEN")

    if not base_url or not admin_token:
        print(
            "ERROR: TRAILBASE_URL and TRAILBASE_ADMIN_TOKEN must be set",
            file=sys.stderr,
        )
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
            print("    DRY RUN — not writing")
            continue

        # Write back
        update_url = f"{base_url}/api/records/v1/{args.table}/{record_id}"
        resp = session.patch(update_url, json={"knowledge_set": new_ks_raw})
        resp.raise_for_status()
        total_rows_updated += 1
        print("    Written ✓")

    print(
        f"\nDone. {total_cards_migrated} cards migrated, {total_rows_updated} rows updated."
    )
    if args.dry_run:
        print("(dry run — no changes written)")


if __name__ == "__main__":
    main()
