# M2 Phase 2: Type Selector UI

**Date:** 2026-04-11

## Summary

Built the New Page modal with entity type picker, replacing the previous inline page creation flow. Added entity type colors throughout the UI (sidebar, editor badge, search, mentions, backlinks). Added ability to change/remove page types via the details panel.

## Changes

### Features
- New Page modal with 8 built-in entity type cards (lucide icons, colored borders)
- "Blank Page" option for untyped pages
- Title input, parent folder picker, and Create button in modal footer
- Cmd/Ctrl+N and sidebar "+" button both open the modal
- Entity type badge in editor header (colored dot + type name)
- Colored dots in sidebar page tree based on entity type
- Entity type selector in details panel to change/remove page types
- Entity type colors in search results, @mentions, and backlinks panel

### Refactoring
- Replaced 4 hardcoded `entityColors` maps with dynamic `entityTypeMap` store lookups
- `pageStore.createPage()` now accepts optional `entity_type_id` parameter
- All E2E tests updated to work with modal-based page creation flow
- Added `e2e/helpers.ts` with shared `createPageViaModal()` and `createAnotherPage()` utilities

### Dependencies
- Added `lucide-svelte` for entity type icons

### Tests
- 8 new Playwright E2E tests for entity type system
- All 57 E2E tests passing (8 new + 49 existing)
- All 44 unit tests passing (no regressions)
- Removed obsolete "toast on page creation" test

## Files Modified
- `src/App.svelte` — entity type store init, new page modal wiring
- `src/lib/components/NewPageModal.svelte` — new component
- `src/lib/components/Editor.svelte` — entity type badge with color
- `src/lib/components/DetailsPanel.svelte` — entity type selector dropdown
- `src/lib/components/Sidebar.svelte` — onNewPage prop, modal integration
- `src/lib/components/PageTreeItem.svelte` — dynamic entity type colors
- `src/lib/components/SearchOverlay.svelte` — dynamic entity type colors
- `src/lib/components/MentionSuggestion.svelte` — dynamic entity type colors
- `src/lib/components/BacklinksPanel.svelte` — dynamic entity type colors
- `src/lib/stores/pageStore.ts` — createPage accepts entity_type_id
- `e2e/helpers.ts` — new shared test utilities
- `e2e/entity-types.spec.ts` — new E2E tests
- All existing `e2e/*.spec.ts` — updated for modal flow
- `package.json` — lucide-svelte dependency
