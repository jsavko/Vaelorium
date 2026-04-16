# Restore: replay journal tail on top of snapshot

**Date:** 2026-04-16

## Summary

Restoring a Tome from backup now replays the journal tail on top of the snapshot, so the user sees fully up-to-date data immediately. Previously, restored Tomes only contained the state at snapshot time — often missing edits made since the last automatic snapshot (triggered by 5 MB / 5k ops thresholds).

## Changes

### Bug Fixes

- **Snapshot cursor preservation** (`snapshot.rs`): `take_snapshot` now preserves `last_applied_op_id`, `last_uploaded_op_id`, and `last_snapshot_id` in the snapshot's `sync_state` table instead of deleting the entire row. This lets a restoring device know where the snapshot's state ends.
- **Engine cursor flush** (`engine.rs`): `sync_tome_once` now saves the runtime state to the DB *before* `take_snapshot`, so the VACUUM captures the current cursor values. Previously, the cursor was updated in-memory only and persisted *after* the snapshot, meaning all snapshots had `last_applied_op_id = NULL`.
- **Journal replay during restore** (`restore.rs`): `backup_restore_tome` now opens the staged snapshot DB, renames the cursor row from the `__snapshot__` sentinel to the real `tome_uuid`, and calls `sync_tome_once` to replay the journal tail. Errors are logged as warnings — the restore still succeeds with snapshot data.

### UI

- Restore button text changed from "Restoring…" to "Restoring & syncing…" to set expectations
- `RestoredTome` now carries an optional `warning` field; if journal replay fails, a toast is shown: "Tome restored, but some recent changes may still be pending. Open it and sync to finish."

### Backward Compatibility

- Old snapshots (without cursor) degrade gracefully: `SyncRuntimeState::load` returns `None` for all cursors, causing `sync_tome_once` to download the full journal — identical behavior to a no-cursor approach, just less efficient.

## Files Modified

- `src-tauri/src/sync/snapshot.rs` — preserve cursor fields in snapshot instead of `DELETE FROM sync_state`
- `src-tauri/src/sync/engine.rs` — flush state.save() before take_snapshot
- `src-tauri/src/commands/backup/restore.rs` — inline journal replay after snapshot restore
- `src-tauri/src/commands/backup/mod.rs` — add `warning` field to `RestoredTome`
- `src-tauri/tests/sync_integration.rs` — 4 new integration tests for cursor preservation and replay
- `src/lib/api/backup.ts` — add `warning` field to TypeScript types
- `src/lib/components/TomePicker.svelte` — updated button text + warning toast
