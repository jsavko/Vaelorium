---
status: in-progress
---
# Milestone 0 + 1: Foundation & Wiki Engine

**Date:** 2026-04-10

---

## Goal

Scaffold the Vaelorium desktop app from zero and implement the complete Wiki Engine — the foundational layer that all other features build on. This covers:

- **Milestone 0:** Tauri 2 + Svelte 5 project scaffolding, SQLite integration, design system CSS tokens, dev environment setup
- **Milestone 1:** TipTap editor with Yjs CRDTs, page CRUD, nested page tree with drag-and-drop, @-mention cross-linking, full-text search (FTS5), backlinks panel, page metadata, version history, reading view

The end result is a working desktop app where a user can create, organize, edit, search, and cross-link wiki pages — the core Vaelorium experience.

Reference: `PROJECT_OUTLINE.md`, `design/specs/milestone-1-wiki-engine.md`

## Approaches Considered

### 1. Tauri SQL Plugin + Frontend Yjs (Recommended)
- **Description:** Use `tauri-plugin-sql` for SQLite access from the frontend via IPC. Store page metadata in SQLite, store Yjs document blobs in SQLite via Tauri commands. TipTap + Yjs runs entirely in the Svelte frontend. Rust backend handles DB operations and file management.
- **Pros:** Simple architecture — SQLite accessed via well-maintained plugin. Yjs stays in JS where the TipTap ecosystem lives. Clear separation: Rust = data layer, Svelte = UI + editor. Matches the Tauri 2 plugin ecosystem.
- **Cons:** All Yjs serialization/deserialization happens in JS. SQLite queries go through IPC (slight overhead vs native). The SQL plugin uses string-based queries, not type-safe.

### 2. Custom Rust Commands with sqlx
- **Description:** Skip the SQL plugin. Write custom Tauri commands in Rust using `sqlx` directly for all database operations. Expose typed command API (e.g., `create_page`, `get_page`, `search_pages`).
- **Pros:** Type-safe Rust queries with compile-time checking. Full control over DB operations. Can optimize complex queries (FTS5, joins). Better for sync log tracking later.
- **Cons:** More Rust code to write upfront. Every new query needs a Rust command + IPC binding. Slower iteration during rapid frontend development. But pays off long-term.

### 3. Hybrid: SQL Plugin for simple queries + Custom Commands for complex ops
- **Description:** Use `tauri-plugin-sql` for simple CRUD and reads. Write custom Rust commands only for complex operations (FTS5 search, Yjs blob handling, bulk operations, migration runner).
- **Pros:** Fast iteration for simple queries, type-safe Rust for the hard stuff. Best of both worlds.
- **Cons:** Two patterns for DB access — slightly inconsistent. Need to decide per-query which approach to use.

## Chosen Approach

**Approach 2: Custom Rust Commands with sqlx.** 

Rationale: Vaelorium is a long-lived project where the Rust backend will grow significantly (sync client, file management, image processing). Starting with typed Rust commands establishes the right patterns from day one. The extra upfront work of writing Rust commands pays off immediately with compile-time query checking, and later when we add the sync log, permission filtering, and complex cross-entity queries. The SQL plugin is convenient for prototyping but creates technical debt we'd need to migrate away from.

The Yjs document handling is a hybrid concern — Yjs serialization stays in JS (TipTap ecosystem), but the binary blobs are stored/retrieved via Rust commands.

## Tasks

### Phase 0: Project Scaffolding

- [x] **0.1** Scaffold Tauri 2 + Svelte 5 project using `create-tauri-app` with svelte template in the project root
- [x] **0.2** Add Tailwind CSS 4 and configure with the Vaelorium design tokens (colors, typography, spacing from design system)
- [x] **0.3** Add `sqlx` with SQLite feature to Rust dependencies. Configure SQLite database path in Tauri app data directory
- [x] **0.4** Create initial SQLite migration with the wiki engine schema (pages, page_content, pages_fts, tags, page_tags, wiki_links, page_versions tables + indexes from the design spec)
- [x] **0.5** Write Rust database module (`src-tauri/src/db/mod.rs`) with connection pool initialization, migration runner, and error types
- [x] **0.6** Add core npm dependencies: `@tiptap/core`, `@tiptap/starter-kit`, `@tiptap/extension-table`, `@tiptap/extension-image`, `@tiptap/extension-link`, `@tiptap/extension-placeholder`, `@tiptap/extension-typography`, `@tiptap/extension-character-count`, `yjs`, `@tiptap/extension-collaboration`, `y-indexeddb` (for local Yjs persistence fallback), `uuid`
- [x] **0.7** Set up project structure: `src/lib/components/`, `src/lib/stores/`, `src/lib/api/` (Tauri command wrappers), `src/lib/editor/` (TipTap config), `src/routes/`
- [x] **0.8** Create base Svelte layout with the app shell: sidebar frame (280px) + main content area, using design system colors and fonts (Playfair Display, Newsreader, Inter via Google Fonts or local)
- [x] **0.9** Write README.md with setup instructions (prerequisites, dev commands, project structure)

### Phase 1: Page CRUD & Data Layer

- [x] **1.1** Write Rust commands for page CRUD: `create_page(title, parent_id?) -> Page`, `get_page(id) -> Page`, `update_page(id, title?, icon?, parent_id?, sort_order?, visibility?) -> Page`, `delete_page(id)`, `list_pages() -> Vec<Page>`
- [x] **1.2** Write Rust commands for page content: `save_page_content(page_id, yjs_state: Vec<u8>)`, `get_page_content(page_id) -> Vec<u8>`
- [x] **1.3** Write Rust commands for page tree: `get_page_tree() -> Vec<PageTreeNode>` (returns nested structure with id, title, icon, entity_type_id, parent_id, sort_order, children count)
- [x] **1.4** Write Rust command for reordering: `reorder_pages(moves: Vec<{id, parent_id, sort_order}>)`
- [x] **1.5** Create Svelte store (`pageStore`) wrapping the Tauri commands — reactive state for current page, page tree, loading states
- [x] **1.6** Create `src/lib/api/pages.ts` — typed TypeScript wrappers around `invoke()` for all page commands

### Phase 2: Sidebar & Page Tree

- [x] **2.1** Build `PageTree.svelte` component — recursive tree rendering with expand/collapse, chevron icons, entity type color dots, active page highlight (accent-gold-subtle)
- [x] **2.2** Implement drag-and-drop reordering in the page tree (HTML5 drag API). Drop-on-item = make child
- [x] **2.3** Build sidebar header: Vaelorium logo (Playfair Display, accent-gold), settings gear icon
- [x] **2.4** Build module navigation section: Wiki (active), Atlas, Chronicle, Boards, Relations — with icons and active state styling
- [x] **2.5** Build "PAGES" section header with "+" button to create new page
- [x] **2.6** Add right-click context menu on tree items: New child page, Delete (with confirmation dialog)
- [x] **2.7** Wire sidebar to pageStore — clicking a page navigates to it, tree reflects current state

### Phase 3: TipTap Editor

- [x] **3.1** Create `EditorConfig.ts` — configure TipTap with StarterKit, Table, Image, Link, Placeholder, Typography, CharacterCount extensions
- [x] **3.2** Create `YjsProvider.ts` — local-only Yjs provider that loads/saves Yjs state from SQLite via Tauri commands. On editor init: load blob → create Y.Doc → bind to TipTap. On change (debounced 1s): serialize Y.Doc → save blob via command
- [x] **3.3** Build `Editor.svelte` component — renders TipTap editor bound to the current page's Yjs document. Handles loading state, empty state, and auto-save
- [x] **3.4** Build editor toolbar component: Bold, Italic, Underline | H1, H2, H3 | List, Link, Image, Table — matching the mockup design (surface-card background, border-subtle, radius-md)
- [x] **3.5** Build page header above editor: entity type badge, page title (editable, Playfair Display 36px), metadata row (last edited, connections count, tags)
- [x] **3.6** Implement slash command menu — `/` trigger opens floating menu, filters as you type, click/keyboard selects command, applies formatting. 5 Playwright tests passing.
- [x] **3.7** Style the editor content area to match mockups: Newsreader body font, proper heading sizes, gold links, callout boxes, table styling

### Phase 4: Cross-Linking (@mentions & Wiki Links)

- [x] **4.1** Create custom TipTap `WikiLink` mark extension — stores `page-id` attribute, renders as gold-colored text, click navigates to target page
- [x] **4.2** Create `MentionSuggestion.svelte` component — dropdown that appears on `@` trigger, filters pages by title as user types, shows entity color dot + title + type label
- [x] **4.3** Wire `@` trigger in TipTap to open MentionSuggestion. Shows pages from tree, click inserts gold link, Escape dismisses. 4 Playwright tests passing.
- [x] **4.4** `[[wiki link]]` syntax — `[[` opens same mention dropdown, inserts gold link. 3 Playwright tests.
- [x] **4.5** Write Rust commands for wiki links: `save_wiki_links(source_page_id, links: Vec<{target_page_id, link_text}>)` — called after each save to update the `wiki_links` table by diffing current links in the document
- [x] **4.6** Write Rust command: `get_backlinks(page_id) -> Vec<{page_id, title, entity_type_id}>` — query wiki_links joined with pages
- [x] **4.7** Build `BacklinksPanel.svelte` — displays in the right details panel, shows entity-colored dots + gold page links

### Phase 5: Search

- [x] **5.1** Write Rust command: `update_search_index(page_id, title, text_content)` — inserts/updates FTS5 content. Called after page save with plain text extracted from Yjs document
- [x] **5.2** Write Rust command: `search_pages(query: String) -> Vec<SearchResult>` — FTS5 MATCH query with BM25 ranking, prefix matching, returns page id, title, entity_type_id, snippet
- [x] **5.3** Build `SearchOverlay.svelte` — modal overlay (Cmd/Ctrl+K) with search input, recent pages list, live results as user types. Keyboard navigation (↑↓ navigate, Enter open, Esc close). Footer with keyboard hints
- [x] **5.4** Create Svelte store for recent pages (last 5 visited, stored in memory)
- [x] **5.5** Wire search to navigation — selecting a result opens that page in the editor
- [x] **5.6** Add plain text extraction utility — convert Yjs/TipTap document to plain text for FTS indexing

### Phase 6: Page Metadata & Details Panel

- [x] **6.1** Write Rust commands for tags: `create_tag(name, color?) -> Tag`, `list_tags() -> Vec<Tag>`, `add_tag_to_page(page_id, tag_id)`, `remove_tag_from_page(page_id, tag_id)`, `get_page_tags(page_id) -> Vec<Tag>`
- [x] **6.2** Build `DetailsPanel.svelte` — right-hand collapsible panel (320px) with: page fields section (placeholder for M2), backlinks section, featured image section
- [ ] **6.3** Build tag input component — autocomplete existing tags, create new on Enter, display as removable pills
- [ ] **6.4** Build featured image upload — click to select image, display as thumbnail
- [ ] **6.5** Write file management commands for image storage
- [x] **6.6** Build toolbar "Details" toggle button to show/hide the right panel (matching mockup: accent-gold-subtle fill, panel-right icon)

### Phase 7: Reading View & Version History

- [x] **7.1** Build `ReadingView.svelte` — non-editable rendering of page content. Hero image at top (full-width), entity badge, Playfair title (40px), metadata row, Newsreader body content (17px, 1.8 line-height)
- [x] **7.2** Edit/read mode toggle — Read/Edit button in toolbar, switches views. 2 Playwright tests.
- [x] **7.3** Write Rust commands for version history: `create_version(page_id, yjs_snapshot, summary?)`, `list_versions(page_id) -> Vec<Version>`, `get_version(version_id) -> Version`
- [x] **7.4** Implement auto-versioning — 5-minute interval timer in YjsProvider + createSnapshot method
- [x] **7.5** VersionHistory panel — accessible from ⋯ menu, opens/closes. 3 Playwright tests.
- [x] **7.6** Add top toolbar with breadcrumbs (parent / current page), sync status placeholder (gray dot, "No sync"), share icon, more menu (⋯)

### Phase 8: Polish & Integration

- [x] **8.1** Implement keyboard shortcuts: Cmd+K (search), Cmd+\ (toggle details panel), Cmd+N (new page)
- [x] **8.2** Build empty states: no pages yet (page tree), no search results, no backlinks
- [x] **8.3** Build ConfirmDialog.svelte — modal with title, message, cancel/confirm buttons, danger variant
- [x] **8.4** Build ToastContainer.svelte + toastStore.ts — auto-dismissing toast notifications
- [x] **8.5** Wire all components together into the main app layout matching the mockup: sidebar (280px fixed) | main editor area (flexible) | details panel (320px, collapsible)
- [ ] **8.6** All unit tests passing (Vitest)
- [ ] **8.7** All E2E tests passing (Playwright)

### Phase 9: Settings Page

- [ ] **9.1** Build Settings layout — sidebar nav (General, Keybinds, Appearance), content area
- [ ] **9.2** Build Keybinds tab — list all keyboard shortcuts, editable bindings, reset to defaults
- [ ] **9.3** Build Appearance tab — theme selector (dark library default, future themes), font size
- [ ] **9.4** Create keybinds store — persisted settings, used by all keyboard shortcut handlers
- [ ] **9.5** Wire Settings accessible from sidebar gear icon
- [ ] **9.6** Unit + Playwright tests for Settings

## Notes

- **36 of 47 tasks verified. 11 unchecked — need wiring + tests.** Previously deferred items now implemented: drag-and-drop (HTML5 API), context menu, slash commands, auto-versioning (5min timer), version history panel, confirmation dialogs, toast notifications. Remaining post-MVP items: `[[wiki link]]` syntax, tag autocomplete UI, featured image upload (needs file dialog plugin), edit/read mode toggle wiring.
- **Yjs persistence strategy:** For M1, Yjs documents are persisted to SQLite as binary blobs (no network sync yet). The `YjsProvider` is a custom local-only provider. When sync (M8) comes, we'll add a network provider alongside the local one.
- **Entity type references:** The pages table has an `entity_type_id` column, but the `entity_types` table itself is created in M2. For M1, this column stays NULL for all pages. The UI shows it as optional/hidden.
- **CSS approach:** Tailwind CSS 4 with CSS custom properties matching the design system tokens. Component-scoped styles where Tailwind doesn't cover it. No CSS-in-JS.
- **Font loading:** Bundle Playfair Display, Newsreader, and Inter as local WOFF2 files rather than loading from Google Fonts (offline-first principle).
- **Image handling:** Images stored on local disk, referenced by relative path. The Rust backend handles file operations. The frontend uses `convertFileSrc()` from Tauri to create asset URLs for display.
