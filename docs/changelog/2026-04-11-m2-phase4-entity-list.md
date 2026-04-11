# M2 Phase 4: Entity List + Custom Types

**Date:** 2026-04-11

## Summary

Built the entity list view for browsing pages by type and the custom type builder for creating user-defined entity types. Added entity type navigation to the sidebar.

## Changes

### Features
- Entity list view: card grid showing all pages of a selected type with search filtering
- Sidebar "TYPES" section with colored dots for all 8 built-in types + custom types
- View switching between editor and entity list
- `list_pages_by_type` Rust command + bridge mock
- Custom Type Builder modal: name, color picker, icon picker, field management
- "+ Custom Type" card in New Page modal
- Empty state for types with no pages
- Cards show page title + key field values as metadata

### Tests
- 8 new Playwright E2E tests (6 entity list + 2 custom type builder)
- All 72 E2E tests passing
- All 44 unit tests passing

## Files Modified
- `src-tauri/src/commands/pages.rs` — list_pages_by_type command
- `src-tauri/src/lib.rs` — register new command
- `src/lib/api/bridge.ts` — mock list_pages_by_type
- `src/lib/api/pages.ts` — listPagesByType wrapper
- `src/lib/components/EntityListView.svelte` — new component
- `src/lib/components/CustomTypeBuilder.svelte` — new component
- `src/lib/components/NewPageModal.svelte` — custom type card + builder integration
- `src/lib/components/Sidebar.svelte` — entity type navigation
- `src/App.svelte` — view switching, type list state
- `e2e/entity-list.spec.ts` — new E2E tests
