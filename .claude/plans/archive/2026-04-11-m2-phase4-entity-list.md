---
status: completed
---
# M2 Phase 4: Entity List + Custom Types

**Date:** 2026-04-11

---

## Goal

Build the entity list view for browsing all pages of a specific type, and the custom type builder for creating user-defined entity types. The list view replaces the main content area when a type tab is selected. The custom type builder is a modal accessible from the new page modal.

**References:**
- Design spec: `design/specs/milestone-2-entity-types.md` (Entity List View + Custom Type Builder sections)
- Parent plan: `.claude/plans/2026-04-11-m2-entity-type-system.md`

## Chosen Approach

Build the entity list as a new view component that replaces the editor when a type is selected from sidebar navigation. Build the custom type builder as a modal with field management. Keep scope focused — card grid view first, list table view as stretch.

## Tasks

- [x] **1.** Add `list_pages_by_type` command to Rust backend + bridge mock — returns pages filtered by entity_type_id with key field values. Add to API wrappers.

- [x] **2.** Build `EntityListView.svelte` — main content replacement showing:
  - Header: type icon + name + page count + "New [Type]" button
  - Search input for filtering by name
  - Card grid (3 columns): featured image placeholder, title, key field values, entity color accent
  - Click card to open page
  - Empty state: "No [type] yet" with create button

- [x] **3.** Add entity type navigation to sidebar — clickable type entries under a "TYPES" section that switch the main view to the entity list for that type. Highlight active type.

- [x] **4.** Wire view switching in App.svelte — track active view (editor vs entity-list). When a type is selected from sidebar, show EntityListView. When a page is selected, show Editor.

- [x] **5.** Build `CustomTypeBuilder.svelte` — modal with:
  - Type name input, icon picker (lucide icons), color picker
  - Field list with add/remove/reorder
  - Each field: name input, type dropdown, options editor for select types
  - Save button creates the type via the entity type store

- [x] **6.** Add "Custom Type" card to NewPageModal — at the end of the type grid, a "+ Custom Type" card that opens the builder modal.

- [x] **7.** Write Playwright E2E tests:
  - Navigate to Character list view
  - Create a character, verify it appears in list
  - Search filters the list
  - Click card opens the page
  - Create a custom type via builder
  - Custom type appears in new page modal
  - Empty state shows when no pages of a type exist

- [x] **8.** Verify all existing tests pass (no regressions).

## Notes

- Card grid is the primary view. List table view can be added in a polish pass.
- Sort dropdown (by name, date) is nice-to-have — default to alphabetical.
- The entity list needs to load field values for each page to show metadata on cards. For performance, fetch key fields only (first 2-3 fields).
- Custom type builder icon picker can reuse lucide icon names. Color picker can be a simple palette of 8-10 preset colors.
