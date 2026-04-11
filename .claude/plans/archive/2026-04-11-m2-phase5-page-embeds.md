---
status: completed
---
# M2 Phase 5: Page Embeds (Transclusion)

**Date:** 2026-04-11

---

## Goal

Enable embedding one page's content inline inside another page. Embedded content renders read-only and updates when the source changes. Useful for reusable lore snippets, NPC descriptions that appear in multiple locations, session recaps, etc.

**References:**
- Parent plan: `.claude/plans/2026-04-11-m2-entity-type-system.md`
- Similar to: Notion synced blocks, Obsidian `![[Page Name]]`

## Chosen Approach

Custom TipTap node extension (`PageEmbed`) that stores a `page_id` attribute. Triggered via `/embed` slash command or `![[Page Name]]` syntax. Renders as a bordered card showing the source page's content read-only, with a link to open the source page.

## Tasks

- [x] **1.** Create `PageEmbed` TipTap node extension — custom node with `page_id` attribute, renders as a non-editable block in the editor.

- [x] **2.** Build `PageEmbedView.svelte` — NodeView component that loads the referenced page's content and renders it as read-only TipTap output inside a styled card. Shows page title as header with entity type badge, and a "Open page" link.

- [x] **3.** Add `/embed` to slash commands — opens a page search dropdown (reuse the mention/search pattern), selecting a page inserts a `PageEmbed` node.

- [x] **4.** Add `![[Page Name]]` trigger syntax — similar to `[[` wiki link syntax but with `!` prefix. Typing `![[` opens the page search dropdown, selecting inserts a `PageEmbed` node instead of a link.

- [x] **5.** Handle embed rendering — the embedded content should update reactively when the source page changes. Load page content via the existing `getPageContent` API.

- [x] **6.** Style the embed card — bordered container with subtle background, page title header (Playfair), read-only content body (Newsreader), entity type badge if applicable, "Open" link in header.

- [x] **7.** Prevent recursive embeds — if Page A embeds Page B, Page B should not be able to embed Page A (or detect and break cycles with a "Recursive embed" placeholder).

- [x] **8.** Write Playwright E2E tests:
  - Insert embed via /embed slash command
  - Embedded page content is visible
  - Click "Open" link navigates to source page
  - Recursive embed shows placeholder

- [x] **9.** Verify all existing tests pass.

## Notes

- Embedded content is read-only inside the host page — editing the source page updates all embeds.
- The embed node stores only the `page_id`, not the content. Content is loaded dynamically.
- For the initial version, embed the full page content. Section-level embedding (embed only a heading section) can come later.
- The TipTap NodeView API supports Svelte components for custom rendering.
