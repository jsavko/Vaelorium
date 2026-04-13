# M7 Sync — Phase 4a: Schema Registry + Three More Tables

**Date:** 2026-04-12

## Summary
Built the schema registry pattern that lets us add op-emission to a new table with ~10 lines of code instead of ~100. Validated by refactoring `commands/pages.rs` (zero behavior change, all tests still pass) then wiring entity_types, relations (create/delete), and tags (create). The engine's apply path is now generic via `apply_op_via_schema` — no more per-table dispatch. Maps, timelines, the M:N `page_tags` pivot, and the S3 backend are deferred to Phase 4b.

## Changes

### Schema registry & helpers
- **`sync::registry`** — new module. `TableSchema { name, columns, primary_key, meta_fields }` + static `TABLES` registry + `schema_by_name()` lookup. Currently lists 9 tables (5 wired this phase, 4 awaiting 4b).
- **`sync::journal::load_row_via_schema(executor, schema, row_id)`** — generic SELECT-and-build-field-map. Tries each common SQLite type (String/i64/f64/bool/Vec<u8>) per column; BLOBs become base64 strings.
- **`sync::journal::emit_for_row(executor, schema, row_id, kind, tx_id, before, session)`** — single helper for op emission. Builds insert/update/delete op via the registry, persists via `record_op`. No-op when sync is disabled.

### Engine generification
- **`sync::engine::apply_op_via_schema`** — replaces the per-table `apply_pages_op` / `apply_page_content_op` dispatch with one generic implementation driven by the registry. Builds `INSERT OR REPLACE`/`UPDATE`/`DELETE` SQL from `TableSchema`. Always sources the PK column from `op.row_id` (so emitters don't need to include the PK in `op.fields`).
- **`page_content` remains a special case** — the `yjs_state` BLOB column has its own apply path. Documented as the lone exception.
- **`bind_json` helper** — maps a `serde_json::Value` to a sqlx bind across String/i64/f64/Bool/Null types.

### Pages refactor (no behavior change)
- `commands/pages.rs` mutations now use `load_row_via_schema(&TABLES.pages, ...)` + `emit_for_row(&TABLES.pages, ..., OpKind::Insert/Update/Delete, ...)` instead of hand-built field maps. The 5 mutation paths (create / update / delete / save_content / reorder) are all on the registry; `save_page_content` keeps its custom path due to the BLOB.

### New table wiring
- **entity_types** — create/update/delete all wired through the registry.
- **relations** — create/delete wired. (Relation types are mostly built-in, defer.)
- **tags** — create wired. add_tag_to_page / remove_tag_from_page deferred to 4b alongside the M:N `page_tags` pivot apply path.

### Tests
- **New: `registry_entity_types_propagate`** integration test — D1 creates an entity_type via the registry helpers, syncs; D2 syncs; D2 has the row with all fields. Validates the registry-driven emit + apply round trip end-to-end.
- All 8 prior sync integration scenarios still pass — zero regression from the engine refactor.
- Backend totals: **38 unit + 9 integration tests** all green.
- Frontend totals: 44 vitest + 21 Playwright pages-touching tests still pass.

## Files Modified
- `src-tauri/src/sync/registry.rs` — new
- `src-tauri/src/sync/mod.rs` — `pub mod registry;`
- `src-tauri/src/sync/journal.rs` — `load_row_via_schema`, `emit_for_row`, internal `read_col_as_json` helper
- `src-tauri/src/sync/engine.rs` — `apply_op_via_schema`, `bind_json`; removed `apply_pages_op`
- `src-tauri/src/commands/pages.rs` — refactored to use registry helpers
- `src-tauri/src/commands/entity_types.rs` — wired (create/update/delete)
- `src-tauri/src/commands/relations.rs` — wired (create/delete)
- `src-tauri/src/commands/tags.rs` — wired (create)
- `src-tauri/tests/sync_integration.rs` — new entity_types test

## Rationale
Phase 2 hand-wired one table in ~300 LOC. Repeating that for 12 more tables in Phase 4 would have been ~1500 LOC of repetitive diffs. The registry pattern collapses each new table to ~30 LOC of mutation-wrapping + ~5 LOC of registry entry. Phase 4a's investment (one new module + two helpers + an engine refactor) earns its keep starting at the second table; the remaining ~7 tables in 4b will be straightforward applications of the same pattern.

## Discovery
**PK column must be sourced from `op.row_id`, not `op.fields`.** The integration test helper for create_page builds the op's `fields` map without including `id`. The first version of `apply_op_via_schema` bound `None` for unspecified columns, including the primary key — breaking inserts. The fix: the apply path always uses `op.row_id` for whichever column is `schema.primary_key`, regardless of what's in `op.fields`. This means emitter code can omit the PK from the field map, which is the natural choice (the PK is already authoritative on the op itself).

## What's Next (Phase 4b)
The remaining tables (maps + map_pins, timelines + timeline_events, page_tags M:N, wiki_links, versions, images metadata) plus the S3-compatible backend. Plan to be written when execution starts.
