# Rich, editable board cards

**Date:** 2026-04-13
**Plan:** `.claude/plans/archive/2026-04-13-rich-board-cards.md`

## Summary

Boards shipped in M6 with plain-text-only cards + line connectors — useful as a whiteboard, but missing the rich-content polish other entities had. This rework repurposes the existing `board_cards.content` TEXT column from plain text to TipTap-serialized HTML, letting cards hold formatted notes, bullet lists, and `@mention` page links that navigate on click.

Zero schema, migration, or sync-registry changes — `content` is already a sync-tracked regular TEXT column. Existing plain-text cards render fine (HTML treats them as a single text run). The engine, backend, and all 68 lib + 13 integration tests are untouched.

## Changes

### Features
- Double-click a card → inline `BoardCardEditor.svelte` opens for rich editing. Esc or click-outside saves via `update_card({ content })`; Cmd/Ctrl+Enter also saves.
- TipTap extension set (paragraphs, lists, bold/italic/strike/code, links, hard breaks, undo/redo, mentions) — deliberately tight; no tables, headings, code blocks, or images inside cards.
- `@`-typing inside a card opens the same global mention picker used in page bodies (shared via the existing `MentionSuggestion` singleton).
- Mentioned page/map/timeline links inside a card are clickable — route to the entity using the same handler as `Editor.svelte`.
- Cards now have a drag-to-resize handle in the bottom-right corner; minimum size 160×100, default for new cards 240×140.
- Double-click canvas now spawns a card and immediately drops into edit mode (no more separate "Add Card" modal).

### Files modified

- `src/lib/components/BoardCardEditor.svelte` (new) — tight TipTap wrapper with MentionExtension + placeholder; emits HTML on blur / Esc / Cmd+Enter.
- `src/lib/components/BoardView.svelte` — renders card content via `{@html ...}`, wires the mention click handler, adds resize handle, swaps to `BoardCardEditor` when editing.
- `src/lib/stores/boardStore.ts` — new `updateCardContent(id, html)` and `resizeCard(id, w, h)` actions.

### Removed

- `InputModal` trigger path for new cards. Cards are now created empty and edited inline — zero-friction flow.

## Rationale

Previous "add card" flow forced the user into a tiny text-only modal with no way to link pages, no formatting, and no richer content. The canvas + connectors were fine, but the card *body* was a dead end. Reusing the existing TipTap + MentionExtension on the HTML stored directly in `content` gave maximum reuse with minimum new surface area: no new table, no new sync special case, no migration.

## Trade-offs locked in

- **No card-level live collab.** Concurrent edits on the same `content` field surface as standard field conflicts in the existing conflict modal. Acceptable for sticky-note-scale content; upgrade to per-card Yjs if usage outgrows this.
- **HTML trust model:** rendered via `{@html ...}` matching the existing `VersionHistory.svelte` pattern. Relies on TipTap producing constrained output locally and the E2EE trusted-peer model for remote content.

## Test evidence

- `npm run check` clean (pre-existing `vite.config.ts` TS error unrelated).
- `cargo test --lib` 68/68 passing (no Rust changes).
