# Version History — UX Overhaul

**Date:** 2026-04-13

## Summary
Four distinct bugs in version history fixed in one pass: autosave no longer creates versions when nothing changed, the preview pane now renders HTML (matches the editor exactly), list and preview sit side-by-side instead of stacked-with-scroll, and the Restore button now actually restores (the editor's in-memory Y.Doc was silently autosaving the pre-restore content back over the DB).

## Changes

### Bug: autosave spam
`YjsProvider.maybeCreateVersion` used to check only elapsed time, so leaving a page open for a day produced ~288 "Auto-save" versions whether or not anything changed. Now tracks `dirtySinceLastVersion`: set on `doc.on('update')`, cleared when a snapshot is created, checked before firing the 5-minute interval. No changes → no version.

### Bug: preview is plain text
The old preview walked the Yjs XML tree and recursively `.toString()`'d nodes into a `<pre>` tag — headings, formatting, wiki-links, embeds, images all rendered as unreadable text. Fix: decode the Yjs snapshot into a throwaway **read-only Tiptap instance** using the same `createEditorExtensions` the live editor uses. `editor.getHTML()` produces proper HTML, rendered with `{@html ...}`. Scoped typography styles approximate the editor's look.

### Bug: preview hidden below a long list
The version list and preview shared one vertical scroll region inside a 400px-wide panel. Selecting a version dropped the preview *below* the entire list, so users had to scroll past every version to read the preview. Fix: widened panel to 720px (`max-width: min(720px, 80vw)` for narrow windows) + grid layout — narrow list column on the left, wide preview on the right, each scrolls independently. When no version is selected, the list uses the full width and the right column shows "Select a version to preview it here."

### Bug: Restore doesn't actually restore
The most serious one, caught by user manual testing:

- `Restore` calls `savePageContent(pageId, snapshot)` which writes the new `yjs_state` to the DB.
- But the live editor's `Y.Doc` is in-memory and unchanged.
- `loadPage(pageId)` reloads the page metadata (title etc.) but doesn't change `currentPage.id`, so the Editor's `$effect` (`page.id !== currentLoadedPageId`) doesn't fire → Y.Doc stays as-is.
- Worse: the next autosave writes the pre-restore Y.Doc state back, silently clobbering the restore. User sees a success toast but the content is the pre-restore version.

Fix: new `pageReloadSignal` writable in `pageStore` + `triggerPageReload()` helper. Editor's `$effect` now reads both `$currentPage` AND `$pageReloadSignal`; incrementing the signal forces `loadEditor` to `provider.destroy()` + recreate. VersionHistory calls `triggerPageReload()` after `savePageContent` + `loadPage`. Also: `loadEditor` now resets `currentLoadedPageId = null` before reassigning, so future reload triggers work even when the page id didn't change.

### Safety: pre-restore snapshot
Restore now takes a snapshot of the *current* page content BEFORE applying the restore, with summary `"Before restore to v{N}"`. If the user regrets the restore they can Restore that pre-restore snapshot back. No more silent data loss.

## Tests
- 4 new vitest in `src/lib/editor/YjsProvider.test.ts` covering dirty-flag behavior: skip-when-clean, fire-when-dirty, clear-on-snapshot, set-on-update.
- All 48 vitest (up from 44) + 3 Playwright version-history tests pass. Zero regressions.

## Files Modified
- `src/lib/editor/YjsProvider.ts` — dirty flag
- `src/lib/editor/YjsProvider.test.ts` — new
- `src/lib/components/VersionHistory.svelte` — layout + HTML preview + triggerPageReload + pre-restore snapshot
- `src/lib/components/Editor.svelte` — honor pageReloadSignal; reset currentLoadedPageId before reassignment
- `src/lib/stores/pageStore.ts` — `pageReloadSignal` + `triggerPageReload()`
- `.claude/plans/2026-04-13-version-history-ux.md` — marked complete

## Rationale
Version history is a safety-net feature — users trust it to preserve their work. "Creates useless versions" erodes trust; "preview you can't read" makes it unusable; "restore silently fails" is unsafe. All four needed to be fixed together for the feature to earn that trust.
