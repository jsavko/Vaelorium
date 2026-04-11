# Fix Page Tree Drag-and-Drop Reordering

**Date:** 2026-04-11

## Summary

Fixed drag-and-drop in the page tree to support sibling reordering and moving pages back to root level. Previously, dropping a page on another always made it a child — now cursor position within the row determines the action.

## Changes

### Bug Fixes
- Drop on top 25% of row = insert before (sibling reorder)
- Drop on middle 50% of row = make child (preserved existing behavior)
- Drop on bottom 25% of row = insert after (sibling reorder)
- Gold line indicators show before/after positions, dashed outline for inside
- Root-level drop zone at bottom of tree list for moving child pages back to root
- Sort order uses 1000-gap spacing to allow insertions without renumbering

## Files Modified
- `src/lib/components/PageTreeItem.svelte` — drop zone logic, position indicators, sibling sort order calculation
- `src/lib/components/Sidebar.svelte` — root drop zone with drag handlers
