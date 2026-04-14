# Tomes & registry

## Responsibility
A **Tome** is a `.tome` file = a self-contained SQLite DB for one worldbuilding project. This subsystem owns: Tome file lifecycle (create/open/close), the per-Tome `tome_metadata` kv table, the stable `tome_uuid` that identifies a Tome across devices, and the app-global `recent_tomes` registry that powers TomePicker.

## Entry points

### Rust
- `commands::tomes::{create_tome, open_tome, close_tome}` (`src-tauri/src/commands/tomes.rs`) — Tome file lifecycle.
- `commands::tomes::{get_tome_metadata, update_tome_metadata}` — kv store for name / description / cover_image / created_at.
- `app_state::{AppState, RecentTome}` (`src-tauri/src/app_state.rs`) — app-global registry persisted to `app_state.json`. Carries `path`, `name`, `description`, `last_opened`, `tome_uuid`, `sync_enabled`.
- `app_state::add_recent_tome` / `set_sync_enabled_for_path` — registry mutation helpers.
- `sync::tome_identity::get_or_create_uuid` (`src-tauri/src/sync/tome_identity.rs`) — lazy-creates `tome_metadata.tome_uuid`. Called at create / open / first-sync. Path-independent, same value on every device for the same Tome.

### Frontend
- `src/lib/stores/tomeStore.ts` — `currentTome`, `currentTomeMetadata`, `isTomeOpen`, `recentTomes`. Also `createTome`, `openTome`, `closeTome` helpers.
- `src/lib/api/tomes.ts` — thin command wrappers + `RecentTome` TS interface.
- `src/lib/components/TomePicker.svelte` — home screen. Renders `$recentTomes` cards + restore panel (from `$restorableTomes`).
- `src/lib/components/CreateTomeModal.svelte` — name + description + native save dialog → `createTome`.

## Identity
- `tome_metadata.tome_uuid` (TEXT, stable UUID) is the canonical identity.
- Bucket prefix on any backend is `tomes/<tome_uuid>/` — path-independent, so the same Tome restored on a new device retains its cloud lineage.
- `RecentTome.tome_uuid` is `None` for legacy (pre-M5) entries — handle that case.

## TomePicker card semantics
- Cloud badge: `tome.sync_enabled && tome.tome_uuid && backedUpUuids.has(tome.tome_uuid)` — requires BOTH "actively syncing locally" AND "snapshot present on backend". Stop-sync makes the badge drop.
- Restore list filter: excludes Tomes in `syncingUuids` (UUIDs with `sync_enabled=true` in recent_tomes). Stop-synced Tomes re-appear here so the trash button can delete the cloud copy.

## Data flow (create)
1. User in CreateTomeModal picks name + save path.
2. Frontend calls `createTome(path, name, description)`.
3. Rust opens SQLite, seeds `tome_metadata` (name, description, created_at), materializes `tome_uuid` via `get_or_create_uuid`.
4. `add_recent_tome(..., sync_enabled=false)` registers entry at front of list (dedup by path, truncate to 10).

## Gotchas
- Tauri args camelCase (`feedback_tauri_camelcase`).
- Tome-DB-dependent stores must init in an `$effect($isTomeOpen)`, not `onMount` (`feedback_init_after_tome_open`).
- Scoped state (current page, form, tome) needs explicit reset on tome transitions (`feedback_reset_on_scope_change`).
- `RecentTome` grew fields over time (`tome_uuid`, `sync_enabled`) — new fields need `#[serde(default)]` to survive old `app_state.json` files (`feedback_serde_default_for_new_fields`).
- `sync_enabled` flag mirrors but does NOT own `sync_config.enabled`; the Tome's own SQLite is the source of truth.

## Where NOT to look
- `src-tauri/src/commands/backup.rs` — that's the backup destination, not Tome lifecycle (except `restore_tome_from_backup` which straddles both).
- `tome_metadata` is not `tome_metadata_kv` — they're the same table, just an informal naming thing.
