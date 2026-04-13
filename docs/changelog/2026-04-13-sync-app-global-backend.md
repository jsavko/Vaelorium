# Sync: app-global backup destination

**Date:** 2026-04-13

## Summary

Sync backend configuration moves from per-Tome to **app-global**. One backup destination (S3 or filesystem folder) and one passphrase are configured once in **Settings → Backup**; each Tome opts into syncing with a single toggle in **Settings → Sync**. Per-Tome data is namespaced under `tomes/{tome_id}/` in the shared backend, so multiple Tomes coexist without conflict.

User-visible UX: configuring a second Tome no longer requires re-entering S3 creds + a new passphrase. The salt-sharing fix shipped earlier today (`095a3b0`) is naturally subsumed — the bucket-root `sync-meta.json` now identifies the *backend* (one salt for everyone) rather than a single Tome.

## Changes

### Features
- New **Backup** tab in Settings: connect a backend + passphrase once for the whole app; disconnect to wipe the local config and keychain entry.
- Simplified **Sync** tab: when backup is configured, just a "Back up this Tome" toggle plus per-Tome status (last sync, queue size, conflicts).
- Sidebar pill gets a new `backup-missing` state ("No backup") so you can see at a glance when no backup is configured.
- Backup auto-unlock via OS keychain on app launch (single keychain entry, key = `"backup"`).

### Refactoring
- Backend keys for per-Tome data prefixed `tomes/{tome_id}/` via new `PrefixedBackend` wrapper. Engine code stays prefix-agnostic.
- `sync_config` schema simplified: dropped `backend_type`, `backend_config`, `passphrase_salt`, `schema_version` columns. Backend creds + salt now live in `<app_data_dir>/sync-backend.json`.
- `SessionState` tracks one app-global key, not per-Tome.
- Runner iterates every enabled Tome each tick instead of single-Tome loop.
- `RemoteMeta` (`sync-meta.json`) drops its `tome_id` field — it identifies the backend, not a Tome (version bumped 1 → 2).
- New Tauri commands: `backup_configure`, `backup_disconnect`, `backup_status`, `backup_unlock`, `backup_try_auto_unlock`. Removed: `sync_unlock`, `sync_try_auto_unlock`. `sync_enable` parameter list trims to `{ tome_id, device_name? }`.

### Migration
- Migration `010_sync_app_global.sql` wipes existing `sync_config` rows and drops the obsolete columns. Acceptable per user direction (only the user is testing M7 today).
- Existing buckets keep their root-level `snapshots/`, `journal/`, `sync-meta.json` keys; they're orphaned under the new prefix layout. Re-configuring the backup re-initializes under `tomes/{id}/`.

## Files Modified

### Rust — new
- `src-tauri/migrations/010_sync_app_global.sql`
- `src-tauri/src/sync/app_backend.rs` — `AppBackendConfig` load/save
- `src-tauri/src/sync/backend/prefixed.rs` — `PrefixedBackend` + `tome_prefix()`
- `src-tauri/src/commands/backup.rs` — app-global backup commands

### Rust — modified
- `src-tauri/src/sync/state.rs` — slim `SyncConfig`, `list_all()`
- `src-tauri/src/sync/session.rs` — single-key session
- `src-tauri/src/sync/keychain.rs` — single `(SERVICE, "backup")` entry
- `src-tauri/src/sync/remote_meta.rs` — drop `tome_id`
- `src-tauri/src/sync/runner.rs` — iterate enabled Tomes, prefixed backend
- `src-tauri/src/sync/mod.rs` / `src-tauri/src/sync/backend/mod.rs` — register modules
- `src-tauri/src/commands/sync.rs` — slim per-Tome commands, `build_tome_backend` helper
- `src-tauri/src/commands/mod.rs` / `src-tauri/src/lib.rs` — register backup commands
- `src-tauri/tests/sync_integration.rs` / `src-tauri/tests/sync_s3_smoke.rs` — new SyncConfig shape, apply 010

### Frontend — new
- `src/lib/api/backup.ts`

### Frontend — modified
- `src/lib/api/sync.ts` — slim `enableSync`, add `backupMissing` to `SyncStatus`, drop unlock helpers
- `src/lib/stores/syncStore.ts` — separate `backupStatus` store, derived indicator gains `backup-missing`
- `src/lib/components/Settings.svelte` — Backup tab + simplified Sync tab
- `src/lib/components/Sidebar.svelte` — `No backup` pill state, drop unused `onOpenUnlock`
- `src/App.svelte` — drop `SyncUnlockModal` (Backup tab handles unlock)

### Frontend — deleted
- `src/lib/components/SyncUnlockModal.svelte` — superseded by Backup tab passphrase prompt

## Tests

- 50 sync unit tests (added `app_backend::*` ×3, `prefixed::*` ×3)
- 11 sync integration tests
- 0 svelte-check errors / 0 warnings
