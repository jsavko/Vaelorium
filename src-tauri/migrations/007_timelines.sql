-- Milestone 5: Timelines (Chronicle)

CREATE TABLE timelines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY,
    timeline_id TEXT NOT NULL REFERENCES timelines(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    date TEXT NOT NULL,
    end_date TEXT,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    color TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_timeline_events_timeline ON timeline_events(timeline_id, date);
CREATE INDEX idx_timeline_events_page ON timeline_events(page_id);
