# Pages & editor

## Responsibility
Rich-text page editing, page CRUD, wiki-style cross-linking, @-mentions of pages, slash menu for block insertion. TipTap editor over a Y.js doc backed by a binary `page_content` BLOB.

## Entry points

### Rust (`src-tauri/src/commands/pages.rs` — 449 lines)
- `list_pages`, `get_page`, `save_page`, `create_page`, `delete_page` — the CRUD.
- `load_page_tree` — hierarchical rendering data for Sidebar.
- Page mutations emit journal ops via `journal::emit_for_row(&mut *tx, &TABLES.pages, ...)`. `page_content` has a custom sync apply path (see sync.md).
- `commands::wiki_links` — derived index of page→page links (NOT synced; rebuilt locally on content change).

### Frontend
- `src/lib/components/Editor.svelte` (894 lines, see file-section-map.md) — the TipTap editor host. Wires Y.js provider, extensions, save hook.
- `src/lib/editor/` — TipTap extension modules:
  - `EditorConfig.ts` — StarterKit + extension assembly. Note `history` renamed to `undoRedo` (`feedback_tiptap_starterkit`).
  - `WikiLink.ts` + `WikiLinkSyntax.ts` — `[[Page Name]]` parsing + rendering.
  - `MentionExtension.ts` — `@`-triggered suggestion for pages.
  - `SlashCommands.ts` — `/` menu for block types.
  - `CalloutExtension.ts`, `FloatImage.ts`, `PageEmbedExtension.ts`, `ImagePastePlugin.ts` — block-level features.
  - `YjsProvider.ts` + `.test.ts` — local Y.doc bridge to the `page_content` BLOB.
- `src/lib/stores/pageStore.ts` — `currentPageId`, `pageTree`, `loadPageTree`.

### Suggestion popovers
- `src/lib/components/MentionSuggestion.svelte` — mention dropdown.
- `src/lib/components/SlashMenu.svelte` — slash menu UI.
- `src/lib/components/BacklinksPanel.svelte` — inbound wiki link display.

## Data flow (edit)
1. TipTap commits a tx on keystroke; Y.doc updates in `YjsProvider`.
2. Debounced save flushes the Y.doc binary to `save_page({page_id, content_blob})`.
3. Rust writes `pages.page_content` BLOB + emits a sync op with the content hash.
4. Remote apply decodes the blob into the local Y.doc on pull.

## Gotchas
- `YjsProvider.destroy()` auto-saves before teardown. Use `discard()` when the DB was replaced externally (restore flow) — `feedback_yjs_destroy_saves`.
- TipTap StarterKit v3: `history` renamed to `undoRedo`; custom commands need a `Commands<ReturnType>` declaration (`feedback_tiptap_starterkit`).
- `wiki_links` is derived, NOT synced — rebuilding it after a page_content apply is the only way cross-device links stay coherent.
- `page_content` BLOB sync uses a custom apply path (binary), not the generic schema dispatcher.

## Where NOT to look
- `src/lib/editor/YjsProvider.ts` is for local realtime editor state — not the sync-engine journal.
- `versions` table holds revision history; local-only, intentionally NOT synced.
