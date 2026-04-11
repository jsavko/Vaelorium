---
status: completed
---
# M2 Phase 1: Entity Type Data Layer

**Date:** 2026-04-11

---

## Goal

Create the database schema, Rust commands, bridge mock, and unit tests for the entity type system. This is the foundation that all M2 UI features build on. No UI changes in this phase — pure data infrastructure.

**References:**
- Design spec: `design/specs/milestone-2-entity-types.md` (Data Model section)
- Parent plan: `.claude/plans/2026-04-11-m2-entity-type-system.md`

## Approaches Considered

### 1. New Migration File + Rust Commands
- **Description:** Create `002_entity_types.sql` migration with entity_types, entity_type_fields, entity_field_values tables. Add FK from pages.entity_type_id → entity_types.id. Seed 8 built-in types with their fields. Write Rust CRUD commands for types, fields, and values.
- **Pros:** Clean separation from M1 schema. Migration is additive. Built-in types available immediately.
- **Cons:** Need to handle FK addition to existing pages table (ALTER TABLE doesn't support ADD CONSTRAINT in SQLite — need to work around).

### 2. Seed Built-in Types in App Code
- **Description:** Same schema migration but seed built-in types from Rust code on first run, not in SQL.
- **Pros:** More flexible seeding logic. Can use UUIDs generated at runtime.
- **Cons:** Types created on first run means they might differ between installations. Migration-based seeding is more deterministic.

### 3. Schema in Migration, Seeding in Separate Migration
- **Description:** `002_entity_types.sql` for schema only. `003_seed_builtin_types.sql` for seeding the 8 built-in types and their fields.
- **Pros:** Clean separation of schema vs data. Can re-run seed independently.
- **Cons:** Two migration files for one logical unit.

## Chosen Approach

**Approach 1: Single migration file with schema + seeded data.** SQLite doesn't support ALTER TABLE ADD CONSTRAINT, but since `entity_type_id` on pages already exists as a TEXT column, we don't need a FK constraint — we just need the entity_types table to exist. The 8 built-in types use hardcoded UUIDs so they're deterministic across installations.

## Tasks

- [x] **1.** Create `src-tauri/migrations/002_entity_types.sql` — CREATE TABLE for entity_types, entity_type_fields, entity_field_values + indexes. INSERT 8 built-in types with hardcoded UUIDs. INSERT default fields for each type (per design spec table).

- [x] **2.** Create `src-tauri/src/commands/entity_types.rs` — Rust commands:
  - `list_entity_types() -> Vec<EntityType>`
  - `get_entity_type(id) -> EntityType`
  - `create_entity_type(name, icon, color) -> EntityType`
  - `update_entity_type(id, name?, icon?, color?) -> EntityType`
  - `delete_entity_type(id)` (only custom types, not built-in)

- [x] **3.** Create `src-tauri/src/commands/entity_fields.rs` — Rust commands:
  - `list_entity_type_fields(entity_type_id) -> Vec<EntityTypeField>`
  - `create_entity_type_field(entity_type_id, name, field_type, options?, ...) -> EntityTypeField`
  - `update_entity_type_field(id, name?, field_type?, sort_order?, ...) -> EntityTypeField`
  - `delete_entity_type_field(id)`
  - `reorder_entity_type_fields(moves: Vec<{id, sort_order}>)`

- [x] **4.** Create `src-tauri/src/commands/field_values.rs` — Rust commands:
  - `get_page_field_values(page_id) -> Vec<FieldValue>`
  - `set_field_value(page_id, field_id, value)` (upsert)
  - `delete_field_value(page_id, field_id)`
  - `query_pages_by_field(field_id, value) -> Vec<Page>` (for cross-entity queries)

- [x] **5.** Register all new commands in `src-tauri/src/lib.rs` invoke_handler. Add modules to `commands/mod.rs`. Verify `cargo check` passes.

- [x] **6.** Add all new commands to `src/lib/api/bridge.ts` mock backend — mock entity_types, entity_type_fields, entity_field_values maps. Seed 8 built-in types on mock initialization.

- [x] **7.** Create `src/lib/api/entityTypes.ts` — TypeScript API wrappers (typed `callCommand` wrappers for all entity type/field/value commands).

- [x] **8.** Create `src/lib/stores/entityTypeStore.ts` — Svelte store wrapping entity type commands. Reactive state for: all types, current page's type + fields + values.

- [x] **9.** Write unit tests (`src/lib/api/entityTypes.test.ts`):
  - List built-in types returns 8
  - Create custom type
  - Add fields to custom type
  - Set/get field values on a page
  - Delete custom type (verify built-in types can't be deleted)
  - Query pages by field value

- [x] **10.** Verify all existing tests still pass (no regressions from new migration/commands).

- [x] **11.** Create Phase 2 plan: `.claude/plans/2026-04-11-m2-phase2-type-selector.md`

## Notes

- Built-in type UUIDs should be deterministic (not random) so they're the same across all installations. Use namespace-based UUIDs or simple hardcoded values like `"builtin-character"`, `"builtin-location"`, etc.
- The `entity_type_id` column on `pages` is already TEXT with no FK. We don't need to ALTER the pages table — just start referencing the new entity_types table.
- Field values are stored as JSON-encoded TEXT. The `value` column stores: `"string"` for text, `42` for number, `true` for boolean, `"option_value"` for select, `["a","b"]` for multi_select, `"page_uuid"` for page_reference.
- The bridge mock needs to seed built-in types on initialization so browser testing works immediately.
