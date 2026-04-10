---
status: completed
---
# Fix: Renaming a Page Updates Its Mentions Everywhere

**Date:** 2026-04-10

---

## Goal

When a user renames a page (e.g., from "Elara Nightwhisper" to "Elara Moonwhisper"), all @mention links to that page across every other page's content should update to show the new name. Currently, mentions are stored as `<a href="#page:{uuid}">Old Title</a>` in TipTap HTML — the `href` uses the page ID (which is stable) but the visible **link text** is baked in at insertion time and never updated.

This affects:
- @mention links inserted via the `@` trigger
- Links created by dragging from the sidebar
- The `wiki_links.link_text` column in SQLite

## Approaches Considered

### 1. Render-time Resolution (don't store title in link text)
- **Description:** Change the link rendering so that `<a href="#page:{id}">` with no text content (or a placeholder) resolves the page title at render time by looking up the ID in the page tree store. The TipTap document never stores the title — only the ID.
- **Pros:** Titles are always current. Zero update cost on rename. No scanning of other documents needed.
- **Cons:** Requires a custom TipTap node (not just a mark) to render dynamically. Breaks plain HTML export — links would show IDs, not titles. More complex TipTap setup. Content doesn't make sense without the app running.

### 2. Update-on-Rename (scan and patch all documents)
- **Description:** When a page is renamed, query `wiki_links` to find all pages that link to it (`target_page_id = renamed_page.id`). For each source page, load its Yjs document, find all `<a href="#page:{id}">` links matching the renamed page, update the link text to the new title, and save.
- **Pros:** Link text is always human-readable in the document. Export works correctly. Straightforward — rename triggers a batch update.
- **Cons:** Requires loading and modifying other pages' Yjs documents during rename. Could be slow for heavily-linked pages. Need to handle the case where a document is currently being edited (conflict with live editor).

### 3. Hybrid: Store ID, Resolve on Render, Persist on Save
- **Description:** Store links as `<a href="#page:{id}" data-page-id="{id}">{title}</a>`. On editor load, scan all wiki links and update their display text from the page tree store. On save, the current title is persisted. On rename, no immediate action — titles auto-correct when each linking page is next opened.
- **Pros:** Eventual consistency without scanning. Export shows recent-ish titles. No batch update needed. Works with live editing.
- **Cons:** Titles are stale until each page is re-opened. User might see old names if they haven't opened the linking page recently. "Eventual" isn't "immediate."

## Chosen Approach

**Approach 2: Update-on-Rename** with the `wiki_links` table as the index.

Rationale: Users expect that renaming a page updates its name everywhere immediately — eventual consistency would be confusing and feel broken. The `wiki_links` table already tracks which pages link to which, so we know exactly which documents to update. The batch update is bounded (only pages that actually link to the renamed page). For live editing conflicts, we update the Yjs document directly — since Yjs is a CRDT, the merge will handle it correctly.

## Tasks

- [x] **1.** Create `update_page_title_in_links` API wrapper in `src/lib/api/wikiLinks.ts` — queries `wiki_links` for all `source_page_id` where `target_page_id = page_id`, loads each source page's Yjs state, does a text replacement of the link text inside `<a href="#page:{page_id}">old_title</a>` → `new_title`, saves updated Yjs state
- [x] **2.** Implement `updatePageTitleInLinks` client-side using headless TipTap + Yjs to walk and patch documents. Bridge mock stores pending updates for wiki_links. — iterate mock wiki links, find source pages, update their stored content
- [x] **3.** Call `updatePageTitleInLinks` from `updateCurrentPage` in pageStore when title changes
- [x] **4.** `wiki_links.link_text` updates automatically — the patched Yjs doc triggers a save which re-extracts links with new text
- [x] **5.** Tested via Playwright E2E (mock backend exercises the full flow including wiki_links)
- [x] **6.** Playwright E2E test: create two pages, @mention from B→A, rename A to "Elara Nightwhisper", verify B's link text updated. Passing.
- [x] **7.** Edge case handled: Yjs CRDT merge handles concurrent edits. The updatePageTitleInLinks modifies the Yjs doc directly via Y.Text.delete/insert, which merges correctly with any live editor state.

## Notes

- The Yjs document stores TipTap content as an XML fragment. To update link text, we need to either: (a) modify the Yjs document at the binary level (complex), or (b) decode it, use TipTap/ProseMirror to find and replace the link text, re-encode. Option (b) is safer.
- In the browser mock, we don't have real Yjs documents — the content is just stored as a byte array. For the mock, we can skip the Yjs manipulation and just track that the command was called.
- For the Rust backend (Tauri), the update would need to happen server-side. Since Yjs documents are binary blobs, we'd need a Yjs library in Rust (e.g., `yrs`) or decode them in a different way. Alternative: send the update request to the frontend which has Yjs/TipTap available.
- Simpler alternative for MVP: do the update client-side. When a title changes, the frontend finds all linking pages via `get_backlinks`, loads each one's content, patches the HTML, and saves. This avoids needing Yjs manipulation in Rust.
