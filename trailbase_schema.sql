-- TrailBase `user` table schema for Origa.
-- Reference for the table as it exists in production.
-- Apply via TrailBase SQL Editor (/_/admin/editor).

CREATE TABLE user (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trailbase_id BLOB UNIQUE NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    native_language INTEGER NOT NULL DEFAULT 0,
    jlpt_progress TEXT CHECK(json_valid(jlpt_progress)),
    current_japanese_level INTEGER,
    duolingo_jwt_token TEXT,
    telegram_user_id INTEGER,
    reminders_enabled INTEGER NOT NULL DEFAULT 0,
    -- knowledge_set holds a compressed (deflate + base64) wire blob produced by
    -- origa_ui/src/repository/knowledge_set_codec.rs. Its value is intentionally
    -- NOT valid JSON, so this column MUST NOT carry a CHECK(json_valid(...))
    -- constraint: doing so rejects every save_sync with a CHECK-constraint
    -- violation (HTTP 500). Data integrity is enforced client-side by the
    -- codec's read-recover policy (corrupt remote -> empty -> self-heal via
    -- local overwrite). See ADR-034.
    knowledge_set TEXT NOT NULL DEFAULT '{"study_cards":{},"lesson_history":[]}',
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    imported_sets TEXT CHECK(json_valid(imported_sets)) NOT NULL DEFAULT '[]',
    daily_load INTEGER DEFAULT 1
) STRICT;

-- Index for faster lookups by trailbase_id
CREATE INDEX idx_user_trailbase_id ON user(trailbase_id);

-- Index for faster lookups by email
CREATE INDEX idx_user_email ON user(email);

-- _ROW_.trailbase_id = _USER_.id
-- _REQ_.trailbase_id = _USER_.id
-- _ROW_.trailbase_id = _USER_.id AND _REQ_.trailbase_id = _USER_.id
-- _ROW_.trailbase_id = _USER_.id
