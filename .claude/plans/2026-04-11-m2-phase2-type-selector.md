---
status: created
---
# M2 Phase 2: Type Selector UI

**Date:** 2026-04-11

---

## Goal

Build the "New Page" modal with entity type picker. Replace the current inline page creation flow with a modal that lets users select an entity type (or blank page). Update the sidebar tree and search results to show entity-colored badges from actual type data.

**References:**
- Design spec: `design/specs/milestone-2-entity-types.md` (New Page Modal section)
- Mockups: `design/vaelorium-entity-types.pen` (screen gtNUI)
- Parent plan: `.claude/plans/2026-04-11-m2-entity-type-system.md`
- Data layer: Phase 1 complete — entity types, fields, values all available via bridge + API wrappers + store

## Approaches Considered

### 1. Modal with Type Grid
- **Description:** Full modal overlay with "Blank Page" option at top, then a 3-column grid of entity type cards (icon + name + color), title input, parent selector, and Create button.
- **Pros:** Matches the design spec exactly. Clear visual hierarchy. Type selection is prominent.
- **Cons:** Bigger change — need a new modal component and to replace existing creation flow.

### 2. Inline Dropdown
- **Description:** Add a type dropdown to the existing creation flow in the sidebar context menu.
- **Pros:** Minimal UI change.
- **Cons:** Doesn't match mockups. Cramped. Doesn't showcase types well.

## Chosen Approach

**Approach 1: Modal with Type Grid.** Matches the design spec. The modal is reusable for other creation flows later.

## Tasks

- [ ] **1.** Initialize entity type store on app startup — call `loadEntityTypes()` from the main layout/app initialization so entity types are available immediately.

- [ ] **2.** Build `NewPageModal.svelte` — Modal overlay with:
  - "Blank Page" full-width option at top
  - "ENTITY TYPES" section label
  - 3-column grid of type cards (icon from lucide, name, colored left border)
  - Selected type gets gold border highlight; clicking again deselects
  - Title input auto-focuses after type selection
  - Parent folder picker (dropdown of existing pages)
  - "Create" button disabled until title entered
  - Escape closes the modal

- [ ] **3.** Wire Cmd/Ctrl+N to open `NewPageModal` instead of directly creating a page. Update sidebar "New Page" button to also open the modal.

- [ ] **4.** Update `pageStore.createPage()` to accept `entity_type_id` parameter. Wire the modal's Create button to call it with selected type.

- [ ] **5.** Update sidebar page tree to use `entityTypeMap` store for entity type colors — replace the hardcoded gray dot with actual type color when the page has an `entity_type_id`.

- [ ] **6.** Update search results and @mention dropdown to show entity type name label + color dot from `entityTypeMap` store.

- [ ] **7.** Update entity type badge in editor header — show type name + colored badge above the page title when the page has an entity type.

- [ ] **8.** Add "Change Type" option to the ⋯ more menu or details panel — dropdown to change or remove a page's entity type.

- [ ] **9.** Write Playwright E2E tests:
  - Open new page modal via Cmd+N
  - Create a blank page (no type)
  - Create a typed page (select Character type)
  - Verify type badge appears in editor
  - Verify colored dot appears in sidebar tree
  - Change page type via menu
  - Remove page type

- [ ] **10.** Verify all existing tests still pass (no regressions).

## Notes

- The 8 built-in entity types are seeded in the bridge mock on initialization, so they're available immediately in browser dev mode.
- Entity type icons use lucide icon names (shield, compass, scroll, etc.) — the app already has lucide-svelte installed.
- The gold highlight color for selected type should use the design system's accent color token.
- Parent folder picker can reuse the existing page tree data from pageStore.
