# Design Spec: Milestone 5 — Timelines (Chronicle)

**Status:** Draft
**Mockups:** `design/vaelorium-timelines.pen` (xBx3G)
**Depends on:** Milestone 1 (Wiki Engine)

---

## Overview

Timelines provide a chronological view of world history and campaign events. Users define custom calendar systems, create events linked to wiki pages, and view them across three modes: Chronicle (narrative vertical list), Gantt (visual duration bars), and Calendar (day/month grid). Multiple parallel timelines can coexist (e.g., "World History" alongside "Campaign Sessions").

---

## User Stories

- **As a GM**, I want to create a custom calendar with my own months, day counts, and eras so my timeline reflects my world's time system.
- **As a GM**, I want to create events on a timeline, each linked to a wiki page, with a date, duration, and description.
- **As a GM**, I want to view events as a vertical narrative timeline (Chronicle view) grouped by era.
- **As a GM**, I want to view events as horizontal bars on a Gantt chart to see overlapping durations.
- **As a GM**, I want a calendar grid view for day-by-day session tracking.
- **As a GM**, I want to maintain multiple timelines (World History, Faction History, Character Arc) and switch between them.

---

## Data Model

### SQLite Schema

```sql
-- Calendar systems
CREATE TABLE calendars (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,                -- "Aethermoor Calendar"
    year_zero_label TEXT DEFAULT 'Year 0',
    months TEXT NOT NULL,              -- JSON array: [{"name":"Frostmere","days":30}, ...]
    days_per_week INTEGER DEFAULT 7,
    day_names TEXT,                    -- JSON array: ["Solday","Moonday",...]
    eras TEXT,                         -- JSON array: [{"name":"Age of Founding","start_year":1}, ...]
    created_at TEXT NOT NULL
);

-- Timelines
CREATE TABLE timelines (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,                        -- hex color for the timeline
    calendar_id TEXT REFERENCES calendars(id),
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Timeline events
CREATE TABLE timeline_events (
    id TEXT PRIMARY KEY,
    timeline_id TEXT NOT NULL REFERENCES timelines(id) ON DELETE CASCADE,
    page_id TEXT REFERENCES pages(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    description TEXT,
    start_year INTEGER,
    start_month INTEGER,               -- 1-based month index
    start_day INTEGER,                 -- 1-based day
    end_year INTEGER,                  -- NULL = point event (no duration)
    end_month INTEGER,
    end_day INTEGER,
    era TEXT,                          -- era label (denormalized for display)
    sort_order INTEGER DEFAULT 0,      -- for ordering within same date
    created_at TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_timeline_events_timeline ON timeline_events(timeline_id, start_year, start_month, start_day);
CREATE INDEX idx_timeline_events_page ON timeline_events(page_id);
```

---

## Key Interactions

### Chronicle View (Primary — shown in mockup)

**Layout:**
- Vertical gold timeline line on the left
- Era badges (gold pill) break the timeline into periods
- Event cards to the right of the line, connected by colored dots
- Each card: year/date label (gold), title (Playfair), description (Newsreader), entity link chips

**Behavior:**
- Scroll vertically through time
- Click event title to open linked page
- Click entity chips to navigate to those pages
- Click era badge to collapse/expand that era's events

### Gantt View

**Layout:**
- Horizontal time axis at top (years, with month subdivisions when zoomed)
- Each event as a horizontal bar, colored by timeline
- Bar length represents duration (start → end date)
- Point events shown as diamonds
- Rows grouped by timeline or entity type (toggle)

**Behavior:**
- Pan horizontally to scroll through time
- Zoom to change time granularity (decades → years → months → days)
- Click bar to see event details tooltip
- Drag bar edges to adjust dates (editing)

### Calendar View

**Layout:**
- Month grid using the custom calendar's month/day definitions
- Events shown as colored dots or small bars on their dates
- Month navigation arrows
- Sidebar shows events for the selected day

**Behavior:**
- Click a day to see its events in detail
- Click + on a day to create a new event on that date
- Month/year picker for quick navigation

### Timeline Sidebar

- List of timelines with color dots
- Click to switch active timeline
- Multiple timelines can be overlaid in Gantt/Calendar views
- "+ New Timeline" button

### Creating Events

1. Click "Add Event" in toolbar
2. Form: Title, linked page (search), start date, end date (optional), description
3. Date picker uses the custom calendar system
4. Event appears immediately on the timeline

---

## Edge Cases

- **No calendar defined:** Default to a simple year-only system. Prompt user to set up a full calendar.
- **Events without dates:** Allowed — shown in a separate "Undated" section at the bottom of Chronicle view.
- **Overlapping eras:** Eras can overlap in the data model but display sequentially in Chronicle view. Gantt view shows the overlap visually.
- **Deleted linked page:** Event remains; page link shows as "Page deleted" with option to relink.
- **Very long timelines:** Virtual scrolling in Chronicle view. Gantt view renders only the visible time range.

---

## Accessibility

- **Chronicle view:** Uses semantic list markup. Events are navigable with Tab. Era badges are collapsible with Enter/Space.
- **Gantt view:** Provides a companion table view (accessible alternative) listing events in tabular format.
- **Calendar view:** Standard grid navigation with arrow keys for days, Page Up/Down for months.
- **Date input:** Custom calendar date picker fully keyboard navigable.
