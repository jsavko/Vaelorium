-- M7 Sync — local-side tables.
-- Backend stores its own ciphertext blobs (snapshots/journal/conflicts/meta).
-- These tables track per-Tome sync configuration, runtime state, the local
-- pending-upload queue, and any unresolved conflicts.

-- Per-Tome sync configuration.
CREATE TABLE IF NOT EXISTS sync_config (
    tome_id            TEXT PRIMARY KEY,
    enabled            INTEGER NOT NULL DEFAULT 0,
    backend_type       TEXT    NOT NULL,
    backend_config     TEXT    NOT NULL,
    passphrase_salt    BLOB    NOT NULL,
    device_id          TEXT    NOT NULL,
    device_name        TEXT    NOT NULL,
    schema_version     INTEGER NOT NULL DEFAULT 1,
    created_at         TEXT    NOT NULL,
    updated_at         TEXT    NOT NULL
);

-- Per-Tome runtime sync state. Tracks where we are in the journal exchange.
CREATE TABLE IF NOT EXISTS sync_state (
    tome_id                 TEXT PRIMARY KEY,
    last_uploaded_op_id     TEXT,
    last_applied_op_id      TEXT,
    last_snapshot_id        TEXT,
    last_sync_at            TEXT,
    last_error              TEXT
);

-- Local journal of mutations not yet uploaded.
-- Each row is one typed Op (see sync::journal::Op).
CREATE TABLE IF NOT EXISTS sync_journal_local (
    op_id              TEXT PRIMARY KEY,
    tome_id            TEXT NOT NULL,
    transaction_id     TEXT NOT NULL,
    table_name         TEXT NOT NULL,
    row_id             TEXT NOT NULL,
    op_kind            TEXT NOT NULL,
    op_data            TEXT NOT NULL,
    schema_version     INTEGER NOT NULL,
    timestamp          TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_journal_local_tome
    ON sync_journal_local(tome_id);
CREATE INDEX IF NOT EXISTS idx_sync_journal_local_tx
    ON sync_journal_local(tome_id, transaction_id);

-- Unresolved conflicts requiring user action via the inline ConflictResolver UI.
-- One row per conflicted (table, row, field) tuple. Cleared when the user picks
-- a side and the resolution op is emitted.
CREATE TABLE IF NOT EXISTS sync_conflicts (
    conflict_id        TEXT PRIMARY KEY,
    tome_id            TEXT NOT NULL,
    table_name         TEXT NOT NULL,
    row_id             TEXT NOT NULL,
    field_name         TEXT NOT NULL,
    local_value        TEXT,
    remote_value       TEXT,
    local_op_id        TEXT NOT NULL,
    remote_op_id       TEXT NOT NULL,
    detected_at        TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sync_conflicts_tome
    ON sync_conflicts(tome_id);
CREATE INDEX IF NOT EXISTS idx_sync_conflicts_row
    ON sync_conflicts(tome_id, table_name, row_id);
