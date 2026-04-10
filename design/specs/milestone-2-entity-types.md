# Design Spec: Milestone 2 — Entity Type System

**Status:** Draft
**Mockups:** `design/vaelorium-entity-types.pen` (gtNUI, XA9sK, jFaAs)
**Depends on:** Milestone 1 (Wiki Engine)

---

## Overview

The Entity Type System adds optional structured fields to wiki pages. A page can have a type (Character, Location, Quest, etc.) which attaches a set of defined fields alongside its free-form wiki content. Users can also create custom types with custom field schemas. Entity types enable filtered list views, cross-entity queries, and structured data display in the details panel.

---

## User Stories

### Applying Types
- **As a GM**, I want to choose an entity type when creating a new page so it gets the right structured fields.
- **As a GM**, I want to change or remove a page's entity type after creation.
- **As a GM**, I want to apply a type to an existing untyped page without losing its content.

### Structured Fields
- **As a GM**, I want to fill in structured fields (Race, Class, HP, Status, etc.) alongside my wiki content.
- **As a GM**, I want some fields to link to other pages (e.g., Location field links to a Location page).
- **As a GM**, I want fields to appear in the details panel on the right side of the editor.

### Custom Types
- **As a GM**, I want to create my own entity types (e.g., "Spell", "Vehicle", "Language") with custom fields.
- **As a GM**, I want to define field types: text, number, select, multi-select, long text, boolean, page reference.
- **As a GM**, I want to reorder fields by dragging them.
- **As a GM**, I want to see a live preview of my custom type as I define it.
- **As a GM**, I want to export/import type definitions as JSON files to share with other campaigns.

### Filtered Views
- **As a GM**, I want to browse all entities of a specific type in a grid or list view.
- **As a GM**, I want to filter entities by field values (e.g., all Characters with Status = "Alive").
- **As a GM**, I want to sort entities by name, date modified, or any structured field.
- **As a GM**, I want to switch between card grid and list table views.

---

## Data Model

### SQLite Schema

```sql
-- Entity type definitions
CREATE TABLE entity_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,                -- "Character", "Location", "Spell", etc.
    icon TEXT,                         -- lucide icon name
    color TEXT,                        -- hex color for badges
    is_builtin BOOLEAN DEFAULT FALSE,  -- TRUE for the 8 built-in types
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Field definitions for entity types
CREATE TABLE entity_type_fields (
    id TEXT PRIMARY KEY,
    entity_type_id TEXT NOT NULL REFERENCES entity_types(id) ON DELETE CASCADE,
    name TEXT NOT NULL,                -- "Race", "HP", "School", etc.
    field_type TEXT NOT NULL,          -- 'text', 'number', 'select', 'multi_select',
                                      -- 'long_text', 'boolean', 'page_reference'
    sort_order INTEGER DEFAULT 0,
    is_required BOOLEAN DEFAULT FALSE,
    default_value TEXT,                -- JSON-encoded default
    options TEXT,                      -- JSON array for select/multi_select choices
    reference_type_id TEXT,            -- for page_reference: limit to this entity type
    created_at TEXT NOT NULL
);

-- Field values for specific pages
CREATE TABLE entity_field_values (
    id TEXT PRIMARY KEY,
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    field_id TEXT NOT NULL REFERENCES entity_type_fields(id) ON DELETE CASCADE,
    value TEXT,                        -- JSON-encoded value
    UNIQUE(page_id, field_id)
);

-- Indexes
CREATE INDEX idx_entity_type_fields_type ON entity_type_fields(entity_type_id, sort_order);
CREATE INDEX idx_entity_field_values_page ON entity_field_values(page_id);
CREATE INDEX idx_entity_field_values_field ON entity_field_values(field_id);
```

### Built-in Entity Types

These ship with the app and cannot be deleted (but can be customized):

| Type | Icon | Color | Default Fields |
|------|------|-------|----------------|
| Character | shield | #B85C5C | Race (text), Class (text), Alignment (select), Status (select: Alive/Dead/Missing/Unknown), HP (number), Location (page_reference→Location), Organisation (page_reference→Organisation) |
| Location | compass | #4A8C6A | Type (select: City/Town/Village/Fortress/Temple/Wilderness/Other), Region (page_reference→Location), Population (number), Climate (text) |
| Quest | scroll | #5C7AB8 | Status (select: Active/Completed/Failed/Abandoned), Priority (select: Low/Medium/High/Critical), Giver (page_reference→Character), Reward (text) |
| Organisation | users | #8B5CB8 | Type (select: Guild/Order/Government/Criminal/Religious/Other), Leader (page_reference→Character), Headquarters (page_reference→Location), Members (number) |
| Item | gem | #B8955C | Type (select: Weapon/Armor/Potion/Scroll/Wondrous/Other), Rarity (select: Common/Uncommon/Rare/Very Rare/Legendary/Artifact), Value (text), Owner (page_reference→Character) |
| Creature | bug | #5CB8A8 | Type (select: Beast/Monstrosity/Undead/Fiend/Celestial/Dragon/Other), Challenge Rating (text), Habitat (page_reference→Location), Alignment (select) |
| Event | sparkles | #B85C8B | Date (text), Duration (text), Location (page_reference→Location), Significance (select: Minor/Major/World-changing) |
| Journal | notebook-pen | #7A8C5C | Session Number (number), Date Played (text), DM (page_reference→Character), Location (page_reference→Location) |

### Field Types

| Field Type | Storage | Input Widget | Display |
|------------|---------|--------------|---------|
| text | `"string value"` | Single-line text input | Plain text |
| number | `42` | Number input with optional step | Plain number |
| select | `"option_value"` | Dropdown select | Colored badge |
| multi_select | `["opt1","opt2"]` | Multi-select with tags | Tag row |
| long_text | `"multiline..."` | Textarea (not TipTap) | Paragraph text |
| boolean | `true` | Toggle switch | Checked/unchecked indicator |
| page_reference | `"page_uuid"` | Page search dropdown (filtered by type) | Entity dot + gold link |

---

## Key Interactions

### New Page Modal (Type Selector)

**Trigger:** "New Page" button or Cmd/Ctrl+N.

**Layout:** Modal with:
1. "Blank Page" option at top (full width, no type)
2. "ENTITY TYPES" label
3. 3-column grid of type cards (icon, name, description)
4. Selected type highlighted with gold border
5. Bottom footer: title input, parent folder picker, "Create" button

**Behavior:**
- Clicking a type card selects it (gold border). Clicking again deselects.
- "Custom Type" card (+ icon) opens the Custom Type Builder.
- Title input auto-focuses after type selection.
- Parent folder defaults to the currently selected sidebar location.
- "Create" is disabled until title is entered.

### Entity List View

**Trigger:** Clicking an entity type tab or selecting "All Characters" from sidebar.

**Layout:**
- Header: entity icon + type name + count badge + "New [Type]" button
- Filter bar: search input, sort dropdown, grid/list toggle
- Type tabs: All, Characters, Locations, Quests, Organisations, Items, More...
- Content: card grid (default) or list table

**Card Grid:**
- Cards show: featured image (or placeholder), title (Playfair), key field values as metadata line, status badge
- 3 columns on desktop, 2 on narrow, 1 on mobile
- Click card to open page

**List Table:**
- Columns: Name, key fields (type-specific), Status, Last edited
- Sortable by clicking column headers
- Row click opens page

**Filtering:**
- Type tabs filter by entity type
- Search input filters by name (instant, client-side for current type)
- Advanced filter (future): filter by specific field values

### Custom Type Builder

**Trigger:** "Custom Type" card in new page modal, or Settings → Entity Types.

**Layout:** Split-pane modal:
- Left (380px): Type name input, icon/color picker, draggable field list with "Add Field" button
- Right: Live preview showing how the type looks with sample data

**Field List:**
- Each field row: drag handle, field name, field type badge, delete X
- Selected field highlighted with gold border
- Clicking a field opens inline editing (name, type dropdown, options)
- Drag to reorder

**Field Type Selection:**
- Dropdown with all field types and brief descriptions
- For select/multi_select: shows options editor (add/remove/reorder choices)
- For page_reference: shows entity type filter dropdown

**Live Preview:**
- Updates in real-time as fields are added/edited
- Shows sample/placeholder values for each field type
- Entity badge uses selected icon and color

**Save:** Creates the entity type and its field definitions. Available immediately in the type selector.

### Details Panel (Structured Fields)

**Location:** Right panel when "Details" is toggled on (see Milestone 1 metadata panel mockup).

**Display:**
- "CHARACTER FIELDS" (or type-appropriate label) section header
- Each field: label (foreground-tertiary) above value (foreground-primary)
- Page reference fields: entity dot + gold link text
- Select fields: plain text value
- Status fields: colored badge (Alive=green, Dead=red, Missing=amber, Unknown=gray)

**Editing:**
- Click any field value to edit inline
- Text fields: input replaces display
- Select fields: dropdown appears
- Page reference: opens mini page search
- Changes saved immediately (debounced 500ms)

---

## Edge Cases

- **Changing entity type:** Preserves wiki content. Old field values are soft-deleted (kept in DB but hidden). If user switches back, values reappear.
- **Removing entity type:** Returns page to untyped. Field values preserved in DB for potential re-application.
- **Deleting a custom type:** Confirmation dialog. All pages of that type revert to untyped. Field values deleted.
- **Field type mismatch:** If a select field's options change and existing pages have now-invalid values, the value displays with a warning icon and "Unknown value" tooltip.
- **Empty entity list:** Shows "No [type] yet" with illustration and "Create your first [type]" button.

---

## Performance

- **Entity list queries:** Index on `pages.entity_type_id` + `pages.updated_at` for fast filtered queries.
- **Field value queries:** Index on `entity_field_values.page_id` for loading a page's fields. Index on `field_id` for cross-entity filtering.
- **Card grid images:** Lazy-load featured images. Show placeholder until loaded.
- **Custom type builder:** Preview updates are local state only — no DB writes until "Save Type."

---

## Accessibility

- **Type selector grid:** Keyboard navigable with arrow keys. Selected type announced via `aria-selected`.
- **Field list:** Drag-and-drop has keyboard alternative (select field, use Ctrl+Up/Down to move).
- **Inline field editing:** Focus management — clicking a field value focuses the input; Escape returns focus to the value display.
- **Entity list:** Card grid is a `role="grid"` with arrow key navigation. Tab moves between cards.
