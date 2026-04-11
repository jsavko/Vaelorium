# Design Spec: Milestone 2.5 — Tomes (Multi-Project Support)

**Status:** Draft
**Mockups:** `design/vaelorium-tomes.pen` (KJ6aC: Tome Picker, 4e6dC: Create New Tome)
**Depends on:** Milestone 2 (Entity Type System)

---

## Overview

A Tome is a self-contained `.vaelorium` file — a SQLite database containing all pages, entity types, field values, tags, and settings for one world/campaign. The app opens one Tome at a time. Users can create new Tomes, open existing ones via file picker or drag-and-drop, and see a list of recently opened Tomes on a home screen.

The Tome Picker is shown on app launch when no Tome is loaded, or when the user closes the current Tome.

---

## User Stories

- **As a GM**, I want each campaign to be its own file so I can back it up, share it, or move it between devices.
- **As a GM**, I want to see my recent Tomes on launch so I can quickly resume where I left off.
- **As a GM**, I want to create a new Tome with a name and optional description.
- **As a GM**, I want to open an existing `.vaelorium` file from my filesystem.
- **As a GM**, I want to close the current Tome and return to the Tome Picker to switch campaigns.
- **As a GM**, I want the current Tome's name visible in the sidebar so I know which world I'm in.

---

## Data Model

### Tome File Format

Each `.vaelorium` file is a SQLite database containing:
- All tables from migrations 001 (wiki engine) and 002 (entity types)
- A `tome_metadata` table for Tome-specific settings

```sql
CREATE TABLE tome_metadata (
    key TEXT PRIMARY KEY,
    value TEXT
);

-- Seeded on creation:
-- key: 'name',        value: 'The Bolagian Chronicle'
-- key: 'description', value: 'A sprawling campaign...'
-- key: 'cover_image', value: null (or base64/path)
-- key: 'created_at',  value: '2026-04-11T00:00:00Z'
```

### App-Level Settings (Outside Tomes)

Recent Tomes list is stored in the app data directory, not inside any Tome:

```
~/.local/share/com.vaelorium.app/app_state.json
```

```json
{
  "recentTomes": [
    {
      "path": "/home/james/Documents/campaigns/bolagian-chronicle.vaelorium",
      "name": "The Bolagian Chronicle",
      "description": "A sprawling campaign...",
      "lastOpened": "2026-04-11T10:30:00Z"
    }
  ]
}
```

---

## Key Interactions

### Tome Picker Home Screen (KJ6aC)

**Trigger:** App launch with no Tome loaded, or "Close Tome" action.

**Layout:**
- Centered vertical layout
- Vaelorium logo + "The Arcane Library" subtitle
- "RECENT TOMES" section with card grid (3 columns)
- Each card: gradient cover area with icon, Tome name (Playfair 16px), description snippet, last opened date
- "Create New Tome" dashed card with + icon
- Footer: "Open Tome File" button + "or drag a .vaelorium file here" hint

**Behavior:**
- Clicking a recent Tome card opens that Tome
- Clicking "Create New Tome" opens the Create modal
- Clicking "Open Tome File" opens a native file picker filtered to `.vaelorium`
- Drag-and-drop a `.vaelorium` file onto the window opens it
- If a Tome file is missing/corrupt, show error state on the card

### Create New Tome Modal (4e6dC)

**Trigger:** Clicking "Create New Tome" card on home screen.

**Layout:**
- Modal overlay with centered card (520px wide)
- Title: "Create New Tome" (Playfair 24px)
- Description text explaining what a Tome is
- Form fields: Name (required), Description (optional textarea), Cover Image (optional drag/upload)
- Footer: Cancel + "Create Tome" button (gold, disabled until name entered)

**Behavior:**
- "Create Tome" opens a native save-file dialog to choose where to save the `.vaelorium` file
- Creates the SQLite database at that path, runs all migrations, seeds `tome_metadata`
- Opens the new Tome immediately
- Adds to recent Tomes list

### Sidebar Integration

- When a Tome is open, the sidebar header shows the Tome name instead of static "Vaelorium"
- A "Close Tome" option in the settings menu or ⋯ menu returns to the Tome Picker
- The Tome name is clickable to edit it

---

## Rust Backend Changes

### Database Connection Swapping

The current `DbPool` is initialized once at startup. For Tomes:

1. App starts with NO database connection
2. Tome Picker is shown (pure frontend, no DB needed)
3. When a Tome is opened:
   - Create/connect to the SQLite file at the given path
   - Run all pending migrations
   - Replace the managed `DbPool` state
4. When a Tome is closed:
   - Drop the current connection pool
   - Return to Tome Picker state

### New Rust Commands

- `get_app_state() -> AppState` — read recent Tomes list from app_state.json
- `save_app_state(state: AppState)` — write recent Tomes list
- `create_tome(path, name, description?) -> TomeInfo` — create new .vaelorium file, run migrations, seed metadata
- `open_tome(path) -> TomeInfo` — open existing .vaelorium file, run migrations, return metadata
- `close_tome()` — close current connection, return to no-DB state
- `get_tome_metadata() -> TomeMetadata` — read current Tome's metadata
- `update_tome_metadata(key, value)` — update Tome metadata

### File Extension

`.vaelorium` — registered as a file type association on install.

---

## Edge Cases

- **Missing file:** If a recent Tome's file is missing, show it grayed out with "File not found" and a remove button.
- **Corrupt file:** If a Tome can't be opened (bad SQLite), show an error toast and stay on Tome Picker.
- **Migration forward-compat:** Newer app versions run new migrations on older Tomes automatically. Older apps can't open Tomes with newer migrations (show version mismatch error).
- **File locking:** SQLite WAL mode handles concurrent reads. Only one app instance should have a Tome open at a time.
- **No Tome open:** All page/entity commands should return errors if no Tome is open. The frontend guards against this by showing the Tome Picker.

---

## Empty States

- **No recent Tomes:** Show "No recent Tomes" text with prominent "Create New Tome" and "Open Tome File" buttons.
- **First launch:** Same as no recent Tomes — clean slate with welcoming copy.
