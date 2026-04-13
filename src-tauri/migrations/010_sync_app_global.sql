-- M7 Sync — move backend config + passphrase salt to app-global scope.
-- Per-Tome sync_config keeps only enabled flag + device identity; backend
-- credentials live in <app_data_dir>/sync-backend.json, and the passphrase
-- in the OS keychain under one entry. Per-Tome data is namespaced under
-- tomes/{tome_id}/ in the shared backend.
--
-- Existing rows reference the old per-Tome backend layout (root-level
-- snapshots/, journal/) and become invalid under the new prefix layout.
-- Wipe them; user re-enables sync after reconfiguring the backup backend.

DELETE FROM sync_config;

ALTER TABLE sync_config DROP COLUMN backend_type;
ALTER TABLE sync_config DROP COLUMN backend_config;
ALTER TABLE sync_config DROP COLUMN passphrase_salt;
ALTER TABLE sync_config DROP COLUMN schema_version;
