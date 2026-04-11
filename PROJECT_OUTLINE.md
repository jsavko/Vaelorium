# Project Outline: Self-Hosted Offline-First Worldbuilding Platform

> **Name:** Vaelorium
> **Created:** 2026-04-10
> **Status:** Active Development — Milestones 0-2 complete, M2.5 next

---

## Vision

A free, open-source, offline-first worldbuilding and RPG campaign management tool that runs as a native desktop app on Windows, Mac, and Linux. Users own their data completely. An optional sync server enables multi-device and multi-user collaboration — self-hostable for free, or available as a managed SaaS offering.

This is the tool LegendKeeper was supposed to be: local-first, self-hosted, with real offline support that doesn't depend on browser cache.

## Core Principles

1. **Offline-first, always.** The app works fully without any network. Sync is additive, not required.
2. **Your data is yours.** All data stored locally in open formats. No vendor lock-in. Export at any time.
3. **Design before code.** Every feature gets mockups and a design spec approved before implementation begins.
4. **Simple until proven otherwise.** No speculative abstractions. Build what's needed, extend when real requirements emerge.
5. **Two products, one codebase.** The sync server is self-hostable and SaaS-deployable from the same codebase.

---

## Branding & Visual Direction

**Name:** Vaelorium
**Domain:** vaelorium.com
**Theme:** Arcane library / ancient athenaeum

The Vaelorium is a vast, timeless library — a place where all the knowledge of your world is catalogued, preserved, and interconnected. Think the Great Library of Alexandria meets a wizard's private study.

**Visual pillars:**
- **Warm and scholarly** — Aged parchment tones, warm amber lighting, deep wood textures. Not cold/techy.
- **Arcane but approachable** — Subtle magical flourishes (faint glowing runes, constellation motifs) without going full fantasy-kitsch. Elegant, not gaudy.
- **Rich materials** — Leather-bound spines, gilt edges, brass fixtures, ink stains, wax seals. Tactile, physical-feeling despite being digital.
- **Depth and layering** — Subtle shadows, card elevation, translucent overlays that feel like parchment layers stacked on a desk.

**Color palette direction:**
- **Primary:** Deep library tones — rich walnut browns, aged parchment cream, warm charcoal
- **Accent:** Arcane gold/amber (for highlights, links, active states), midnight blue or deep teal (for secondary accents)
- **Semantic:** Muted jewel tones for entity type badges (ruby for Characters, emerald for Locations, sapphire for Quests, amethyst for Organisations, etc.)
- **Dark mode:** Midnight blue-black with warm gold accents — a library at night by candlelight

**Typography direction:**
- **Headings:** A serif with character — something that feels like it belongs on a book spine or chapter heading
- **Body:** A highly readable serif or humanist sans — long-form writing comfort is critical for a wiki tool
- **UI/labels:** Clean sans-serif for buttons, navigation, metadata — contrasts with the editorial feel of content

**Iconography:**
- Line-art style with subtle weight variation, as if drawn with a quill
- Entity type icons that evoke their meaning: a shield for Characters, a compass rose for Locations, a scroll for Quests, etc.

**Voice & tone:**
- The app speaks like a knowledgeable but warm librarian — helpful, precise, never condescending
- Feature names can lean into the metaphor where natural (e.g., "The Atlas" for maps, "The Chronicle" for timelines, "The Scriptorium" for the editor)
- Avoid over-theming to the point of confusion — clarity always wins over cleverness

---

## Technology Stack

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Desktop shell | Tauri 2 | Small binaries, native performance, cross-platform, Rust backend |
| Frontend framework | Svelte 5 | Lightweight, fast, best DX, strong TipTap/Tauri ecosystem |
| Rich text editor | TipTap (ProseMirror) | Headless, framework-agnostic, native Yjs/CRDT support |
| Local database | SQLite | Embedded, zero-config, battle-tested, FTS5 for search |
| CRDT / sync (documents) | Yjs | Industry-standard CRDT for rich text, proven TipTap integration |
| CRDT / sync (structured data) | Custom change log over SQLite | Row-level sync for entity metadata, relations, permissions |
| Sync server | Rust (Axum or Actix) | Standalone service, WebSocket + REST API, containerized |
| Image/file storage | Local filesystem (desktop), S3-compatible (SaaS) | Simple for self-hosted, scalable for managed |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Desktop App (Tauri 2)           │
│  ┌───────────────────────────────────────────┐   │
│  │           Frontend (Svelte/Solid)         │   │
│  │  ┌─────────┐ ┌──────┐ ┌──────┐ ┌──────┐  │   │
│  │  │  Wiki   │ │ Maps │ │ Time │ │Board │  │   │
│  │  │ Editor  │ │      │ │ lines│ │      │  │   │
│  │  │(TipTap) │ │      │ │      │ │      │  │   │
│  │  └─────────┘ └──────┘ └──────┘ └──────┘  │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │  Entity Type System (JSON schemas)  │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  └───────────────────────────────────────────┘   │
│  ┌───────────────────────────────────────────┐   │
│  │           Rust Backend (Tauri)             │   │
│  │  ┌──────────┐ ┌─────┐ ┌───────────────┐  │   │
│  │  │  SQLite  │ │ Yjs │ │  File Storage  │  │   │
│  │  │  + FTS5  │ │Store│ │  (local disk)  │  │   │
│  │  └──────────┘ └─────┘ └───────────────┘  │   │
│  │  ┌─────────────────────────────────────┐  │   │
│  │  │         Sync Client (optional)      │  │   │
│  │  └─────────────────────────────────────┘  │   │
│  └───────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
                        │ (optional)
                        ▼
┌─────────────────────────────────────────────────┐
│              Sync Server (Rust)                  │
│  ┌──────────┐ ┌──────────┐ ┌─────────────────┐  │
│  │ Auth &   │ │ Yjs Sync │ │ Structured Data │  │
│  │ Tenancy  │ │ Relay    │ │ Sync (REST)     │  │
│  └──────────┘ └──────────┘ └─────────────────┘  │
│  ┌──────────┐ ┌──────────────────────────────┐  │
│  │ Storage  │ │ Admin / Billing (SaaS only)  │  │
│  └──────────┘ └──────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

---

## Key Components

### 1. Wiki Engine
The foundational layer. Everything is a page.

- Nested page hierarchy (infinite depth, drag-and-drop reorder)
- Rich text editing via TipTap (headings, lists, tables, embeds, callouts)
- Cross-linking between pages via @-mentions and `[[wiki links]]`
- Page metadata: title, icon/emoji, featured image, tags, created/modified dates
- Version history (stored as Yjs snapshots)
- Full-text search via SQLite FTS5

### 2. Entity Type System
Optional structure layered onto wiki pages.

- A page can optionally have a **type** (Character, Location, Organisation, Quest, Item, Creature, Event, Journal, etc.)
- Each type defines a **JSON schema** of structured fields displayed alongside the wiki content
- Built-in types ship with sensible defaults (e.g., Character has fields for Race, Class, HP, Alignment, etc.)
- Users can create **custom types** with custom fields
- Structured fields are queryable — "show me all Characters in Location X with HP > 50"
- Entity types are templates: applying one to a page adds the fields, removing it strips them
- Templates are shareable as JSON files

### 3. Relations & Connections
How entities link to each other beyond simple wiki links.

- Typed, bidirectional relations between any two pages (e.g., "Alice *is the leader of* The Guild")
- Relation types are user-definable
- Visual relation explorer (graph view of connected entities)
- Relations are first-class data — queryable, filterable, displayable on entity pages

### 4. Interactive Maps
Visual spatial layer for worlds, regions, dungeons.

- Upload map images (support for large images, tiled rendering)
- Place pins on maps, each linked to a wiki page
- Customizable pin icons, colors, labels
- Nested map hierarchy (world map → region → city → building → room)
- Click a pin to navigate to linked page or child map
- Tag-based pin filtering and visibility (GM-only pins, per-player visibility)
- Layers support (overlay different information on the same map)

### 5. Timelines
Chronological view of world history and events.

- Custom calendar systems (define months, days per month, week length, year zero, eras)
- Three views: Chronicle (narrative list), Gantt (visual bars), Calendar (day grid)
- Events link to wiki pages
- Events can span durations (wars, reigns, seasons)
- Multiple parallel timelines (political history vs. religious history vs. character arcs)
- Zoom from cosmic scale (eons) to granular (hours)

### 6. Boards (Whiteboards)
Visual brainstorming and diagramming.

- Freeform canvas with pan/zoom
- Card nodes that link to wiki pages
- Connectors between cards (directional, labeled)
- Use cases: faction relationship diagrams, quest flowcharts, family trees, DM screens, session prep
- Freehand drawing, text notes, image embeds

### 7. Permissions & Roles
Control what different users see and edit.

- Roles: **Owner/GM** (full access), **Co-GM** (full access, no delete campaign), **Player** (sees only what's shared)
- Per-page visibility: public (all users), GM-only, or specific players
- Secret sections within pages (GM eyes only, revealed per-player)
- Permissions are advisory locally, enforced by sync filtering (sync server only sends data the user's role permits)

### 8. Sync Server
Standalone service enabling multi-device and multi-user collaboration.

- **Yjs relay:** WebSocket server that relays Yjs document updates between clients
- **Structured data sync:** REST API for pushing/pulling SQLite change logs (entity metadata, relations, permissions)
- **File sync:** Upload/download images and map files
- **Auth:** Token-based authentication (JWT or similar)
- **Multi-tenant:** Namespace isolation per campaign/workspace
- **Self-hosted mode:** Single Docker container, minimal config, SQLite or Postgres backend
- **SaaS mode:** Multi-tenant, managed Postgres, S3 storage, usage metering, billing integration, TLS termination

### 9. Import & Export
Data portability is non-negotiable.

- **Export:** Full project as JSON, HTML static site, or Markdown archive
- **Import:** From LegendKeeper (JSON export), Kanka (API or JSON), World Anvil (if possible), Markdown folders
- Export is always free, never paywalled

---

## Milestones

Each milestone follows the same process:
1. **Design** — Mockups and interaction design for all features in the milestone
2. **Spec** — Written design spec with data models, API contracts, and edge cases
3. **Review** — Review and approve designs before any code is written
4. **Build** — Implement the milestone
5. **Test** — Manual and automated testing
6. **Ship** — Release as a versioned build

---

### Milestone 0: Foundation & Tooling
> **Goal:** Project scaffolding, dev environment, CI/CD, design system

**Deliverables:**
- [x] Project name and branding direction decided — **Vaelorium**, library/arcane athenaeum theme
- [x] Tauri 2 project scaffolded with Svelte 5
- [x] Rust backend skeleton with SQLite integration (sqlx, 21 commands)
- [x] Development environment documented (README with setup instructions)
- [ ] CI pipeline (build + test on all 3 platforms) — **DEFERRED to Milestone 10**
- [x] Design system / component library foundations (Tailwind CSS 4 + design tokens in app.css)
- [x] Design tool and workflow established for mockups (Pencil MCP)

---

### Milestone 1: Wiki Engine
> **Goal:** The core writing and organizing experience — pages, nesting, rich text, linking, search

**Design phase deliverables:**
- [x] Mockups: Page editor view (editing state, reading state)
- [x] Mockups: Page tree / sidebar navigation
- [x] Mockups: Cross-linking UI (@-mentions, wiki links, backlinks panel)
- [x] Mockups: Search interface and results
- [x] Mockups: Page metadata panel (tags, icon, featured image)
- [x] Design spec: TipTap editor configuration (which extensions, custom blocks)
- [x] Design spec: Page data model (SQLite schema + Yjs document structure)
- [x] Design spec: FTS5 search indexing strategy

**Build phase deliverables:**
- [x] TipTap editor integrated with Yjs for CRDT storage
- [x] Page CRUD operations (create, read, update, delete)
- [x] Nested page tree with drag-and-drop reordering
- [x] Cross-linking with @-mentions and [[wiki links]]
- [x] Backlinks panel (show all pages that link to this one)
- [x] Full-text search
- [x] Page metadata (tags, icon, featured image)
- [x] Version history (Yjs snapshots, preview, restore) — diff view **DEFERRED to Milestone 10**
- [x] Keyboard shortcuts for power users (customizable in Settings)

**Additional features delivered (not in original spec):**
- [x] Slash commands (/ trigger with 9 block types)
- [x] Settings page (keybinds + appearance/themes)
- [x] Reading view toggle
- [x] Rename propagates across all page mentions
- [x] Context menu (right-click: new child, delete)
- [x] Toast notifications, confirmation dialogs
- [x] Browser mock bridge for testing without Tauri
- [x] 80 tests (30 unit + 50 E2E)

---

### Milestone 2: Entity Type System ✅
> **Goal:** Optional structured fields on wiki pages — the hybrid model

**Design phase deliverables:**
- [x] Mockups: Entity type selector when creating/editing a page
- [x] Mockups: Structured fields panel alongside wiki content
- [x] Mockups: Custom type builder UI
- [x] Mockups: Filtered entity list views ("all Characters", "all Locations in Region X")
- [x] Design spec: JSON schema format for entity type definitions
- [x] Design spec: Built-in type definitions (Character, Location, Organisation, Quest, Item, Creature, Event, Journal)
- [x] Design spec: How structured fields are stored in SQLite and queried

**Build phase deliverables:**
- [x] Entity type system with SQLite schema (entity_types, entity_type_fields, entity_field_values)
- [x] 8 built-in entity types with 35 default field definitions (seeded in migration)
- [x] Custom entity type creation via Custom Type Builder modal (name, color, icon, fields)
- [x] Structured fields rendered alongside wiki content in details panel (7 field types: text, number, select, multi_select, long_text, boolean, page_reference)
- [x] Filtered list views per entity type (card grid with search, sidebar type navigation)
- [x] Cross-entity queries ("all Characters at Location X") via query_pages_by_field command
- [x] Page embed/transclusion via /embed slash command (read-only inline content)
- [ ] Template sharing (export/import type definitions as JSON) — deferred to M10

**Additional features delivered:**
- New Page modal with entity type picker (replaces inline creation)
- Entity type badges with colors in sidebar, editor header, search, mentions, backlinks
- Change/remove page entity type via details panel dropdown
- Debounced field value auto-save
- Recursive embed protection
- 14 Rust backend commands for entity types, fields, and field values
- lucide-svelte icons for entity types

---

### Milestone 2.5: Tomes (Multi-Project Support)
> **Goal:** Each campaign/world is a self-contained "Tome" — a separate database file that users can create, open, and switch between

A Tome is a `.vaelorium` file containing all pages, maps, timelines, boards, relations, and settings for one world/campaign. The app opens one tome at a time. Users can create new tomes, open existing ones, and see recent tomes on a home screen.

**Design phase deliverables:**
- [ ] Mockups: Tome Picker home screen (create new, open existing, recent tomes list)
- [ ] Mockups: Tome metadata (name, description, cover image, last opened date)
- [ ] Design spec: Tome file format (SQLite database with `.vaelorium` extension)
- [ ] Design spec: How the Rust backend swaps database connections when switching tomes

**Build phase deliverables:**
- [ ] Tome Picker home screen — shown on app launch when no tome is loaded
- [ ] Create New Tome — name, optional description, creates new `.vaelorium` database file
- [ ] Open Tome — file picker to select an existing `.vaelorium` file
- [ ] Recent Tomes list — persisted across sessions (stored outside any individual tome)
- [ ] Rust backend: swap SQLite connection pool when opening a different tome
- [ ] All migrations run automatically when a tome is opened (forward-compatible)
- [ ] Current tome name displayed in sidebar header (replaces static "Vaelorium" when a tome is open)
- [ ] Close Tome — returns to Tome Picker home screen

---

### Milestone 3: Relations & Connections
> **Goal:** Typed relationships between entities with visual exploration

**Design phase deliverables:**
- [ ] Mockups: Relation creation UI (inline and dedicated)
- [ ] Mockups: Relations panel on entity pages
- [ ] Mockups: Graph/network view of entity connections
- [ ] Design spec: Relation data model (types, directionality, metadata)
- [ ] Design spec: Graph layout algorithm and interaction model

**Build phase deliverables:**
- [ ] Typed bidirectional relations between any two pages
- [ ] User-definable relation types
- [ ] Relations panel on each page showing all connections
- [ ] Graph view for visual exploration of entity networks
- [ ] Relation filtering and search

---

### Milestone 4: Interactive Maps
> **Goal:** Upload maps, place pins, nest maps, link everything to wiki pages

**Design phase deliverables:**
- [ ] Mockups: Map viewer with pins and labels
- [ ] Mockups: Pin creation and editing UI
- [ ] Mockups: Map nesting navigation (breadcrumbs, click-to-zoom)
- [ ] Mockups: Layer management panel
- [ ] Mockups: Pin filtering UI (by tag, visibility, entity type)
- [ ] Design spec: Map data model (images, pins, layers, nesting hierarchy)
- [ ] Design spec: Large image rendering strategy (tiling, progressive loading)
- [ ] Design spec: Map image storage and file management

**Build phase deliverables:**
- [ ] Map image upload and display with pan/zoom
- [ ] Pin placement, editing, and linking to wiki pages
- [ ] Customizable pin icons, colors, and labels
- [ ] Nested map hierarchy with navigation
- [ ] Layer support (toggle different overlays)
- [ ] Pin filtering by tags and visibility roles
- [ ] Large image handling (tiled rendering for maps > 4K)

---

### Milestone 5: Timelines
> **Goal:** Chronological views with custom calendar systems

**Design phase deliverables:**
- [ ] Mockups: Chronicle view (narrative event list)
- [ ] Mockups: Gantt view (visual timeline bars)
- [ ] Mockups: Calendar view (day/week/month grid)
- [ ] Mockups: Custom calendar builder UI
- [ ] Mockups: Event creation and linking to wiki pages
- [ ] Design spec: Calendar system data model (months, days, eras, moons)
- [ ] Design spec: Event data model and timeline rendering approach

**Build phase deliverables:**
- [ ] Custom calendar system creation (months, week length, eras, etc.)
- [ ] Event CRUD linked to wiki pages
- [ ] Chronicle view
- [ ] Gantt view
- [ ] Calendar view
- [ ] Multi-timeline support (parallel timelines)
- [ ] Scale zooming (eons to hours)

---

### Milestone 6: Boards (Whiteboards)
> **Goal:** Visual brainstorming canvases with entity-linked cards

**Design phase deliverables:**
- [ ] Mockups: Board canvas with cards and connectors
- [ ] Mockups: Card creation (standalone vs. linked to wiki page)
- [ ] Mockups: Connector types and labels
- [ ] Mockups: Board toolbar (draw, text, image, shapes)
- [ ] Design spec: Board data model (nodes, edges, positions, styles)
- [ ] Design spec: Canvas rendering approach (SVG, Canvas API, or library)

**Build phase deliverables:**
- [ ] Freeform canvas with pan/zoom
- [ ] Card nodes (standalone and wiki-linked)
- [ ] Directional labeled connectors between cards
- [ ] Drag-and-drop layout
- [ ] Freehand drawing, text annotations, image embeds
- [ ] Board templates for common patterns (family tree, faction web, quest flow)

---

### Milestone 7: Permissions & Roles
> **Goal:** Multi-user access control for collaborative campaigns

**Design phase deliverables:**
- [ ] Mockups: User/role management panel
- [ ] Mockups: Per-page visibility controls
- [ ] Mockups: Secret/GM-only section UI within pages
- [ ] Mockups: Player view vs. GM view toggle
- [ ] Design spec: Role and permission data model
- [ ] Design spec: How permissions interact with sync (filtering strategy)

**Build phase deliverables:**
- [ ] Role definitions (Owner, Co-GM, Player, custom roles)
- [ ] Per-page visibility settings
- [ ] Secret sections within pages (collapsible, role-gated)
- [ ] GM/Player view toggle for previewing what players see
- [ ] Permission-aware search (players don't see GM-only results)

---

### Milestone 8: Sync Server
> **Goal:** Standalone sync service — self-hostable and SaaS-ready

**Design phase deliverables:**
- [ ] Mockups: Sync status UI in desktop app (connected, syncing, offline, conflict)
- [ ] Mockups: Sync server configuration in app settings
- [ ] Mockups: SaaS admin dashboard (if applicable)
- [ ] Design spec: Sync protocol (Yjs relay + structured data REST API)
- [ ] Design spec: Auth model (tokens, sessions, device registration)
- [ ] Design spec: Multi-tenant architecture for SaaS mode
- [ ] Design spec: Conflict resolution strategy for structured data
- [ ] Design spec: File sync protocol for images and maps
- [ ] Design spec: Docker deployment configuration

**Build phase deliverables:**
- [ ] Yjs WebSocket relay server
- [ ] Structured data sync REST API (change log push/pull)
- [ ] File/image sync
- [ ] Token-based authentication
- [ ] Device registration and management
- [ ] Sync client in desktop app (background sync, status indicators)
- [ ] Self-hosted deployment (single Docker container with docs)
- [ ] SaaS multi-tenant mode (namespace isolation, managed storage)
- [ ] Conflict UI in desktop app for manual resolution when needed

---

### Milestone 9: Import & Export
> **Goal:** Data portability — get data in and out freely

**Design phase deliverables:**
- [ ] Mockups: Export wizard (format selection, scope selection)
- [ ] Mockups: Import wizard (source selection, mapping preview)
- [ ] Design spec: Export formats (JSON archive, HTML static site, Markdown)
- [ ] Design spec: Import parsers (LegendKeeper JSON, Kanka JSON/API, Markdown folders)

**Build phase deliverables:**
- [ ] JSON export (full project, lossless round-trip)
- [ ] HTML static site export (browsable offline)
- [ ] Markdown export
- [ ] LegendKeeper JSON import
- [ ] Kanka JSON import
- [ ] Markdown folder import
- [ ] Selective export (single page, subtree, entity type)

---

### Milestone 10: Polish & Launch Prep
> **Goal:** Production-ready quality, packaging, documentation, landing page

**Deliverables:**
- [ ] Cross-platform testing and bug fixes (Windows, Mac, Linux)
- [ ] Performance profiling and optimization (large worlds: 1000+ pages, 50+ maps)
- [ ] Accessibility audit (keyboard navigation, screen readers, color contrast)
- [ ] Auto-update mechanism for desktop app
- [ ] User documentation / help system
- [ ] Landing page and project website
- [ ] SaaS billing integration (if launching managed sync)
- [ ] Packaging and distribution (GitHub Releases, platform-specific installers)
- [ ] Open source repository setup (LICENSE, CONTRIBUTING, CODE_OF_CONDUCT)
- [ ] CI pipeline (build + test on all 3 platforms) — deferred from M0
- [ ] Version diff view (side-by-side comparison between versions) — deferred from M1
- [ ] Callout block TipTap extension (info/warning/note boxes) — deferred from M1
- [ ] Share button placeholder in toolbar
- [ ] Connection count in page metadata row
- [ ] ⌘K shortcut badge in search overlay

---

## Design-First Workflow

Every feature follows this process before any code is written:

```
1. MOCKUPS        → Visual designs showing the UI and interactions
                     (tool TBD — Figma, Penpot, or .pen files)
2. DESIGN SPEC    → Written document covering:
                     - User stories and workflows
                     - Data model (schemas, relations)
                     - API contracts (if applicable)
                     - Edge cases and error states
                     - Accessibility considerations
                     - Performance requirements
3. REVIEW         → Review and iterate on designs
4. APPROVE        → Sign off before implementation begins
5. PLAN           → Create implementation plan (/plan)
6. BUILD          → Implement against approved designs
7. VALIDATE       → Compare implementation to mockups, fix deviations
```

---

## Success Criteria

The project is "v1.0" when a user can:

1. Create a new world/campaign project
2. Write and organize wiki pages in a nested hierarchy with rich text
3. Apply entity types to pages and fill in structured fields
4. Create relations between entities and explore them visually
5. Upload maps, place pins linked to pages, and nest maps
6. Build timelines with custom calendars
7. Lay out diagrams on whiteboards with entity-linked cards
8. Invite players with appropriate permission controls
9. Work entirely offline with zero degradation
10. Optionally sync between devices (self-hosted or managed)
11. Export their entire world as JSON or a browsable HTML site
12. Import from LegendKeeper or Kanka

---

## What This Document Is For

This outline is the **north star** for the project. Every plan, every design spec, every PR should trace back to a component and milestone defined here. If work doesn't map to this outline, either the outline needs updating or the work needs questioning.

Update this document as decisions are made and scope evolves. It is a living document, not a contract.
