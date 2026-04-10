# Design Spec: Milestone 6 — Boards (Whiteboards)

**Status:** Draft
**Mockups:** `design/vaelorium-boards.pen` (LoJ9M)
**Depends on:** Milestone 1 (Wiki Engine)

---

## Overview

Boards are freeform visual canvases for brainstorming, diagramming, and planning. Users place cards (standalone or linked to wiki pages), connect them with labeled edges, and add freehand drawings, text annotations, and images. Use cases: faction relationship diagrams, quest flowcharts, family trees, DM session prep screens.

---

## User Stories

- **As a GM**, I want to create a board to visually lay out relationships between factions.
- **As a GM**, I want to add cards to a board that link to wiki pages, showing the entity badge and a brief description.
- **As a GM**, I want to draw labeled connectors between cards to represent relationships (Allies, Enemies, etc.).
- **As a GM**, I want to add standalone text notes and sticky-note-style GM notes on the board.
- **As a GM**, I want to add freehand drawings and images to the board.
- **As a GM**, I want boards to be saved as part of my campaign and synced across devices.

---

## Data Model

### SQLite Schema

```sql
-- Boards
CREATE TABLE boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    sort_order INTEGER DEFAULT 0,
    viewport_x REAL DEFAULT 0,         -- saved pan position
    viewport_y REAL DEFAULT 0,
    viewport_zoom REAL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Board nodes (cards, text, images)
CREATE TABLE board_nodes (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    node_type TEXT NOT NULL,           -- 'card', 'text', 'note', 'image'
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,  -- for card type
    title TEXT,
    content TEXT,                      -- text content or image path
    x REAL NOT NULL,
    y REAL NOT NULL,
    width REAL DEFAULT 180,
    height REAL,                       -- NULL = auto-height based on content
    style TEXT,                        -- JSON: {color, borderColor, fontSize, ...}
    created_at TEXT NOT NULL
);

-- Board edges (connectors between nodes)
CREATE TABLE board_edges (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    source_node_id TEXT NOT NULL REFERENCES board_nodes(id) ON DELETE CASCADE,
    target_node_id TEXT NOT NULL REFERENCES board_nodes(id) ON DELETE CASCADE,
    label TEXT,                        -- "Allies", "Enemies", etc.
    color TEXT,                        -- hex color
    style TEXT DEFAULT 'solid',        -- 'solid', 'dashed', 'dotted'
    thickness REAL DEFAULT 2,
    created_at TEXT NOT NULL
);

-- Board drawings (freehand paths)
CREATE TABLE board_drawings (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    path_data TEXT NOT NULL,           -- SVG path string
    color TEXT DEFAULT '#E8DFD0',
    thickness REAL DEFAULT 2,
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_board_nodes_board ON board_nodes(board_id);
CREATE INDEX idx_board_edges_board ON board_edges(board_id);
CREATE INDEX idx_board_drawings_board ON board_drawings(board_id);
```

---

## Key Interactions

### Canvas

**Rendering:**
- Infinite canvas with pan (click-drag on empty area) and zoom (scroll wheel)
- Grid background (subtle dots) for alignment reference — toggleable
- Viewport state (pan/zoom) saved per board and restored on reopen

### Tool Palette (Toolbar)

| Tool | Icon | Behavior |
|------|------|----------|
| Card | square | Click canvas to place a card. Popover: link to page or standalone. |
| Connect | move-diagonal | Drag from one card to another to create an edge. Label popover appears. |
| Text | type | Click canvas to place a text annotation. Inline editing. |
| Draw | pencil | Freehand drawing mode. Click-drag to draw paths. |
| Image | image | Click canvas to place an image. File picker opens. |

**Selection:**
- Click a node/edge to select it (gold highlight border)
- Drag to multi-select
- Selected items can be moved, resized, or deleted (Delete/Backspace key)

### Cards

**Entity-linked cards (shown in mockup):**
- Entity type badge (colored pill with icon)
- Title (Playfair)
- Description preview (~50 chars, Inter)
- Border color matches entity type
- Double-click to open the linked wiki page

**Standalone cards:**
- User-defined title and content
- Default border color (border-default), user can customize

**GM Note cards (shown in mockup):**
- Dashed gold border
- Pencil emoji + "GM Note" header
- Text content
- Visually distinct from entity cards

### Connectors

- Drawn as lines between card edges (not centers — smart routing to nearest edges)
- Label appears as a small colored badge on the line midpoint
- Color and style (solid/dashed) are editable per connector
- Arrowhead on the target end (optional toggle)

### Board Sidebar

- List of boards with active board highlighted
- "+ New Board" button
- Right-click context menu: Rename, Duplicate, Delete

---

## Edge Cases

- **Deleted linked page:** Card remains but shows "Page deleted" in place of title. Border becomes dashed gray.
- **Empty board:** Shows "Add cards, notes, and connections to start building your diagram" centered prompt.
- **Many nodes (100+):** Viewport culling — only render nodes visible in the current viewport. Off-screen nodes excluded from DOM.
- **Overlapping cards:** Click selects the topmost card. Z-order based on creation time; "Bring to front" option in context menu.
- **Undo/Redo:** Local undo stack for board operations (add/move/delete/connect). Cmd+Z / Cmd+Shift+Z.

---

## Performance

- **Canvas rendering:** HTML/CSS for cards (flexbox layout). SVG or Canvas for connectors and drawings.
- **Large boards:** Viewport culling at 50+ nodes. Virtual rendering for 200+ nodes.
- **Freehand drawing:** Use pointer events with debounced path simplification. Store simplified SVG paths.

---

## Accessibility

- **Keyboard:** Tab cycles through nodes in creation order. Arrow keys to fine-position selected node. Enter opens card. Delete removes selected.
- **Screen reader:** Boards provide a companion "Board Outline" panel listing all nodes and connections in text form.
- **Connector creation:** Keyboard alternative — select source card, press C, select target card, choose label from dropdown.
