-- device_name moves to app-global scope (sync-backend.json).
-- device_id stays per-Tome so journal::active_sync_session can read it
-- from the pool without needing SessionState, but it's seeded from the
-- app-global id at sync_enable time so all Tomes on one device share it.

ALTER TABLE sync_config DROP COLUMN device_name;
