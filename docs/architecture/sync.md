# Sync engine

## Responsibility
Incremental sync of per-Tome SQLite state to a pluggable backup backend (filesystem / S3 / hosted cloud). Snapshot + encrypted-delta-journal protocol, client-side ChaCha20-Poly1305 E2EE, CAS on `sync-meta.json`. M7 shipped in v0.2.0.

## Entry points
- `sync::engine::sync_tome_once` (`src-tauri/src/sync/engine.rs`) — one sync cycle (pull → apply → push → prune). The runner calls this; `sync_now` command also does.
- `sync::runner::start` (`src-tauri/src/sync/runner.rs`) — spawns the background loop (`POLL_INTERVAL = 10min` + nudges on Tome open/close/enable). Publishes `SyncStatusEvent` for the frontend pill.
- `sync::registry::TABLES` (`src-tauri/src/sync/registry.rs`) — authoritative list of sync-tracked tables. **Add a new table here first**, then emit_for_row from its mutation paths.
- `sync::journal::emit_for_row` / `record_op` (`src-tauri/src/sync/journal.rs`) — every mutation that should sync calls this.
- `sync::state::SyncConfig` (`src-tauri/src/sync/state.rs`) — per-Tome sync row in SQLite (`enabled`, `device_id`).
- `sync::tome_identity::get_or_create_uuid` (`src-tauri/src/sync/tome_identity.rs`) — stable per-Tome UUID used as bucket prefix `tomes/<uuid>/`.

## Commands (`src-tauri/src/commands/sync.rs`)
- `sync_enable` / `sync_disable` — toggle `sync_config.enabled`. Also mirrors onto `recent_tomes.sync_enabled` (see tomes.md).
- `sync_now` — ad-hoc `sync_tome_once` invocation.
- `sync_status` — returns `SyncStatusPayload` (backend, queue size, conflicts, last error).
- `list_conflicts` / `resolve_conflict` — surfaces `sync::conflict::Conflict` to `ConflictResolver.svelte`.
- `take_snapshot` — forces a new snapshot outside normal cadence.

## Data flow
1. Mutation command (e.g. `pages::update_page`) opens a tx, writes row, calls `journal::emit_for_row(&mut *tx, &TABLES.pages, &op)`, commits.
2. `session.nudge()` wakes the runner.
3. Runner loads `pending_ops`, encrypts, `PUT`s delta to backend, updates `sync-meta.json` via CAS.
4. On pull: decrypt peer deltas, call `engine::apply_op_via_schema` to upsert into local DB.
5. Conflict detection runs per-op; losers land in `sync_conflicts` table.

## Special apply paths
- **`page_content`** (BLOB column) — custom path in `engine.rs`; can't go through generic schema.
- **`page_tags`** (M:N pivot) — composite `row_id = page_id|tag_id`.

## Gotchas
- Tauri `invoke()` args must be camelCase (see `feedback_tauri_camelcase` memory).
- New SQL migration needs registration in both `db/migrations.rs` AND every test harness list (`feedback_register_migrations_in_both_runners`).
- Meta-field denylist per table — `updated_at` etc. are bookkeeping, not conflict-relevant (`feedback_sync_meta_fields`).
- Restored snapshots already have schema; don't re-run migrations (`feedback_snapshot_no_migrations`).
- Don't spawn from the Tauri setup hook with `tokio::spawn` — use `tauri::async_runtime::spawn` (`feedback_tauri_async_spawn`).

## Intentionally NOT synced
`wiki_links` (derived), `versions` (large; local-only), `images` (binary blob deferred), `relation_types` (built-ins dominate). Don't relitigate without strong reason.

## See also
- `project_m7_sync` auto-memory — locked protocol decisions.
- `docs/sync-user-guide.md` — end-user documentation.
- `docs/sync-s3-testing.md` — Minio recipe for S3 backend manual tests.

## Where NOT to look
- `src/lib/editor/YjsProvider.ts` — Y.js is for local realtime editor state, not sync.
- `src-tauri/src/commands/export.rs` — one-shot JSON/Markdown export, unrelated to sync.
