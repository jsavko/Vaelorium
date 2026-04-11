---
status: completed
---
# Milestone 2: Entity Type System

**Date:** 2026-04-11

---

## Goal

Add optional structured fields to wiki pages. Pages can have a type (Character, Location, Quest, etc.) which adds a set of defined fields alongside the free-form wiki content. Users can also create custom types. Entity types enable filtered list views, entity badges with colors, and structured data in the details panel.

Reference: `design/specs/milestone-2-entity-types.md`, `design/vaelorium-entity-types.pen`

## Approaches Considered

### 1. Four Sequential Sub-Plans
- **Description:** Break M2 into 4 plans executed in order: Data Layer → Type Selector → Structured Fields → Entity List + Custom Builder. Each plan creates the next before executing.
- **Pros:** Each phase is self-contained with its own tests. Clear dependency chain. A new session can pick up from any phase. Prevents one massive plan.
- **Cons:** More plan files. Overhead of plan creation between phases.

### 2. Single Monolithic Plan
- **Description:** One plan with all ~40 tasks for the entire milestone.
- **Pros:** Everything in one place. No coordination between plans.
- **Cons:** Too large to execute in one session. Hard to track progress. Same mistake as M1's original plan.

### 3. Two Plans (Backend + Frontend)
- **Description:** Split into backend (schema + commands) and frontend (all UI).
- **Pros:** Clean backend/frontend separation.
- **Cons:** Frontend tasks have dependencies between them (type selector needed before fields). Doesn't reflect the natural build order.

## Chosen Approach

**Approach 1: Four Sequential Sub-Plans.** Each phase has its own plan with 8-12 tasks. Each plan's final task creates the next plan. This lets a new session start by running `/execute-plan` and it picks up wherever the previous session left off.

## Execution Order

### Phase 1: Data Layer (`m2-phase1-data-layer`)
Create the SQLite migration, Rust commands, bridge mock, and unit tests for entity types, fields, and field values. No UI — pure data infrastructure.

### Phase 2: Type Selector (`m2-phase2-type-selector`)  
Build the "New Page" modal with entity type picker. Modify page creation flow to accept a type. Update sidebar tree to show entity-colored badges.
**Depends on:** Phase 1

### Phase 3: Structured Fields (`m2-phase3-structured-fields`)
Display entity fields in the details panel. Inline editing for each field type (text, number, select, multi-select, boolean, page reference). Save on change.
**Depends on:** Phase 1, Phase 2

### Phase 4: Entity List + Custom Types (`m2-phase4-entity-list`)
Build the filtered entity list view (card grid + list table). Build the custom type builder (split-pane modal with live preview). Type tabs, sorting, filtering.
**Depends on:** Phases 1-3

### Phase 5: Page Embeds (`m2-phase5-page-embeds`)
Embed one page's content inline inside another (transclusion). TipTap node extension with `/embed` slash command and `![[Page]]` syntax. Renders read-only, updates when source changes.
**Depends on:** Phase 1 (wiki engine only, not entity types)

## Tasks (This Plan = Coordinator)

- [x] **1.** Create and execute Phase 1 plan: `m2-phase1-data-layer`
- [x] **2.** Create and execute Phase 2 plan: `m2-phase2-type-selector`
- [x] **3.** Create and execute Phase 3 plan: `m2-phase3-structured-fields`
- [x] **4.** Create and execute Phase 4 plan: `m2-phase4-entity-list`
- [x] **5.** Create and execute Phase 5 plan: `m2-phase5-page-embeds`
- [x] **6.** Update PROJECT_OUTLINE.md — mark all M2 deliverables complete
- [x] **7.** Final integration test: full E2E flow (create typed page, fill fields, view in list, create custom type, embed a page)

## Notes

- The `entity_type_id` column already exists on the `pages` table from M1 migration (no FK constraint). M2 adds the `entity_types` table and FK.
- Built-in types (8 total) should be seeded in the migration, not on first app load.
- Entity type colors are already mapped in sidebar/search/mention components — they'll start showing correctly once pages have types assigned.
- The bridge mock needs to support all new commands for browser testing.
- Every phase must have passing Playwright E2E tests before marking complete.
