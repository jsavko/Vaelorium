# M2 Phase 1: Entity Type Data Layer

**Date:** 2026-04-11

## Summary

Created the complete data infrastructure for the entity type system — database schema, Rust backend commands, TypeScript API layer, Svelte store, and bridge mock with seeded built-in types. This is the foundation that all M2 UI features build on.

## Changes

### Features
- SQLite migration (`002_entity_types.sql`) with 3 new tables: `entity_types`, `entity_type_fields`, `entity_field_values`
- 8 built-in entity types seeded with deterministic IDs and 35 default fields
- Full CRUD Rust commands for entity types, fields, and field values
- Cross-entity query support (`query_pages_by_field`)
- Field reordering command
- Built-in type deletion protection
- TypeScript API wrappers with full type definitions
- Svelte reactive store with derived helpers (type map, builtin/custom filters)
- Bridge mock with all 14 new commands + seeded built-in data

### Tests
- 14 new unit tests covering all entity type operations
- All 44 tests passing (14 new + 30 existing, zero regressions)

## Files Modified
- `src-tauri/migrations/002_entity_types.sql` — new migration with schema + seed data
- `src-tauri/src/commands/entity_types.rs` — entity type CRUD commands
- `src-tauri/src/commands/entity_fields.rs` — entity field CRUD + reorder commands
- `src-tauri/src/commands/field_values.rs` — field value get/set/delete/query commands
- `src-tauri/src/commands/mod.rs` — register new modules
- `src-tauri/src/lib.rs` — register 14 new Tauri commands
- `src-tauri/src/db/migrations.rs` — register 002 migration
- `src/lib/api/bridge.ts` — mock backend with seeded built-in types + all new commands
- `src/lib/api/entityTypes.ts` — TypeScript API wrappers
- `src/lib/stores/entityTypeStore.ts` — Svelte store
- `src/lib/api/entityTypes.test.ts` — unit tests

## Rationale

The entity type system is a core M2 feature that enables structured data alongside wiki content. Building the complete data layer first (without UI) ensures a solid foundation — all commands are tested via the bridge mock before any UI work begins. Phase 2 (Type Selector UI) can now build directly on this infrastructure.
