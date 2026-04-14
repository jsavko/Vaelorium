# Sidebar: refresh child list on delete + clamp context menu to viewport

**Date:** 2026-04-14

## Summary

Two small but user-visible bugs in the sidebar page tree are fixed:

1. Deleting a nested page now updates the sidebar immediately instead of leaving it visually lingering until the parent is collapsed + re-expanded.
2. The page-context-menu no longer clips past the window edge when the user right-clicks near the right or bottom of the viewport — it clamps to stay fully visible.

## Changes

### Bug fixes
- **Nested-page deletion reflects in the sidebar.** `PageTreeItem.svelte` now derives its child list directly from `$pageTree` (`$derived($pageTree.filter(...).sort(...))`) instead of `$derived(getChildren(node.id))`. The helper did `get(pageTree)` internally, which is a one-shot read and never registers as a Svelte 5 reactive dependency — so the root-level list updated fine (the proper `derived(pageTree, ...)` store) while expanded parents stayed stale.
- **Context menu viewport clamp.** `ContextMenu.svelte` now measures its rendered bounding box in a `$effect` and clamps the left/top position against `window.innerWidth / innerHeight` with a 6px margin. First render uses raw click coords (no flash); post-layout effect overrides with clamped values.

### Refactoring
- Removed the unused `getChildren` helper from `pageStore.ts` — had no remaining callers after the reactivity fix, and keeping it invites reintroducing the same `$derived(getChildren(...))` bug.

### Files Modified
- `src/lib/components/PageTreeItem.svelte` — inline `$pageTree` filter; dropped `getChildren` import.
- `src/lib/components/ContextMenu.svelte` — `bind:this` + post-layout clamp effect using `getBoundingClientRect`; uses `measured?.x ?? x` pattern so the initial frame still shows at click coords without the `state_referenced_locally` warning.
- `src/lib/stores/pageStore.ts` — dropped `getChildren` helper.

### Verification
- `npm run check` — 3,954 files clean (0 errors, 0 warnings).
- Runtime verification (delete a nested page + right-click at bottom-right) is on the user.

### Rationale

Both bugs surface from the same Svelte 5 subtlety (reactivity tracks *direct* reads inside reactive expressions, not values pulled through imperative helpers). Lesson captured in `feedback_derived_plus_get_is_not_reactive` memory. Context-menu clamping is table stakes UX for a right-click menu; the `measured?.x ?? x` fallback avoids the typical flash-at-origin problem that straight mounting patterns have.
