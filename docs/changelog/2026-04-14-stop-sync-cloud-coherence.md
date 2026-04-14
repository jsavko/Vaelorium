# Stop-sync / cloud-copy coherence + UUID-aware quota

**Date:** 2026-04-14

## Summary

Stopping sync on a Tome no longer leaves an orphaned cloud copy with no UI affordance to manage it. The Tome's cloud badge drops from the TomePicker card, the Tome re-appears in the "Restore from backup" list, and the existing trash button deletes the cloud copy. Additionally, re-enabling sync on a Tome whose UUID is already on the backend no longer trips the preemptive Tome-limit banner — it's an idempotent operation from the cloud's ledger.

## Changes

### Features
- **Cloud badge reflects "actively syncing"**, not "snapshot exists on backend". After `sync_disable` the badge disappears.
- **Stop-synced Tomes re-surface in the restore list.** TomePicker filters the restore panel by "locally syncing" UUIDs rather than "any local presence", so orphaned cloud copies are visible and deletable.
- **UUID-aware quota banner.** The Settings → Sync enable-sync CTA no longer warns / disables when the Tome's UUID is already present on the backend — re-enabling is idempotent.

### Files Modified
- `src-tauri/src/app_state.rs` — `RecentTome.sync_enabled: bool` (with `#[serde(default)]`); `add_recent_tome` signature extended; new `set_sync_enabled_for_path` helper for in-session toggling.
- `src-tauri/src/commands/tomes.rs` — `open_tome` probes `SyncConfig::load` to seed `sync_enabled`; `create_tome` defaults to `false`.
- `src-tauri/src/commands/sync.rs` — `sync_enable` / `sync_disable` mirror state into the recent_tomes registry.
- `src-tauri/src/commands/backup.rs` — restore-from-backup path seeds `sync_enabled=true` (restore implies sync).
- `src-tauri/src/lib.rs` — legacy-migration path passes `sync_enabled=false`.
- `src/lib/api/tomes.ts` — `RecentTome.sync_enabled: boolean`.
- `src/lib/components/TomePicker.svelte` — `syncingUuids` derivation; badge + restore-filter keyed off it.
- `src/lib/components/Settings.svelte` — `currentTomeAtQuota` derivation folds in `$restorableTomes` so UUID re-enable bypasses the banner.

### Rationale

The cloud badge's previous semantics ("snapshot exists on backend") lied after stop-sync: the user sees a cloud icon but the Tome isn't actually syncing anymore. Aligning badge + restore list around `sync_enabled` gives one coherent rule: a Tome is either actively syncing (badge, not in restore list) or has an orphaned backend copy (no badge, available in restore list to delete or restore). The UUID-aware quota check closes a related papercut — users who stop-synced a Tome and want to re-enable it shouldn't be falsely rejected as a new-Tome slot.
