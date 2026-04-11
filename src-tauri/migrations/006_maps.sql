-- Milestone 4: Interactive Maps (Atlas)

CREATE TABLE maps (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    image_id TEXT REFERENCES images(id),
    parent_map_id TEXT REFERENCES maps(id) ON DELETE SET NULL,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE map_pins (
    id TEXT PRIMARY KEY,
    map_id TEXT NOT NULL REFERENCES maps(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    label TEXT,
    x REAL NOT NULL,
    y REAL NOT NULL,
    icon TEXT DEFAULT 'map-pin',
    color TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_maps_parent ON maps(parent_map_id);
CREATE INDEX idx_map_pins_map ON map_pins(map_id);
CREATE INDEX idx_map_pins_page ON map_pins(page_id);
