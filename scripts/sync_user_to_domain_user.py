#!/usr/bin/env python3
"""
Sync users from the `user` table (old client, reviews format) into the
`domain_user` table (new client, counters format).

Rolling-deployment migration (PR #350): the old client keeps writing to
`user`, the new client writes to `domain_user`. This script merges any
fresh data from `user` into `domain_user`, so new-client users who
reviewed cards on an old-client device during the rollout window don't
lose progress.

The merge is idempotent (LWW + max() + union semantics) — it can be run
repeatedly as the app-store rollout progresses, and the final run after
100% rollout guarantees no data is left behind.

⚠️  CONCURRENCY: This script does a read-merge-write cycle without
    optimistic locking. If a new-client user does save_sync to
    domain_user while this script runs, their write may be lost.
    Mitigations:
      - Run during low-traffic window (e.g. night)
      - The idempotent merge means re-runs are safe — data lost to a
        race will be recovered on the next sync run if the source data
        still exists in `user`.

Usage:
    TRAILBASE_URL=https://app.origa.uwuwu.net \\
    TRAILBASE_ADMIN_TOKEN=... \\
    python scripts/sync_user_to_domain_user.py [--apply]

Without --apply the script runs in dry-run mode: it shows what would
change but writes nothing.

The merge logic mirrors the Rust domain contracts exactly:
  - MemoryHistory::merge     (LWW state + max counters)
  - StudyCard::merge          (LWW favorites + max streak)
  - KnowledgeSet::merge       (union tombstones + per-card merge)
  - StatsTracker::merge       (per-date lesson history)

Prerequisites:
    pip install requests
"""

from __future__ import annotations

import argparse
import copy
import json
import os
import sys
from datetime import datetime
from typing import Any

import requests

# Import shared codec + merge logic
sys.path.insert(0, os.path.dirname(__file__))
from _knowledge_set_codec import (  # noqa: E402
    decode_knowledge_set,
    encode_knowledge_set,
    migrate_knowledge_set,
    merge_knowledge_sets,
)

# ─── HTTP helpers ─────────────────────────────────────────────────────


def fetch_all_records(session: requests.Session, base_url: str, table: str) -> list[dict]:
    """Fetch all records from a TrailBase table with cursor pagination."""
    records: list[dict] = []
    cursor = None

    while True:
        params: dict[str, Any] = {"limit": 1024}
        if cursor:
            params["cursor"] = cursor

        url = f"{base_url}/api/records/v1/{table}"
        resp = session.get(url, params=params)
        resp.raise_for_status()
        data = resp.json()

        batch = data.get("records", [])
        records.extend(batch)
        cursor = data.get("cursor")

        if not cursor:
            break

    return records


def parse_timestamp(ts: str | None) -> datetime | None:
    """Parse RFC3339 timestamp string to datetime (UTC)."""
    if not ts:
        return None
    try:
        # Handle both 'Z' suffix and explicit offset
        if ts.endswith("Z"):
            return datetime.fromisoformat(ts[:-1] + "+00:00")
        return datetime.fromisoformat(ts)
    except (ValueError, TypeError):
        return None


# ─── Field-level merge ────────────────────────────────────────────────


def merge_scalar_fields(
    user_row: dict[str, Any], domain_row: dict[str, Any]
) -> dict[str, Any]:
    """Merge non-knowledge_set fields from user (remote) into domain_user.

    Mirrors User::merge: scalar fields (email, username, native_language,
    telegram_user_id, daily_load, reminders_enabled) follow LWW by updated_at.
    imported_sets is unioned. jlpt_progress and known_vocab_hash are taken
    from the newer row — they're recomputed client-side from knowledge_set.
    """
    user_updated = parse_timestamp(user_row.get("updated_at"))
    domain_updated = parse_timestamp(domain_row.get("updated_at"))

    # Default: keep domain_user values (current source of truth for new client)
    merged = copy.deepcopy(domain_row)

    # If user row is newer → take scalar fields from it
    if user_updated and (not domain_updated or user_updated >= domain_updated):
        for field in (
            "username",
            "native_language",
            "telegram_user_id",
            "daily_load",
            "reminders_enabled",
            "current_japanese_level",
            "jlpt_progress",
            "known_vocab_hash",
        ):
            if field in user_row:
                merged[field] = user_row[field]

    # imported_sets: union (stored as JSON array string)
    merged["imported_sets"] = _union_imported_sets(
        domain_row.get("imported_sets"), user_row.get("imported_sets")
    )

    # trailbase_id: keep domain_user's (it's the RLS identity for new client)
    # updated_at: set to max of both
    if user_updated and domain_updated:
        merged["updated_at"] = max(user_updated, domain_updated).isoformat()
    elif user_updated:
        merged["updated_at"] = user_updated.isoformat()

    return merged


def _union_imported_sets(
    domain_json: str | None, user_json: str | None
) -> str:
    """Union two imported_sets JSON arrays."""
    domain_sets = set()
    user_sets = set()

    if domain_json:
        try:
            domain_sets = set(json.loads(domain_json))
        except (json.JSONDecodeError, TypeError):
            pass

    if user_json:
        try:
            user_sets = set(json.loads(user_json))
        except (json.JSONDecodeError, TypeError):
            pass

    return json.dumps(sorted(domain_sets | user_sets))


# ─── Per-user sync ────────────────────────────────────────────────────


def sync_user_to_domain(
    user_row: dict[str, Any],
    domain_row: dict[str, Any] | None,
    apply: bool,  # noqa: A002
) -> dict[str, Any]:
    """Sync one user record into domain_user. Returns a summary dict."""

    # Case B: user only in `user` table → create in domain_user
    if domain_row is None:
        return _create_domain_from_user(user_row, apply)

    # Case A: both exist → merge
    return _merge_user_into_domain(user_row, domain_row, apply)


def _create_domain_from_user(
    user_row: dict[str, Any], apply: bool  # noqa: A002
) -> dict[str, Any]:
    """Create a new domain_user row from a user row (full copy + migration)."""
    email = user_row.get("email", "?")

    # Build new row — copy all fields from user
    new_row = copy.deepcopy(user_row)
    # Remove the DB primary key (let TrailBase auto-assign)
    new_row.pop("id", None)

    # Convert knowledge_set: reviews → counters
    ks_raw = new_row.get("knowledge_set")
    if ks_raw and ks_raw != '{"study_cards":{},"lesson_history":[]}':
        ks = decode_knowledge_set(ks_raw)
        ks, migrated = migrate_knowledge_set(ks)
        new_ks_raw = encode_knowledge_set(ks)
        old_size = len(ks_raw)
        new_size = len(new_ks_raw)
    else:
        migrated = 0
        old_size = len(ks_raw) if ks_raw else 0
        new_size = old_size
        new_ks_raw = ks_raw

    new_row["knowledge_set"] = new_ks_raw

    print(
        f"  [{email}] CREATE in domain_user: "
        f"{migrated} cards migrated, {old_size} → {new_size} bytes"
    )

    if not apply:
        print("    DRY RUN — not writing")
        return {"action": "create", "email": email, "migrated": migrated}

    # POST create
    create_url = f"{_BASE_URL}/api/records/v1/{_TABLE_DOMAIN}"
    resp = _SESSION.post(create_url, json=new_row)
    resp.raise_for_status()
    print(f"    Created ✓ (id={resp.json().get('id', '?')})")
    return {"action": "create", "email": email, "migrated": migrated, "id": resp.json().get("id")}


def _merge_user_into_domain(
    user_row: dict[str, Any],
    domain_row: dict[str, Any],
    apply: bool,  # noqa: A002
) -> dict[str, Any]:
    """Merge user data into existing domain_user row."""
    email = user_row.get("email", "?")
    record_id = domain_row.get("id")

    # Decode both knowledge_sets
    user_ks_raw = user_row.get("knowledge_set") or ""
    domain_ks_raw = domain_row.get("knowledge_set") or ""

    if not user_ks_raw or user_ks_raw == '{"study_cards":{},"lesson_history":[]}':
        print(f"  [{email}] No data in user table, skipping")
        return {"action": "skip", "email": email, "reason": "empty user ks"}

    user_ks = decode_knowledge_set(user_ks_raw)

    # Convert user's reviews → counters first (old format → new format)
    user_ks, migrated = migrate_knowledge_set(user_ks)

    if not domain_ks_raw or domain_ks_raw == '{"study_cards":{},"lesson_history":[]}':
        # domain_user is empty — just take user's migrated KS
        merged_ks = user_ks
        user_card_count = len(user_ks.get("study_cards", {}))
        print(f"  [{email}] domain_user empty, importing {user_card_count} cards")
    else:
        domain_ks = decode_knowledge_set(domain_ks_raw)
        # Merge: domain_user is "local" (current), user is "remote" (incoming)
        merged_ks = merge_knowledge_sets(domain_ks, user_ks)
        domain_card_count = len(domain_ks.get("study_cards", {}))
        user_card_count = len(user_ks.get("study_cards", {}))
        merged_card_count = len(merged_ks.get("study_cards", {}))
        print(
            f"  [{email}] MERGE: domain={domain_card_count} cards, "
            f"user={user_card_count} cards → {merged_card_count} cards"
        )

    new_ks_raw = encode_knowledge_set(merged_ks)

    # Merge scalar fields
    merged_row = merge_scalar_fields(user_row, domain_row)
    merged_row["knowledge_set"] = new_ks_raw

    if not apply:
        print("    DRY RUN — not writing")
        return {"action": "update", "email": email, "migrated": migrated}

    # PATCH update
    update_url = f"{_BASE_URL}/api/records/v1/{_TABLE_DOMAIN}/{record_id}"
    resp = _SESSION.patch(update_url, json=merged_row)
    resp.raise_for_status()
    print("    Written ✓")
    return {"action": "update", "email": email, "migrated": migrated}


# ─── Main ─────────────────────────────────────────────────────────────

# Module-level globals set in main() — used by helper functions above.
_BASE_URL: str = ""
_SESSION: requests.Session = requests.Session()  # type: ignore[assignment]
_TABLE_DOMAIN: str = "domain_user"
_TABLE_SOURCE: str = "user"


def main() -> None:
    global _BASE_URL, _SESSION, _TABLE_DOMAIN, _TABLE_SOURCE

    parser = argparse.ArgumentParser(
        description="Sync users from `user` table into `domain_user` table"
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Apply changes (default: dry-run, show what would change)",
    )
    parser.add_argument(
        "--source-table",
        default="user",
        help="Source table name (default: user)",
    )
    parser.add_argument(
        "--target-table",
        default="domain_user",
        help="Target table name (default: domain_user)",
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

    _BASE_URL = base_url
    _TABLE_SOURCE = args.source_table
    _TABLE_DOMAIN = args.target_table

    _SESSION = requests.Session()
    _SESSION.headers.update(
        {
            "Authorization": f"Bearer {admin_token}",
            "Content-Type": "application/json",
        }
    )

    print(f"Mode: {'APPLY' if args.apply else 'DRY RUN'}")
    print(f"Source: {args.source_table} → Target: {args.target_table}")
    print()

    # Fetch all records from both tables
    print(f"Fetching {args.source_table}...")
    user_records = fetch_all_records(_SESSION, base_url, args.source_table)
    print(f"  {len(user_records)} records")

    print(f"Fetching {args.target_table}...")
    domain_records = fetch_all_records(_SESSION, base_url, args.target_table)
    print(f"  {len(domain_records)} records")
    print()

    # Index domain_user by email
    domain_by_email: dict[str, dict[str, Any]] = {}
    for row in domain_records:
        email = row.get("email")
        if email:
            domain_by_email[email] = row

    # Process each user
    results: list[dict[str, Any]] = []
    for user_row in user_records:
        email = user_row.get("email", "?")
        domain_row = domain_by_email.get(email)
        result = sync_user_to_domain(user_row, domain_row, args.apply)
        results.append(result)

    # Summary
    created = sum(1 for r in results if r["action"] == "create")
    updated = sum(1 for r in results if r["action"] == "update")
    skipped = sum(1 for r in results if r["action"] == "skip")

    print()
    print(f"Done. Created: {created}, Updated: {updated}, Skipped: {skipped}")
    if not args.apply:
        print("(dry run — no changes written)")


if __name__ == "__main__":
    main()
