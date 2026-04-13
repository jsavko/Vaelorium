# Phase 5.1 — Sync Recovery Flow

**Date:** 2026-04-13
**Plan:** `.claude/plans/archive/2026-04-13-m7-sync-phase5-1-recovery.md`

## Summary

Wired the existing `restore_snapshot_to_file` engine primitive into a user-facing recovery path: a fresh device can now configure Backup with the same backend + passphrase as another device, see all Tomes available on that backend, and restore one with a click — no more physically copying `.tome` files between machines.

## Changes

### Features
- `sync::snapshot::list_tome_snapshots(raw_backend)` — discovers `tomes/<uuid>/snapshots/*.snap.enc` keys, groups by `tome_uuid`, returns the latest snapshot per Tome with size + timestamp.
- `sync::snapshot::restore_snapshot_by_key` — restores a snapshot by full backend key (used for cross-Tome discovery; complements the existing per-Tome `restore_snapshot_to_file`).
- `backup_list_restorable_tomes` Tauri command — discovers + peeks each snapshot's `tome_metadata` to surface the display name and description.
- `backup_restore_tome` Tauri command — restores a chosen Tome to `<app_data>/restored/<safe-name>.tome`, registers in recent Tomes, returns the path so the UI can immediately invoke `open_tome`.
- TomePicker "Restore from backup" panel — visible only when Backup is configured. Locked-state shows a deep-link to Settings → Backup; unlocked-state lists restorable Tomes with size, snapshot age, Restore button. Filename collisions are handled with `(N)` suffixes.

### Files Modified
- `src-tauri/src/sync/snapshot.rs` — `TomeSnapshotSummary`, `list_tome_snapshots`, `restore_snapshot_by_key`, two unit tests
- `src-tauri/src/commands/backup.rs` — `RestorableTome`, `RestoredTome`, `backup_list_restorable_tomes`, `backup_restore_tome`, `peek_tome_metadata`, `sanitize_filename`
- `src-tauri/src/lib.rs` — register two new commands
- `src-tauri/tests/sync_integration.rs` — Scenario G (multi-Tome recovery discovery)
- `src/lib/api/backup.ts` — `RestorableTome`, `RestoredTome`, `listRestorableTomes`, `restoreTomeFromBackup`; bug fix on `getBackupStatus` web fallback
- `src/lib/components/TomePicker.svelte` — "Restore from backup" panel + props for `onOpenSettings`
- `src/App.svelte` — pass `onOpenSettings` to TomePicker; render `<Settings>` in both branches so it's reachable from the picker
- `docs/sync-manual-testing.md` — multi-device restore walkthrough + conflict rehearsal steps

## Rationale
Without this flow the only way to bootstrap a Tome on a second machine was to physically copy the `.tome` file. That made testing the conflict-resolution UX painful and would have shipped a half-baked sync story for v0.2.0. The engine primitives existed since Phase 2 — this is purely the wiring that turns them into a UX.

## Test evidence
- 55 library unit tests passing (2 new in `snapshot::tests`)
- 12 sync integration scenarios passing (Scenario G new — multi-Tome discovery + selective restore)
- Svelte type-check clean

## Notes for testing
The restored Tome has no `sync_config` row (snapshots intentionally strip sync state). To resume sync from device B, the user opens Settings → Sync → Enable. Future Phase 5.5 wizard will offer to do this automatically.
