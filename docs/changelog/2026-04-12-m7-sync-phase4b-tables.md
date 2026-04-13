# M7 Sync — Phase 4b-Tables: Wire Remaining Modules + M:N Pivot

**Date:** 2026-04-12

## Summary
Wired op-emission for every remaining user-data table via the registry pattern from Phase 4a. Added the special-case apply path for the `page_tags` M:N pivot. After this commit: every user-facing mutation in the app (across 11 sync-tracked tables) produces sync ops. Only the S3 transport remains for a fully-functional MVP.

## Changes

### New table wiring
- **`commands/maps.rs`** — `create_map`, `delete_map`, `create_pin`, `update_pin`, `delete_pin` all wired through `emit_for_row(&TABLES.maps | TABLES.map_pins, ...)`.
- **`commands/timelines.rs`** — `create_timeline`, `delete_timeline`, `create_timeline_event`, `update_timeline_event`, `delete_timeline_event` wired.
- **`commands/tags.rs`** — `add_tag_to_page`, `remove_tag_from_page` wired (uses M:N pivot composite-key path).
- **`commands/entity_fields.rs`** — `create_entity_type_field`, `update_entity_type_field`, `delete_entity_type_field` wired.
- **`commands/field_values.rs`** — `set_field_value` (upsert: distinguishes Insert vs Update via existence pre-check), `delete_field_value` wired.
- **`commands/boards.rs`** — `create_board`, `delete_board`, `create_card`, `update_card`, `delete_card`, `create_connector`, `delete_connector` wired across 3 tables (boards, board_cards, board_connectors).

### M:N pivot apply path
- **`engine::apply_page_tags_op`** — special-case for the `page_tags` pivot. `op.row_id` is `"<page_id>|<tag_id>"`. Insert: `INSERT OR IGNORE INTO page_tags`. Delete: `DELETE FROM page_tags WHERE page_id = ? AND tag_id = ?`. Bypasses `apply_op_via_schema` because the composite key doesn't fit the registry's single-PK assumption.
- **`engine::apply_op` dispatch** extended to recognize `page_tags` alongside the existing `page_content` BLOB special case.

### Registry additions
- `TableSchema` entries for: `entity_type_fields`, `entity_field_values`, `page_tags` (re-added from 4a deferral), `boards`, `board_cards`, `board_connectors`. `schema_by_name()` lookup updated for all.

### Tests
- **`registry_boards_propagate`** — D1 creates a board + card via registry helpers, syncs; D2 syncs and observes both rows with content intact. Validates the largest registry-wired table family.
- **`registry_page_tags_pivot_propagates`** — D1 creates a page, a tag, and the pivot association; syncs; D2 observes the M:N row. Validates the special apply path end-to-end.
- All prior 9 sync integration scenarios + 38 sync unit + 44 vitest + 21 Playwright still pass — zero regressions.

## Files Modified
- `src-tauri/src/sync/registry.rs` — 6 new registry entries (entity_type_fields, entity_field_values, page_tags, boards, board_cards, board_connectors)
- `src-tauri/src/sync/engine.rs` — `apply_page_tags_op` + dispatch update
- `src-tauri/src/commands/{maps,timelines,tags,entity_fields,field_values,boards}.rs` — wired through registry
- `src-tauri/tests/sync_integration.rs` — 2 new scenarios (boards + page_tags pivot)

## Rationale
Phase 4a built the registry pattern; this phase applied it. Adding 11 mutation functions across 6 modules took ~600 LOC of mostly-mechanical edits — far less than the ~1500 LOC the same work would have required without the registry. The M:N pivot needed a special apply path, but it's a clean carve-out (~15 LOC) and documented as the second of two known special cases (the first being `page_content` for binary BLOB).

## Decisions Made During Execution
- **`create_relation_type` deferred** — relation types are mostly built-in; user-created ones are rare and low-priority. Wire in Phase 5 polish if needed.
- **`reorder_entity_type_fields` deferred** — multi-row update with similar shape to `reorder_pages`. Low priority; lands when needed.
- **`images` metadata syncing skipped this phase** — image binary files don't sync (separate file-sync layer is M8 territory). Syncing just the metadata row without the file produces broken thumbnails on remote devices, so the value is low until binary sync exists. Defer the whole table.
- **`wiki_links` and `versions` stay local-only as planned** — derived/recomputable from page content; syncing would balloon the journal for no benefit.

## Coverage Status
**Sync-tracked tables: 11 of ~14 user-data tables.**
- ✅ pages, page_content (BLOB special), entity_types, entity_type_fields, entity_field_values, relations, tags, page_tags (M:N special), maps, map_pins, timelines, timeline_events, boards, board_cards, board_connectors
- ⏭ Skipped (intentional): wiki_links (derived), versions (large; local-only), images (binary blob deferred), relation_types (built-ins dominate)

## What's Next (Phase 4b-S3)
S3-compatible backend implementation (`aws-sdk-s3`), atomic_swap via `If-Match` conditional writes, error wrapping, Settings UI un-disable + S3 form fields, integration tests against a mock crate. Plan to be written when execution starts.
