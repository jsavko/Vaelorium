---
status: completed
---
# M2 Phase 3: Structured Fields

**Date:** 2026-04-11

---

## Goal

Display entity type fields in the details panel with inline editing. Each field type (text, number, select, multi_select, boolean, page_reference) gets an appropriate input widget. Values save on change via the field values API.

**References:**
- Design spec: `design/specs/milestone-2-entity-types.md` (Field Types table)
- Data layer: Phase 1 — `entityTypeStore`, `entityTypes.ts` API wrappers, bridge mock all ready
- Parent plan: `.claude/plans/2026-04-11-m2-entity-type-system.md`

## Chosen Approach

Build an `EntityFields.svelte` component that renders in the details panel when the current page has an entity type. It loads fields for the type and values for the page, then renders each field with the appropriate input widget. Values are saved on blur/change via the store's `setFieldValue`.

## Tasks

- [x] **1.** Wire `loadPageEntityData` in the details panel — when `currentPage` changes and has an `entity_type_id`, load fields + values via the entity type store.

- [x] **2.** Build `EntityFields.svelte` — iterates over `currentPageFields` store, renders each field with label and appropriate input based on `field_type`. Field types: text (input), number (number input), select (dropdown from options JSON), multi_select (tag picker from options JSON), long_text (textarea), boolean (toggle), page_reference (page search dropdown).

- [x] **3.** Wire field value persistence — on input change/blur, call `setFieldValue(pageId, fieldId, jsonValue)`. Debounce text/number inputs (300ms). Select/boolean save immediately on change.

- [x] **4.** Handle page_reference fields — dropdown filtered by `reference_type_id`. Show matching pages from `pageTree`. Display as entity dot + gold link when set.

- [x] **5.** Add EntityFields to DetailsPanel — render between "Entity Type" section and "Page Info" section when the page has a type.

- [x] **6.** Write unit tests for field value operations (extend `entityTypes.test.ts`).

- [x] **7.** Write Playwright E2E tests:
  - Create a Character page, open details, verify fields appear (Race, Class, etc.)
  - Edit a text field, verify value persists across page switches
  - Edit a select field (Status dropdown)
  - Edit a number field (HP)
  - Toggle a boolean field
  - Change page type, verify fields update

- [x] **8.** Verify all existing tests pass (no regressions).

## Notes

- Field values are stored as JSON-encoded strings: `"Elf"` for text, `42` for number, `true` for boolean, `"option"` for select, `["a","b"]` for multi_select, `"page-uuid"` for page_reference.
- The `options` column on entity_type_fields is a JSON array string — parse it to get dropdown options.
- `currentPageFields` and `currentPageFieldValues` stores are already defined in entityTypeStore.ts.
- multi_select and page_reference are more complex widgets — implement basic versions first, polish in Phase 4 if needed.
