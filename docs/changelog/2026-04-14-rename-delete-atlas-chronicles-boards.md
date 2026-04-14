# Rename & delete for Atlas, Chronicles, Boards

**Date:** 2026-04-14

## Summary
Users can now rename and delete maps, timelines, and boards after creating them. Before this change, all three were create-only from the UI (delete commands existed in Rust but had no frontend entry point; rename wasn't implemented at all).

## Features
- **Atlas (maps):** right-click a map card to open a context menu with Rename / Delete.
- **Boards:** right-click a board card to open the same Rename / Delete menu.
- **Chronicle (timelines):** a new "⋯" button next to the current timeline's name opens the Rename / Delete menu. Deleting the active timeline falls back to the first remaining one.
- Rename uses `InputModal` prefilled with the current name (text pre-selected for overtype).
- Delete uses `ConfirmDialog` in danger mode, with the entity's current name in the prompt.

## Files Modified

### Backend (Rust)
- `src-tauri/src/commands/maps.rs` — new `update_map(id, title)` command; bumps `updated_at`, emits sync journal op.
- `src-tauri/src/commands/timelines.rs` — new `update_timeline(id, name)` command; same pattern.
- `src-tauri/src/commands/boards.rs` — new `update_board(id, name)` command; same pattern.
- `src-tauri/src/lib.rs` — registered all three in `invoke_handler![]`.

### Frontend
- `src/lib/api/{maps,timelines,boards}.ts` — added `updateMap` / `updateTimeline` / `updateBoard` wrappers (camelCase args).
- `src/lib/stores/{mapStore,timelineStore,boardStore}.ts` — added `renameMap` / `renameTimeline` / `renameBoard` + `deleteMap` / `deleteTimeline` / `deleteBoard` store helpers that refresh list + current-selection state.
- `src/lib/api/bridge.ts` — mock handlers for `update_map`, `update_timeline`, `update_board` so browser-mock dev works.
- `src/lib/components/MapList.svelte` — wired `oncontextmenu` on map cards; Rename / Delete flow.
- `src/lib/components/BoardList.svelte` — same pattern for boards.
- `src/lib/components/ChronicleView.svelte` — "⋯" actions button next to timeline name; Rename / Delete flow; auto-advances to first remaining timeline after delete.
- `src/lib/components/InputModal.svelte` — added optional `initialValue` prop; modal now focuses + selects prefilled text on open.

## Rationale
Context menu mirrors the page-tree UX (`PageTreeItem.svelte`), so users get a consistent "right-click to manage" gesture across the app. `ChronicleView` doesn't have a card grid (it shows one active timeline at a time), so a small `MoreHorizontal` button replaces the right-click gesture while opening the same `ContextMenu` component.

All three backend commands emit `journal::emit_for_row` with `OpKind::Update`, so renames propagate through the existing sync pipeline without schema or registry changes.

## Verification
Boards and Chronicle verified end-to-end in browser-mock: create, rename (prefilled modal, save updates title), delete (confirm dialog, list empties). Atlas code path is identical to Boards and shares all UI components; not exercised in this pass because the create flow requires a real image file pick.

Rust unit tests and Playwright e2e deliberately not added — the three new commands are thin shims of the same shape as existing `update_pin` / `update_timeline_event` (which also lack dedicated tests), and no Playwright harness currently exists for the Atlas/Chronicle/Boards list views.
