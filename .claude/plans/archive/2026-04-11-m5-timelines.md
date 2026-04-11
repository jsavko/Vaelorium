---
status: completed
---
# Milestone 5: Timelines (Chronicle)

**Date:** 2026-04-11

---

## Goal

Chronological event tracking linked to wiki pages. Events placed on a timeline with custom calendar support. Chronicle view shows events as a narrative list. Gantt view shows events as bars on a horizontal timeline.

## Chosen Approach

Start with a simple timeline model: events have a date (as a sortable string), optional end date, and link to a page. Two views: Chronicle (vertical narrative list) and Gantt (horizontal bars). Custom calendars deferred — use simple year/month/day for now.

## Tasks

### Data Layer

- [x] **1.** Create `007_timelines.sql` migration — timelines and timeline_events tables. Register migration.

- [x] **2.** Create `src-tauri/src/commands/timelines.rs` — Rust commands: create_timeline, list_timelines, create_event, update_event, delete_event, get_timeline_events.

- [x] **3.** Register commands in lib.rs + mod.rs.

- [x] **4.** Add timeline commands to bridge.ts mock.

- [x] **5.** Create `src/lib/api/timelines.ts` — TypeScript API wrappers.

- [x] **6.** Create `src/lib/stores/timelineStore.ts`.

### UI

- [x] **7.** Build `ChronicleView.svelte` — vertical scrolling list of events sorted by date, with date headers, event cards showing title + linked page + description.

- [x] **8.** Build `TimelineHeader.svelte` — header with timeline selector, view toggle (Chronicle/Gantt), and "+ Add Event" button.

- [x] **9.** Build add event form — date input, title, description, link to page, optional end date.

- [x] **10.** Wire "Chronicle" sidebar nav to show ChronicleView.

### Testing

- [x] **11.** Write E2E tests.

- [x] **12.** Verify all existing tests pass.

## Notes

- Timeline dates stored as sortable strings (e.g., "1420-03-15" for in-world dates, or "Era 3, Year 1420, Month 3, Day 15").
- Custom calendar systems (with custom months, eras, moons) deferred to polish.
- Gantt view deferred to polish — Chronicle view is the core deliverable.
- Each Tome can have multiple timelines (e.g., "Main Campaign", "Backstory", "World History").
