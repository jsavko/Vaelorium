# Design Spec: Milestone 1 — Wiki Engine

**Status:** Draft
**Mockups:** `design/vaelorium-wiki-engine.pen` (ZSWxH, iefLD, 1qtOm, aB4kG, NuEKV)

---

## Overview

The Wiki Engine is the foundational layer of Vaelorium. Every piece of content in the app is a wiki page. Pages support rich text editing, nested hierarchy, cross-linking, full-text search, version history, and metadata. All other features (entity types, maps, timelines, boards) build on top of this page primitive.

---

## User Stories

### Core Editing
- **As a GM**, I want to create a new page so I can document a character, location, or piece of lore.
- **As a GM**, I want to write rich text content (headings, bold, italic, lists, tables, callouts, images) so my world documentation is expressive and readable.
- **As a GM**, I want to organize pages in a nested tree so I can group related content (e.g., all characters under a "Characters" folder).
- **As a GM**, I want to drag and drop pages in the tree to reorganize my world structure.

### Cross-Linking
- **As a GM**, I want to type `@` in the editor and see a dropdown of matching pages so I can quickly link to other content.
- **As a GM**, I want to use `[[page name]]` syntax as an alternative way to create wiki links.
- **As a GM**, I want to see a backlinks panel showing all pages that reference the current page so I can understand how content is connected.

### Search
- **As a GM**, I want to press `Cmd/Ctrl+K` to open a search overlay so I can quickly navigate to any page.
- **As a GM**, I want search to be instant and include full-text content search, not just page titles.
- **As a GM**, I want search results to show the entity type (Character, Location, etc.) so I can distinguish between pages with similar names.

### Reading & Sharing
- **As a player**, I want to view pages shared with me in a clean reading view without editing controls.
- **As a GM**, I want to toggle between edit mode and reading mode to preview how a page looks to players.

### Metadata & Organization
- **As a GM**, I want to add tags to pages so I can categorize and filter content.
- **As a GM**, I want to set a featured image on a page so it appears in cards and the reading view hero.
- **As a GM**, I want to see when a page was last edited and how many connections it has.

### Version History
- **As a GM**, I want to see the version history of a page so I can review or revert changes.
- **As a GM**, I want to see a diff between versions so I can understand what changed.

---

## Data Model

### SQLite Schema

```sql
-- Pages table: the core content unit
CREATE TABLE pages (
    id TEXT PRIMARY KEY,              -- UUID
    title TEXT NOT NULL,
    icon TEXT,                         -- emoji or icon identifier
    featured_image_path TEXT,          -- relative path to local file
    parent_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    sort_order INTEGER DEFAULT 0,      -- for ordering within parent
    entity_type_id TEXT REFERENCES entity_types(id), -- NULL = untyped page
    visibility TEXT DEFAULT 'private', -- 'private', 'players', 'public'
    created_at TEXT NOT NULL,          -- ISO 8601
    updated_at TEXT NOT NULL,          -- ISO 8601
    created_by TEXT,                   -- user identifier
    updated_by TEXT                    -- user identifier
);

-- Page content stored as Yjs binary for CRDT sync
CREATE TABLE page_content (
    page_id TEXT PRIMARY KEY REFERENCES pages(id) ON DELETE CASCADE,
    yjs_state BLOB NOT NULL,           -- Yjs document state
    yjs_version INTEGER DEFAULT 0      -- monotonic version counter
);

-- Full-text search index
CREATE VIRTUAL TABLE pages_fts USING fts5(
    title,
    text_content,                      -- plain text extracted from Yjs doc
    content='pages_fts_content',
    content_rowid='rowid'
);

-- Backing table for FTS content
CREATE TABLE pages_fts_content (
    rowid INTEGER PRIMARY KEY AUTOINCREMENT,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    text_content TEXT NOT NULL          -- plain text, updated on save
);

-- Tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT                          -- hex color, optional
);

CREATE TABLE page_tags (
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (page_id, tag_id)
);

-- Wiki links (cross-references between pages)
CREATE TABLE wiki_links (
    source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    link_text TEXT,                     -- display text of the link
    PRIMARY KEY (source_page_id, target_page_id)
);

-- Version history snapshots
CREATE TABLE page_versions (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    yjs_snapshot BLOB NOT NULL,        -- Yjs snapshot for this version
    version_number INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    created_by TEXT,
    summary TEXT                        -- auto-generated or user-provided
);

-- Indexes
CREATE INDEX idx_pages_parent ON pages(parent_id);
CREATE INDEX idx_pages_entity_type ON pages(entity_type_id);
CREATE INDEX idx_pages_updated ON pages(updated_at DESC);
CREATE INDEX idx_wiki_links_target ON wiki_links(target_page_id);
CREATE INDEX idx_page_versions_page ON page_versions(page_id, version_number DESC);
```

### Yjs Document Structure

Each page's rich text content is stored as a Yjs document with the following structure:

```
Y.Doc
  └── "content" (Y.XmlFragment)     -- TipTap document content
       ├── <paragraph>
       ├── <heading level=2>
       ├── <paragraph>
       │    └── <mention page-id="...">  -- wiki link marks
       ├── <callout type="info">
       ├── <table>
       └── ...
```

The Yjs document is serialized to binary (`Y.encodeStateAsUpdate()`) for storage in the `page_content.yjs_state` column. On load, it's decoded back into a live Yjs document that TipTap binds to.

### File Storage

Images and attachments are stored on the local filesystem:

```
{app_data}/
  campaigns/
    {campaign_id}/
      images/
        {page_id}/
          featured.webp
          {attachment_id}.webp
```

---

## TipTap Editor Configuration

### Core Extensions
- `StarterKit` — paragraphs, headings (H1-H3), bold, italic, strike, code, blockquote, bullet list, ordered list, horizontal rule
- `Table` + `TableRow` + `TableHeader` + `TableCell` — table support
- `Image` — inline and block images
- `Link` — external hyperlinks
- `Placeholder` — "Start writing..." placeholder text
- `Typography` — smart quotes, em dashes
- `CharacterCount` — optional word/character count

### Custom Extensions
- **WikiLink** — custom mark for `@mention` and `[[wiki link]]` cross-references. Renders as a gold-colored link that opens the target page on click. Stores `page-id` as an attribute.
- **Callout** — custom node for info/warning/note callout boxes (as shown in mockup with gold info icon).
- **SecretBlock** — custom node for GM-only content sections. Rendered with a dashed border and eye icon. Hidden from players based on visibility settings. (Milestone 7 dependency, but the node type should be defined now.)

### Yjs Integration
- `Collaboration` extension with Yjs provider
- Document stored as Yjs state, enabling offline editing with CRDT merge on sync
- Version snapshots captured on explicit save or at timed intervals (e.g., every 5 minutes of active editing)

### Slash Commands
The editor supports `/` slash commands for quick insertion:
- `/heading` → Insert heading (H1, H2, H3)
- `/list` → Bullet or ordered list
- `/table` → Insert table
- `/callout` → Insert callout block
- `/image` → Insert image from file picker
- `/divider` → Horizontal rule
- `/link` → Insert wiki link (opens page search)

---

## Key Interactions

### Page Tree (Sidebar)

**Structure:**
- Root-level folders and pages
- Expandable/collapsible with chevron icons
- Active page highlighted with `accent-gold-subtle` background
- Entity type indicated by colored dot next to page name

**Drag & Drop:**
- Pages can be dragged to reorder within a parent or moved to a different parent
- Drop targets highlighted during drag
- Dropping on a page makes it a child; dropping between pages reorders

**Context Menu (right-click):**
- New child page
- Rename
- Duplicate
- Move to...
- Change entity type
- Delete (with confirmation)

### @Mention / Wiki Link

**Trigger:** Typing `@` in the editor opens the mention dropdown.

**Behavior:**
1. Dropdown appears below the cursor
2. Shows "LINK TO PAGE" header
3. Results filter as user types (fuzzy match on title)
4. Each result shows: entity color dot, page title, entity type label
5. First result is pre-selected (navigable with arrow keys)
6. Enter or click inserts a WikiLink mark into the document
7. Escape dismisses the dropdown

**Alternative:** Typing `[[` opens the same dropdown. Closing `]]` is auto-inserted on selection.

**Rendering:** Wiki links appear as gold-colored text (`accent-gold`). Hovering shows a mini-preview tooltip. Clicking navigates to the target page.

### Search (Cmd/Ctrl+K)

**Behavior:**
1. Modal overlay with dimmed background
2. Search input auto-focused with placeholder "Search the archives..."
3. Shows "RECENT PAGES" by default (last 5 visited)
4. As user types, switches to live search results
5. Results show: entity color dot, page title, entity type label right-aligned
6. First result highlighted; arrow keys navigate, Enter opens
7. Footer shows keyboard hints: ↑↓ Navigate, ⏎ Open, Esc Close

**Search implementation:**
- SQLite FTS5 with `MATCH` query on title and text_content
- Results ranked by BM25 relevance score
- Prefix matching enabled (typing "Ela" matches "Elara")
- Recent pages stored as a simple list in local app state (not persisted to DB)

### Backlinks Panel

**Location:** Right-hand details panel under "BACKLINKS" section (see metadata panel mockup).

**Data source:** Query `wiki_links` table where `target_page_id = current_page.id`, joined with `pages` to get titles and entity types.

**Display:** Each backlink shows entity color dot + page title as a gold link. Clicking navigates to that page.

### Version History

**Access:** Via page menu (⋯ button) → "Version history"

**Display:** Side panel or modal showing a list of versions with:
- Version number
- Timestamp
- Author
- Auto-generated summary (e.g., "Added 3 paragraphs, edited heading")

**Diff view:** Selecting a version shows a side-by-side or inline diff against the current version. Yjs snapshots are decoded and compared.

**Restore:** "Restore this version" creates a new version with the old content (non-destructive).

---

## Edge Cases & Error States

### Empty States
- **No pages yet:** Page tree shows "Create your first page" prompt with a gold "New Page" button
- **No search results:** "No pages found" with suggestion to create a new page
- **No backlinks:** "No other pages link to this one yet" message
- **No tags:** Tag section hidden until first tag is added

### Error States
- **Failed to save:** Toast notification "Changes saved locally. Sync pending." (offline-first — local save should never fail)
- **Broken wiki link:** If a linked page is deleted, the WikiLink mark renders with a strikethrough and red color. Tooltip says "This page was deleted."
- **Image load failure:** Placeholder with broken-image icon and "Image not found" text

### Conflict Handling
- **Concurrent edits (sync):** Yjs CRDT handles merge automatically. No user-facing conflict UI needed for rich text.
- **Concurrent metadata edits:** Last-write-wins for simple fields (title, icon, tags). Shown via sync status indicator.

### Performance
- **Large pages:** TipTap handles documents up to ~50,000 words without degradation. For larger pages, consider lazy-rendering sections.
- **Large page trees:** Virtual scrolling for trees with 500+ pages. Only render visible nodes.
- **Search indexing:** FTS5 index updated asynchronously after page save (debounced 1s). Search is always fast; index may lag by 1-2 seconds after edit.

---

## Accessibility

- **Keyboard navigation:** Full keyboard support for page tree (arrow keys to navigate, Enter to open, Space to expand/collapse)
- **Editor shortcuts:** Standard rich text shortcuts (Cmd+B, Cmd+I, etc.) plus custom shortcuts for wiki links (Cmd+Shift+K)
- **Search:** Cmd/Ctrl+K opens search from anywhere. Full keyboard navigation within results.
- **Focus management:** Focus trapped in modals (search overlay). Returned to previous position on close.
- **Screen reader:** Page tree uses `role="tree"` / `role="treeitem"`. Editor uses TipTap's built-in ARIA support. Search results use `role="listbox"`.
- **Color contrast:** All text meets WCAG AA contrast ratios against the dark walnut background (verified in design system).

---

## Technical Dependencies

- **Tauri 2** — Desktop shell, Rust backend, IPC for SQLite access
- **Svelte 5** — Frontend framework
- **TipTap** — Rich text editor (ProseMirror-based)
- **Yjs** — CRDT for document state and sync
- **SQLite** — Local database via Tauri SQL plugin
- **SQLite FTS5** — Full-text search

---

## Out of Scope (handled in later milestones)

- Entity type structured fields (Milestone 2)
- Relations between pages (Milestone 3)
- Map pins linking to pages (Milestone 4)
- Timeline event linking (Milestone 5)
- Permission visibility filtering (Milestone 7)
- Sync server integration (Milestone 8)
- Import/Export (Milestone 9)
