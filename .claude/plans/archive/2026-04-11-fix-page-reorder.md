---
status: completed
---
# Fix Page Tree Drag-and-Drop Reordering

**Date:** 2026-04-11

---

## Goal

The page tree's drag-and-drop currently only supports making a page a child of another page. It doesn't support:
- Reordering sibling pages (moving a page above or below another at the same level)
- Moving a child page back to root level
- Placing a page between two existing pages

The tree needs proper drop zones that distinguish between "drop on item" (make child) and "drop between items" (reorder as sibling).

## Approaches Considered

### 1. Drop Zone Indicators (Top/Middle/Bottom)
- **Description:** Each tree row has three invisible hit zones: top 25% = insert before (sibling), middle 50% = make child, bottom 25% = insert after (sibling). Show a visual indicator (line for between, highlight for child) based on cursor position within the row.
- **Pros:** Intuitive — matches VS Code, Finder, and other tree implementations. Single component handles all drop types. No extra DOM elements needed.
- **Cons:** Slightly more complex drag logic. Need to calculate cursor position relative to row height.

### 2. Separate Drop Line Elements
- **Description:** Add thin invisible `<div>` drop targets between every tree item. These only accept drops for reordering. The existing row drop target remains for "make child."
- **Pros:** Clear separation of concerns. Easy to style the drop line indicator.
- **Cons:** Doubles the number of DOM elements. Complex layout. Hard to align drop lines with nested depths.

### 3. Context Menu Reorder
- **Description:** Add "Move Up", "Move Down", "Move to Root" options to the right-click context menu instead of fixing drag-and-drop.
- **Pros:** Simple to implement. No drag-and-drop complexity.
- **Cons:** Slow for large reorders. Doesn't match user expectations for a tree UI. Can't move between distant positions easily.

## Chosen Approach

**Approach 1: Drop Zone Indicators (Top/Middle/Bottom).** This is the standard pattern for tree drag-and-drop. The cursor's Y position within each row determines the drop action. A thin line indicator shows "insert before/after" positions, while the existing highlight shows "make child."

## Tasks

- [x] **1.** Update `PageTreeItem.svelte` `handleDragOver` — calculate whether cursor is in top 25%, middle 50%, or bottom 25% of the row. Set a `dropPosition` state: `'before' | 'inside' | 'after' | null`.

- [x] **2.** Update `handleDrop` — use `dropPosition` to determine the action:
  - `'before'`: insert as sibling before this node (same parent_id, sort_order before this node)
  - `'inside'`: make child (current behavior)
  - `'after'`: insert as sibling after this node (same parent_id, sort_order after this node)

- [x] **3.** Add visual drop indicators — show a gold line above the row for `'before'`, gold highlight for `'inside'`, gold line below for `'after'`. Replace the current `drag-over` class with position-specific styling.

- [x] **4.** Add a root-level drop zone in the Sidebar — a drop area at the bottom of the tree list (or the tree-list container itself) that accepts drops and sets `parent_id: null` to move pages to root level.

- [x] **5.** Calculate correct sort_order values — when inserting before/after, compute the sort_order as the midpoint between adjacent items (or 0/max+1 for edges). Send the full reorder move with correct parent_id and sort_order.

- [x] **6.** Write Playwright E2E tests:
  - Drag a page to reorder (sibling position)
  - Drag a child page to root level
  - Drag a page into another page (make child — existing behavior preserved)

- [x] **7.** Verify all existing tests pass (no regressions).

## Notes

- The `reorderPages` API already supports setting both `parent_id` and `sort_order` in a single call, so no backend changes needed.
- Sort order uses integers. When inserting between items with sort_order 3 and 5, use sort_order 4. If items are consecutive (3, 4), we may need to re-number all siblings — or use a fractional approach with large gaps (multiply by 1000).
- The current sort_order increment is 1 per page. Consider using gaps (e.g., 1000, 2000, 3000) to allow insertions without renumbering.
- Need to pass `node.parent_id` from PageTreeItem so the drop handler knows the target's parent for sibling insertions.
