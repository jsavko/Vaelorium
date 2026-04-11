---
status: completed
---
# Milestone 3: Relations & Connections

**Date:** 2026-04-11

---

## Goal

Typed bidirectional relations between pages. Relations panel in the details panel. Graph view for visual exploration. Custom relation types.

**References:**
- Design spec: `design/specs/milestone-3-relations.md`

## Chosen Approach

Two phases: Phase 1 (data layer + relations panel) and Phase 2 (graph view). Execute both in this plan.

## Tasks

### Phase 1: Data Layer + Relations Panel

- [x] **1.** Create `005_relations.sql` migration — relation_types and relations tables. Seed 10 built-in relation types. Register migration.

- [x] **2.** Create `src-tauri/src/commands/relations.rs` — Rust commands: list_relation_types, create_relation_type, create_relation, delete_relation, get_page_relations, list_all_relations.

- [x] **3.** Register commands in lib.rs + mod.rs.

- [x] **4.** Add relation commands to bridge.ts mock — seed built-in types, mock CRUD.

- [x] **5.** Create `src/lib/api/relations.ts` — TypeScript API wrappers.

- [x] **6.** Create `src/lib/stores/relationStore.ts` — store for current page relations, all relation types.

- [x] **7.** Build `RelationsPanel.svelte` — section in DetailsPanel showing relations + add relation inline form (page search + type dropdown).

- [x] **8.** Add RelationsPanel to DetailsPanel.svelte.

### Phase 2: Graph View

- [x] **9.** Install d3-force for graph layout.

- [x] **10.** Build `GraphView.svelte` — canvas-based graph with force-directed layout, entity-colored nodes, relation-colored edges.

- [x] **11.** Wire "Relations" sidebar nav item to show GraphView (replace editor area like EntityListView).

- [x] **12.** Add graph controls — zoom, pan, filter by entity type, click node to navigate.

### Testing

- [x] **13.** Write unit tests for relation commands (bridge mock).

- [x] **14.** Write E2E tests: add relation, see it on both pages, delete relation, graph view opens.

- [x] **15.** Verify all existing tests pass.

## Notes

- Relations are bidirectional: creating A→B also shows on B's panel (with inverse label).
- Relation types have name + inverse_name for asymmetric relations (e.g., "Parent of" / "Child of").
- Graph rendering uses HTML Canvas for performance. SVG fallback not needed for M3 scope.
