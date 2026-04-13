-- M7 Sync — local activity log for "did my last sync actually run?" UI.
-- Per-device, never synced; follows the same local-only convention as
-- sync_journal_local / sync_state / sync_conflicts.

CREATE TABLE sync_activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tome_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    ops_uploaded INTEGER NOT NULL DEFAULT 0,
    ops_applied INTEGER NOT NULL DEFAULT 0,
    conflicts_created INTEGER NOT NULL DEFAULT 0,
    snapshot_taken TEXT,
    outcome TEXT NOT NULL,
    error TEXT
);

CREATE INDEX idx_sync_activity_tome_id ON sync_activity(tome_id, id DESC);
