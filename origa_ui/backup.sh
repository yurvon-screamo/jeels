#!/bin/sh
# TrailBase SQLite → S3 backup with GFS (Grandfather-Father-Son) rotation.
#
# Retention (always exactly 6 main.db snapshots + 1 latest logs.db):
#   snapshot-1.db.gz       — most recent  (≈12h ago)
#   snapshot-2.db.gz       — 24h ago
#   snapshot-3.db.gz       — 36h ago
#   weekly-current.db.gz   — this week (Monday 03:00 UTC)
#   weekly-previous.db.gz  — last week
#   monthly.db.gz          — this month (1st, 03:00 UTC)
#   logs-latest.db.gz      — latest logs.db (overwritten each run)
set -eu

# --- Configuration ---
DATA_DIR="${TRAILBASE_DATA_DIR:-/app/traildepot/data}"
BACKUP_BUCKET_PATH="${BACKUP_BUCKET_PATH:-trailbase}"
S3_BASE="s3://${AWS_S3_BUCKET_NAME}/${BACKUP_BUCKET_PATH}"
TMP_DIR="${TMPDIR:-/tmp}"

export AWS_PAGER=""

# --- S3 helper (always uses the configured endpoint) ---
s3() {
    aws s3 "$@" --endpoint-url "${AWS_ENDPOINT_URL}"
}

log() {
    printf '[backup %s] %s\n' "$(date -u '+%Y-%m-%dT%H:%M:%SZ')" "$*"
}

# --- Bail out if database doesn't exist yet ---
if [ ! -f "${DATA_DIR}/main.db" ]; then
    log "main.db not found — nothing to back up."
    exit 0
fi

# --- Determine rotation scope from current date (UTC) ---
CURRENT_HOUR=$(date -u +%H)
CURRENT_DOW=$(date -u +%u)   # 1 = Monday
CURRENT_DOM=$(date -u +%-d)  # day of month, no leading zero

DO_WEEKLY=0
DO_MONTHLY=0

# Weekly rotation: Monday 03:00 UTC
if [ "${CURRENT_DOW}" = "1" ] && [ "${CURRENT_HOUR}" = "03" ]; then
    DO_WEEKLY=1
fi

# Monthly rotation: 1st of month 03:00 UTC
if [ "${CURRENT_DOM}" = "1" ] && [ "${CURRENT_HOUR}" = "03" ]; then
    DO_MONTHLY=1
fi

# --- Step 1: Online SQLite backup (safe while server is running) ---
log "Backing up main.db..."
sqlite3 "${DATA_DIR}/main.db" ".backup '${TMP_DIR}/main.db'"
gzip -f "${TMP_DIR}/main.db"

HAS_LOGS=0
if [ -f "${DATA_DIR}/logs.db" ]; then
    log "Backing up logs.db..."
    sqlite3 "${DATA_DIR}/logs.db" ".backup '${TMP_DIR}/logs.db'"
    gzip -f "${TMP_DIR}/logs.db"
    HAS_LOGS=1
fi

# --- Step 2: Snapshot rotation (shift: 3 deleted, 2→3, 1→2, new→1) ---
log "Rotating snapshots..."
s3 rm "${S3_BASE}/snapshot-3.db.gz" 2>/dev/null || true
s3 mv "${S3_BASE}/snapshot-2.db.gz" "${S3_BASE}/snapshot-3.db.gz" 2>/dev/null || true
s3 mv "${S3_BASE}/snapshot-1.db.gz" "${S3_BASE}/snapshot-2.db.gz" 2>/dev/null || true
s3 cp "${TMP_DIR}/main.db.gz" "${S3_BASE}/snapshot-1.db.gz"

# --- Step 3: logs.db (latest only, overwrite each run) ---
if [ "${HAS_LOGS}" = "1" ]; then
    log "Uploading logs-latest..."
    s3 cp "${TMP_DIR}/logs.db.gz" "${S3_BASE}/logs-latest.db.gz"
fi

# --- Step 4: Weekly rotation (Monday 03:00 UTC) ---
if [ "${DO_WEEKLY}" = "1" ]; then
    log "Weekly rotation..."
    s3 mv "${S3_BASE}/weekly-current.db.gz" "${S3_BASE}/weekly-previous.db.gz" 2>/dev/null || true
    s3 cp "${S3_BASE}/snapshot-1.db.gz" "${S3_BASE}/weekly-current.db.gz"
fi

# --- Step 5: Monthly overwrite (1st of month 03:00 UTC) ---
if [ "${DO_MONTHLY}" = "1" ]; then
    log "Monthly rotation..."
    s3 cp "${S3_BASE}/snapshot-1.db.gz" "${S3_BASE}/monthly.db.gz"
fi

# --- Cleanup ---
rm -f "${TMP_DIR}/main.db.gz" "${TMP_DIR}/logs.db.gz"
log "Backup completed."
