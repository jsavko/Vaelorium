# Design Spec: Milestone 3 — Relations & Connections

**Status:** Draft
**Mockups:** `design/vaelorium-relations.pen` (2huCP)
**Depends on:** Milestone 1 (Wiki Engine), Milestone 2 (Entity Types)

---

## Overview

Relations are typed, bidirectional connections between any two pages. They enable a visual network graph of how entities in the world relate to each other. Relations are first-class data — queryable, filterable, and displayed on entity pages and in a dedicated graph view.

---

## User Stories

- **As a GM**, I want to create a relation between two pages (e.g., "Elara is the High Priestess of The Silver Flame") so I can track how entities connect.
- **As a GM**, I want relations to be bidirectional — if A relates to B, B relates back to A (with an inverse label).
- **As a GM**, I want to define custom relation types (e.g., "Leader of", "Resides at", "Mentor of", "Enemy of").
- **As a GM**, I want to see all relations for a page in a list on that page's detail panel.
- **As a GM**, I want to explore relations visually in a graph/network view showing entities as nodes and relations as edges.
- **As a GM**, I want to filter the graph by entity type to focus on specific connections (e.g., only Characters and Organisations).
- **As a GM**, I want to click a node in the graph to navigate to that page.

---

## Data Model

### SQLite Schema

```sql
-- Relation type definitions
CREATE TABLE relation_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,               -- "Leader of"
    inverse_name TEXT,                -- "Led by" (for the reverse direction)
    color TEXT,                       -- hex color for the edge
    is_builtin BOOLEAN DEFAULT FALSE,
    created_at TEXT NOT NULL
);

-- Relations between pages
CREATE TABLE relations (
    id TEXT PRIMARY KEY,
    source_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    target_page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    relation_type_id TEXT NOT NULL REFERENCES relation_types(id) ON DELETE CASCADE,
    description TEXT,                 -- optional note on this specific relation
    created_at TEXT NOT NULL,
    created_by TEXT
);

-- Indexes
CREATE INDEX idx_relations_source ON relations(source_page_id);
CREATE INDEX idx_relations_target ON relations(target_page_id);
CREATE INDEX idx_relations_type ON relations(relation_type_id);
```

### Built-in Relation Types

| Name | Inverse Name | Color |
|------|-------------|-------|
| Leader of | Led by | #C8A55C (gold) |
| Member of | Has member | #8B5CB8 (purple) |
| Resides at | Home of | #4A8C6A (green) |
| Located in | Contains | #4A8C6A (green) |
| Ally of | Ally of | #5C8A5C (success green) |
| Enemy of | Enemy of | #B85C5C (red) |
| Mentor of | Mentored by | #5C7AB8 (blue) |
| Parent of | Child of | #B8955C (amber) |
| Owns | Owned by | #B8955C (amber) |
| Created by | Created | #5CB8A8 (teal) |

---

## Key Interactions

### Creating Relations

**From a page's detail panel:**
1. Click "+ Add Relation" button in the relations section
2. Inline form appears: [Page search] [Relation type dropdown] [Page search]
3. Source defaults to current page
4. User selects target page and relation type
5. Relation created immediately; appears in both pages' relation lists

**From the graph view:**
1. Drag from one node to another
2. Relation type picker appears
3. Select type and confirm

### Relations Panel (on entity pages)

**Location:** Below backlinks in the right details panel, or as inline cards in the wiki content area (as shown in the main editor mockup).

**Display:** Each relation shows:
- Entity color dot (based on target page type)
- Target page name (gold link)
- Relation label in tertiary text (e.g., "— High Priestess of")
- Click to navigate to target page

### Graph View

**Layout:** Full-screen canvas (sidebar + graph area as shown in mockup).

**Nodes:**
- Pill-shaped with entity-colored border
- Entity type color dot + page title
- Selected/focused node has gold glow border
- Double-click opens the page

**Edges:**
- Lines connecting related nodes
- Relation type color
- Italic label on hover or always visible (user toggle)
- Directional (arrow on the line, subtle)

**Controls:**
- Pan: Click and drag on canvas
- Zoom: Scroll wheel, or +/- buttons in toolbar
- Filter: Dropdown to show/hide entity types
- Layout: Force-directed auto-layout with manual drag override
- Legend: Bottom-left showing entity type colors

**Graph Layout Algorithm:**
- Force-directed layout (d3-force or similar) as initial positioning
- User can drag nodes to manual positions; positions are saved per-page
- Re-layout button resets to force-directed calculation

**Performance:**
- Graphs with up to 200 nodes and 500 edges should render smoothly
- For larger graphs, cluster distant nodes or paginate by proximity to selected node
- Use Canvas/WebGL rendering (not SVG) for large graphs

---

## Edge Cases

- **Self-referential relation:** Allowed (a page can relate to itself, e.g., "Conflicted with self")
- **Duplicate relations:** Same source, target, and type not allowed. Different types between same pages are allowed.
- **Deleted pages:** When a page is deleted, all its relations are cascade-deleted.
- **Orphan nodes in graph:** Pages with no relations don't appear in the graph by default. Toggle to show all pages.
- **Large graphs:** If a campaign has 500+ entities, default the graph to show only entities connected to the currently selected page (ego graph, depth 2).

---

## Accessibility

- **Graph view:** Not inherently accessible to screen readers. Provide an alternative "Relation List" view (table format) accessible via tab in the toolbar.
- **Keyboard:** Tab between nodes, Enter to select/open, arrow keys to traverse edges.
- **Relation creation:** Fully keyboard-accessible form with proper focus management.
