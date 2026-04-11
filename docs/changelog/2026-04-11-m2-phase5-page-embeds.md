# M2 Phase 5: Page Embeds (Transclusion)

**Date:** 2026-04-11

## Summary

Added the ability to embed one page's content inline inside another page. Embedded content renders read-only and loads dynamically from the source page. Triggered via `/embed` slash command with a page picker.

## Changes

### Features
- `PageEmbedNode` TipTap extension — custom block node storing `pageId` attribute
- `/embed` slash command in the slash menu opens a page picker
- Embed picker dialog shows all pages except the current one
- Embedded content renders as a styled card with page title header and "Open →" link
- "Open →" button navigates to the source page
- Recursive embed detection prevents infinite loops
- Embed content loads dynamically from Yjs document state

### Tests
- 5 new Playwright E2E tests for page embeds
- All 77 E2E tests passing (5 new + 72 existing)
- All 44 unit tests passing

## Files Modified
- `src/lib/editor/PageEmbedExtension.ts` — new TipTap node extension
- `src/lib/editor/EditorConfig.ts` — register PageEmbedNode
- `src/lib/editor/SlashCommands.ts` — add Page Embed command
- `src/lib/components/Editor.svelte` — embed picker UI, event listeners, embed CSS
- `e2e/page-embeds.spec.ts` — new E2E tests
- `e2e/slash-commands.spec.ts` — updated command count (9 → 10)

## Notes
- The `![[Page Name]]` trigger syntax (task 4 in the plan) was deferred — `/embed` provides the core functionality. The syntax trigger can be added as a polish item.
