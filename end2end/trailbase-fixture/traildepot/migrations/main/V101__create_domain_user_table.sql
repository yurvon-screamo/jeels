CREATE TABLE IF NOT EXISTS "domain_user" (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trailbase_id TEXT NOT NULL,
    username TEXT NOT NULL DEFAULT '',
    email TEXT NOT NULL UNIQUE,
    native_language INTEGER NOT NULL DEFAULT 0,
    jlpt_progress TEXT,
    current_japanese_level INTEGER,
    telegram_user_id INTEGER,
    knowledge_set TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    imported_sets TEXT,
    daily_load INTEGER,
    known_vocab_hash INTEGER DEFAULT 0,
    _owner TEXT
) STRICT;
