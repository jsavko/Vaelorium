---
status: created
---
# M1 Wiki Engine — Gap Analysis & Fix Plan

**Date:** 2026-04-10

---

## Mockup vs Reality Comparison

### Screen 1: Main Editor View (ZSWxH)

| Mockup Feature | Status | Notes |
|----------------|--------|-------|
| Sidebar: Vaelorium logo in gold | DONE | Working |
| Sidebar: Module nav (Wiki/Atlas/Chronicle/Boards/Relations) | DONE | Working, Wiki highlighted |
| Sidebar: Page tree with expand/collapse chevrons | PARTIAL | Tree renders, expand works. Chevrons need visual cleanup. |
| Sidebar: Entity type color dots in tree | DONE | Dot renders (gray default, colored when typed) |
| Sidebar: Active page gold highlight | DONE | Working |
| Sidebar: Drag-and-drop reorder | DONE | Drop-on-item = make child |
| Sidebar: Right-click context menu | DONE | New child + Delete with confirm |
| Top toolbar: Breadcrumbs (parent / current page) | PARTIAL | Shows title only, no parent chain |
| Top toolbar: Sync status (green dot + "Synced") | DONE | Shows gray "No sync" |
| Top toolbar: Share button | MISSING | Not implemented |
| Top toolbar: More menu (⋯) | MISSING | Not implemented |
| Editor: Entity type badge above title | PARTIAL | Shows if entity_type_id set, but no colored badges |
| Editor: Editable Playfair title (36px) | DONE | Working |
| Editor: Metadata row (last edited, connections, tags) | PARTIAL | Shows last edited date only. No connection count, no tags. |
| Editor: TipTap toolbar (B/I/U/H1/H2/H3/List/Link/Image/Table) | PARTIAL | Has B/I/H1/H2/H3/List. Missing U, Link, Image, Table buttons |
| Editor: Rich text body (Newsreader) | DONE | Working |
| Editor: Section headings (Playfair) | DONE | H1/H2/H3 styled |
| Editor: Wiki link callout box | MISSING | Callout block extension not wired |
| Editor: Relationship cards at bottom | OUT OF SCOPE | Milestone 3 |
| Editor: Slash commands (/) | DONE | Working with filter, 5 tests passing |

### Screen 2: Search Overlay (iefLD)

| Mockup Feature | Status | Notes |
|----------------|--------|-------|
| Cmd+K opens overlay | DONE | Working |
| Dimmed background | DONE | Working |
| Search input with placeholder | DONE | Working |
| ⌘K shortcut badge | PARTIAL | Shows "Esc" badge, not ⌘K |
| RECENT PAGES default list | DONE | Working |
| Entity color dot per result | PARTIAL | Shows gray dot, not entity-colored |
| Type label right-aligned | MISSING | No type label shown |
| First result highlighted | DONE | Working |
| Arrow key navigation | DONE | Working |
| Enter to open | DONE | Working |
| Footer keyboard hints | DONE | Working |
| Live search results | DONE | Mock search works on title match |

### Screen 3: @Mention Linking (1qtOm)

| Mockup Feature | Status | Notes |
|----------------|--------|-------|
| @ trigger opens dropdown | DONE | Working |
| "LINK TO PAGE" header | DONE | Working |
| Entity color dots | PARTIAL | Shows gray, not entity-colored |
| Page title + entity type label | PARTIAL | Title shown, type not shown |
| Gold highlight on trigger text | MISSING | @trigger not visually highlighted |
| Click inserts gold link | DONE | Working |
| Link click navigates to page | DONE | Just fixed |

### Screen 4: Metadata & Backlinks Panel (aB4kG)

| Mockup Feature | Status | Notes |
|----------------|--------|-------|
| "Details" toggle button | DONE | Working |
| Panel header with close X | DONE | Working |
| CHARACTER FIELDS section | OUT OF SCOPE | Milestone 2 |
| Race/Class/Alignment/Status fields | OUT OF SCOPE | Milestone 2 |
| Location/Organisation links | OUT OF SCOPE | Milestone 2 |
| BACKLINKS section | BROKEN | Shows "No other pages link here" even when links exist |
| Backlink: entity dot + gold link | NOT TESTABLE | Because backlinks aren't populated |
| FEATURED IMAGE section | MISSING | Not implemented |

### Screen 5: Reading View (NuEKV)

| Mockup Feature | Status | Notes |
|----------------|--------|-------|
| Hero image at top | MISSING | ReadingView component exists but not wired |
| Entity badge | MISSING | Not shown in reading view |
| Large Playfair title (40px) | MISSING | Component exists but no toggle |
| Metadata row (Race/Class/etc) | OUT OF SCOPE | Milestone 2 |
| Gold "Edit" button | MISSING | No edit/read toggle |
| Newsreader body (17px, 1.8lh) | MISSING | Component exists but not accessible |

---

## Bugs to Fix

1. **Backlinks not populating** — @mention inserts a link but doesn't call `save_wiki_links`. Need to extract links from Yjs doc on save and update wiki_links table.
2. **SQLite multi-connection mode** — Need to ensure WAL mode + proper pool settings for concurrent read/write.
3. **Toolbar missing buttons** — Need Underline, Link, Image, Table buttons in the toolbar.
4. **Breadcrumbs incomplete** — Need to show parent page chain, not just current title.
5. **Entity color not used in search/mention results** — Always shows gray dot instead of entity type color.

## Features to Wire (have components, need integration)

6. **Reading view toggle** — ReadingView.svelte exists but no way to access it. Need edit/read mode toggle button in toolbar.
7. **Version history access** — VersionHistory.svelte exists but no way to open it. Need menu item in ⋯ more menu.
8. **[[wiki link]] syntax** — Second trigger for mentions using `[[` instead of `@`.
9. **Tag input in details panel** — Backend ready, need autocomplete UI component.
10. **Featured image in details panel** — Need file picker and display.
11. **Share button** — Placeholder for now.
12. **More menu (⋯)** — Dropdown with: Version history, Delete page, Copy link.

## New Features Needed

13. **Settings page** — Keybinds tab + Appearance/Themes tab. Accessible from sidebar gear icon.
14. **Callout block extension** — TipTap extension for info/warning/note callout boxes.

---

## Priority Order

### P0: Critical Bugs — ALL DONE
- [x] Fix backlinks: extract wiki links from doc on save — commit 5b13bb9
- [x] Fix SQLite multi-connection: PRAGMAs per-connection — commit 5b13bb9

### P1: Missing from Mockups (visible gaps) — ALL DONE
- [x] Add toolbar buttons: Underline, Link, Image, Table — commit 7f0afbf
- [x] Add ⋯ more menu with Version History and Delete — commit 7f0afbf
- [x] Wire reading view with edit/read toggle — commit 7f0afbf
- [x] Fix breadcrumbs to show parent chain — commit 7f0afbf
- [x] Entity type colors in search/mention — already wired, shows gray for untyped pages (correct for M1)
- [x] Type label in search results — already implemented, shows when entity_type_id is set

### P2: Feature Completion
- [ ] Wire [[wiki link]] syntax trigger
- [ ] Build tag input component in details panel  
- [ ] Build featured image upload in details panel
- [ ] Build callout block TipTap extension
- [ ] Build settings page (keybinds + appearance)

### P3: Polish
- [ ] Settings gear icon in sidebar
- [ ] Share button placeholder
- [ ] Connection count in metadata row
- [ ] ⌘K badge in search (not just Esc)
