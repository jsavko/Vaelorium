# Design Spec: Milestone 4 — Interactive Maps (Atlas)

**Status:** Draft
**Mockups:** `design/vaelorium-maps.pen` (PPpgZ)
**Depends on:** Milestone 1 (Wiki Engine)

---

## Overview

The Atlas is Vaelorium's interactive map system. Users upload map images, place pins linked to wiki pages, organize maps in a nested hierarchy (world → region → city → building), and control visibility per layer and role.

---

## User Stories

- **As a GM**, I want to upload a map image (up to 14K pixels) so I can visualize my world geography.
- **As a GM**, I want to place pins on the map, each linked to a wiki page.
- **As a GM**, I want to customize pin icons, colors, and labels.
- **As a GM**, I want to nest maps (clicking a region pin opens a more detailed regional map).
- **As a GM**, I want to organize pins into layers (Locations, Points of Interest, GM Only) that can be toggled on/off.
- **As a GM**, I want to click a pin to see a tooltip preview of the linked page.
- **As a player**, I want to only see pins and layers that the GM has made visible to me.

---

## Data Model

### SQLite Schema

```sql
-- Maps
CREATE TABLE maps (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    image_path TEXT NOT NULL,          -- relative path to map image file
    parent_map_id TEXT REFERENCES maps(id) ON DELETE SET NULL,
    parent_pin_id TEXT REFERENCES map_pins(id) ON DELETE SET NULL,  -- pin that opens this child map
    width INTEGER,                     -- image width in pixels
    height INTEGER,                    -- image height in pixels
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Map layers
CREATE TABLE map_layers (
    id TEXT PRIMARY KEY,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    visible BOOLEAN DEFAULT TRUE,
    visibility TEXT DEFAULT 'all',     -- 'all', 'gm_only', or specific role
    sort_order INTEGER DEFAULT 0
);

-- Map pins
CREATE TABLE map_pins (
    id TEXT PRIMARY KEY,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    layer_id TEXT REFERENCES map_layers(id) ON DELETE SET NULL,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,  -- linked wiki page
    child_map_id TEXT REFERENCES maps(id) ON DELETE SET NULL, -- drill-down map
    label TEXT,
    icon TEXT DEFAULT 'map-pin',       -- lucide icon name
    color TEXT,                        -- hex color (defaults to entity type color)
    x REAL NOT NULL,                   -- normalized position 0-1 (fraction of map width)
    y REAL NOT NULL,                   -- normalized position 0-1 (fraction of map height)
    size TEXT DEFAULT 'medium',        -- 'small', 'medium', 'large'
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_maps_parent ON maps(parent_map_id);
CREATE INDEX idx_map_layers_map ON map_layers(map_id);
CREATE INDEX idx_map_pins_map ON map_pins(map_id);
CREATE INDEX idx_map_pins_page ON map_pins(page_id);
```

### File Storage

```
{app_data}/
  campaigns/
    {campaign_id}/
      maps/
        {map_id}/
          original.webp      -- full resolution upload
          tiles/              -- generated tiles for large maps (future)
            {zoom}/{x}_{y}.webp
```

---

## Key Interactions

### Map Viewer

**Rendering:**
- Map image displayed with pan (click-drag) and zoom (scroll wheel)
- For images > 4096px, use progressive loading or tiled rendering
- Pin positions stored as normalized coordinates (0-1) so they scale with any zoom level

**Pin display:**
- Pins render as icons (lucide) with the entity type color
- Label appears below the pin
- Pins scale slightly with zoom (not linearly — they stay readable at all zoom levels)

**Pin tooltip (click):**
- Clicking a pin shows a floating card preview (as shown in mockup):
  - Featured image thumbnail
  - Entity type badge
  - Page title (Playfair)
  - First ~100 chars of page content
  - "Open page →" gold link
- If pin links to a child map: "Open map →" replaces "Open page →"

**Navigation breadcrumbs:**
- Top bar shows: World Map / Region Name / City Name
- Click any breadcrumb to navigate up the hierarchy

### Adding Pins

1. Click "Add Pin" button in toolbar
2. Cursor changes to crosshair
3. Click on map to place pin
4. Pin creation popover appears:
   - Link to page (search input) or create new page
   - Layer selection dropdown
   - Icon picker (optional)
   - Custom label (defaults to page title)
5. Save creates the pin; cancel removes it

### Map Sidebar

**Map Tree:**
- Hierarchical list of maps with expand/collapse
- Active map highlighted
- Drag to reorder or re-parent maps

**Layers:**
- Toggle checkbox per layer
- Checked layers show their pins; unchecked hides them
- "GM Only" layer has a distinct unchecked state
- Layer order determines render order (top layer renders on top)

### Map Upload

- Supported formats: PNG, JPEG, WebP
- Max file size: 100MB
- Max dimensions: 16384 x 16384 pixels
- Images are converted to WebP on import for storage efficiency
- Upload via drag-and-drop onto the map canvas or via "Upload Map" in sidebar

---

## Edge Cases

- **Deleted linked page:** Pin remains on map but tooltip shows "Page deleted" with option to relink or remove pin.
- **Empty map (no pins):** Shows the map image with a subtle "Click anywhere to add your first pin" prompt.
- **No maps yet:** Atlas view shows "Upload your first map" prompt with drag-and-drop area.
- **Very large images:** Show loading progress bar. Generate lower-resolution preview for immediate display while full image loads.
- **Pin overlap:** When multiple pins are very close at current zoom, cluster them into a numbered badge. Clicking expands to show individual pins.

---

## Performance

- **Image rendering:** Use the browser/webview's native image scaling. For maps > 8K, pre-generate zoom tiles during import.
- **Pin rendering:** Canvas overlay (not DOM elements) for maps with 100+ pins. DOM elements for < 50 pins (simpler interaction).
- **Pan/zoom:** RequestAnimationFrame-based rendering. 60fps target.

---

## Accessibility

- **Keyboard navigation:** Tab between pins in reading order (left-to-right, top-to-bottom). Enter to open tooltip. Escape to close.
- **Screen reader:** Each pin announced as "[Label] — [Entity type] — [coordinates]". Map described via alt text.
- **Zoom controls:** +/- buttons in toolbar as alternative to scroll wheel.
