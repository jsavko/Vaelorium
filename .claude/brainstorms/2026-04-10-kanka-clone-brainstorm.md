---
status: complete
---
# Self-Hosted Offline-First LegendKeeper Alternative — Architecture Brainstorm

**Date:** 2026-04-10

---

## User Decisions So Far

- **Data model:** Hybrid wiki + entity (free-form pages with optional structured types)
- **Sync strategy:** CRDTs
- **Maps:** Include in MVP
- **Frontend framework:** Needs research (see framework comparison below)
- **Multi-user:** Yes, with permissions (simple role system for MVP)
- **Sync topology:** TBD (hub-and-spoke recommended for MVP)

## Frontend Framework Research

| Framework | Bundle Size | Performance | Tauri Fit | Ecosystem | Notes |
|-----------|------------|-------------|-----------|-----------|-------|
| **Svelte 5** | Smallest | Excellent | Official template, compile-time focus pairs well with Tauri's lightweight philosophy | Growing, smaller than React | Best DX + performance balance |
| **SolidJS** | Smallest | Best raw benchmarks | Official template | Smallest ecosystem | Fastest but fewer libraries |
| **Vue 4** | Medium | Good | Official template | Large, mature | Smoothest team onboarding |
| **React 19** | Largest | Good | Official template | Massive | Most libraries/hiring, but heaviest |

**Recommendation:** Svelte 5 or SolidJS — both pair naturally with Tauri's "small and fast" philosophy.

**Rich text editor:** TipTap (ProseMirror-based) — battle-tested, headless (works with any framework), native Yjs/CRDT support for collaborative editing and sync.

## Multi-User Permissions in Offline-First

Permissions in a local-first app are inherently advisory, not cryptographic — the local app enforces what you can see/edit based on your role, but since data lives locally, a determined user could bypass it (same tradeoff LegendKeeper acknowledged with Hydra).

**MVP approach:** Simple role system (GM/admin sees everything, players see what's shared). Sync filters what gets sent to each user based on role. Not bulletproof, but practical.

**Future:** Encrypt per-role and only sync what each user should see for true security.

---

## Architecture Approaches

### 1. "The Pragmatist" — Tauri + SQLite + Yjs, Hub-and-Spoke Sync

**Core Concept:** Desktop-first Tauri app with SQLite for structured data, Yjs CRDTs for rich text, and an optional lightweight sync server you self-host.

**How it works:**
- Tauri 2 shell with Svelte 5 frontend, TipTap editor
- SQLite stores entity metadata, relations, tags, structured fields
- Yjs documents stored as binary blobs in SQLite for each wiki page
- Sync server is a small Rust or Node service — instances push/pull Yjs updates and SQLite change logs to it
- Sync server can be a $5 VPS, a Raspberry Pi, or skipped entirely for single-user

**Strengths:** Proven stack, each piece is battle-tested, clear separation of concerns, sync server is optional and simple. TipTap + Yjs gives you collaborative editing for free when online.

**Risks:** Two data models (SQLite for structure + Yjs for documents) means two sync mechanisms. Hub-and-spoke means no direct device-to-device sync without the server.

---

### 2. "The CRDT Purist" — Everything in Yjs/Automerge

**Core Concept:** All data — wiki pages, entity fields, relations, maps — lives in CRDT documents. No traditional database at all.

**How it works:**
- Every entity is a Yjs document with typed fields (Y.Map for structured data, Y.XmlFragment for rich text)
- Collections are Y.Arrays of document references
- Sync is peer-to-peer via y-webrtc or through a relay server via y-websocket
- Tauri app persists Yjs state to disk (IndexedDB-like file store)
- Queries built as in-memory indexes rebuilt on startup

**Strengths:** One sync mechanism for everything. True P2P possible. Conflict resolution is automatic everywhere. Elegant architecture.

**Risks:** No SQL — querying, filtering, and searching across hundreds of entities gets painful. Rebuilding indexes on startup is slow for large worlds. Yjs wasn't designed to be a database. Architecturally beautiful but practically fragile at scale.

---

### 3. "The PowerSync Path" — Tauri + SQLite + PowerSync

**Core Concept:** Use PowerSync (open-source sync framework) to handle bidirectional SQLite sync between instances, with a Postgres server as the sync hub.

**How it works:**
- Tauri app with SQLite locally, PowerSync SDK handles sync
- Self-hosted Postgres instance acts as the central truth
- PowerSync's Tauri SDK (released 2025) replaces browser WASM SQLite with native Rust-backed SQLite
- Rich text stored as JSON (ProseMirror format) in SQLite text columns
- Conflict resolution via PowerSync's built-in rules (per-table strategies)

**Strengths:** PowerSync handles the hardest part (sync) and has a dedicated Tauri SDK. Battle-tested at scale. SQLite everywhere means one query language. Open-source self-hostable.

**Risks:** Requires Postgres as the hub — heavier than a minimal sync server. Rich text conflicts resolved at the row level, not character level (no CRDT merging of concurrent text edits). Dependency on PowerSync's roadmap.

---

### 4. "The Hybrid" — SQLite for Structure, Yjs for Content, libSQL for Sync

**Core Concept:** Split the problem: use SQLite/libSQL for structured entity data with Turso-style embedded replicas for sync, and Yjs for rich text documents with a separate sync channel.

**How it works:**
- libSQL (SQLite fork by Turso) as the local database with embedded replica mode
- Structured data (entity types, fields, relations, permissions) syncs via libSQL replication
- Rich text content stored as Yjs binary in a separate document store, synced via y-websocket to a small relay
- Self-hosted sync hub runs both: a libSQL server + a y-websocket relay
- TipTap editor with Yjs provider for real-time + offline text editing
- Maps stored as structured data (pins, layers) in SQLite + image files synced separately

**Strengths:** Each data type gets the right sync mechanism. Character-level merge for text, row-level sync for structured data. libSQL is SQLite-compatible so all queries still work. Turso's embedded replica mode is designed exactly for this.

**Risks:** Two sync channels to manage. libSQL's offline writes feature was beta as of late 2024 (may have matured). More moving parts than approach #1.

---

### 5. "The Web-First" — SvelteKit App + SQLite via WASM, Optional Tauri Wrapper

**Core Concept:** Build a SvelteKit web app that runs SQLite in the browser via WASM (sql.js/cr-sqlite), then wrap it in Tauri for desktop. Same codebase, two deployment targets.

**How it works:**
- SvelteKit app with cr-sqlite (CRDT-enhanced SQLite) running in-browser or in Tauri's webview
- cr-sqlite adds CRDT semantics directly to SQLite tables — merge without conflicts at the row/column level
- Self-hosted as a static site + tiny sync server, or run locally in Tauri
- TipTap for editing, document content stored as JSON in cr-sqlite columns
- Web users get the same offline-capable experience via service workers

**Strengths:** One codebase for web AND desktop. cr-sqlite gives you CRDT sync at the database level — no separate Yjs layer needed. Simplest mental model. Web version works immediately for single-instance users.

**Risks:** WASM SQLite in browsers is slower than native. cr-sqlite is newer/less proven than Yjs. Browser storage limits and persistence quirks (the exact problem LegendKeeper hit). Tauri version mitigates this with native filesystem, but web version inherits the fragility.

---

### 6. "The Plugin Architecture" — Core Engine + Module System

**Core Concept:** Build a minimal core (wiki + relations + sync) and make entity types, maps, timelines, and boards into loadable modules. Community can extend it.

**How it works:**
- Core: Tauri + Svelte + SQLite + Yjs — handles pages, links, search, sync, permissions
- Entity type system: JSON schema definitions that add structured fields to pages (Character template adds HP, Race, Class fields; Location template adds coordinates, climate)
- Built-in modules: Maps, Timelines, Boards ship as first-party modules
- Module API: defines new page types, custom views, sidebar widgets, and map pin behaviors
- Templates are shareable as JSON files

**Strengths:** Answers the "hybrid wiki + entity" question elegantly — entity types are just templates applied to wiki pages. Extensible without forking. Community can build game-system-specific modules (D&D 5e, Pathfinder, etc.). Keeps core simple.

**Risks:** Plugin APIs are hard to design well upfront. Risk of over-engineering the extension system before the core is solid. Module boundaries are the hardest architectural decision.

---

## Chosen Direction

**Combine #1 and #6** — "The Pragmatist" stack (Tauri + Svelte + SQLite + TipTap/Yjs) with the hybrid entity/wiki model from #6 where entity types are optional schemas layered onto free-form wiki pages. Hub-and-spoke sync with dual deployment model.

### Sync & Business Model Decision

The sync server is a standalone, first-class service — same codebase serves both:

- **Self-hosted:** Users run the sync server on their own VPS, Pi, NAS, or home server. Free, open-source.
- **Managed SaaS:** We host the sync server as a paid service for users who don't want to manage infrastructure.

This follows the open-core model (Bitwarden, Gitea, etc.). The desktop app is always free and fully functional offline. Sync is optional. The SaaS tier adds multi-tenant isolation, auth, usage metering, TLS, and managed backups.

**Architectural implication:** The sync server must be designed as a clean standalone service from day one — containerized, horizontally scalable, with a clear REST/WebSocket API. Not bolted onto the desktop app.

### MVP Scope

1. Wiki with nested pages, rich text (TipTap), and cross-linking
2. Entity type templates (Character, Location, Quest, etc.) as optional structure on pages
3. Interactive maps with pins linked to pages
4. SQLite + FTS5 search
5. Simple role permissions (GM/Player)
6. Export as JSON/HTML
7. Sync server (self-hostable + SaaS-ready)

## Remaining Open Questions

1. Frontend framework final pick — Svelte 5 vs SolidJS (leaning Svelte)
2. How important is the web version for v1 vs. desktop-only first?
3. Project name?

## Research Sources

- [Tauri Frontend Configuration](https://v2.tauri.app/start/frontend/)
- [Best UI Libraries for Tauri — CrabNebula](https://crabnebula.dev/blog/the-best-ui-libraries-for-cross-platform-apps-with-tauri/)
- [React vs Vue vs Svelte 2025 Performance](https://www.frontendtools.tech/blog/best-frontend-frameworks-2025-comparison)
- [TipTap GitHub](https://github.com/ueberdosis/tiptap)
- [TipTap + Yjs Integration](https://docs.yjs.dev/ecosystem/editor-bindings/tiptap2)
- [Which Rich Text Editor in 2025 — Liveblocks](https://liveblocks.io/blog/which-rich-text-editor-framework-should-you-choose-in-2025)
- [p2panda — Convergent Access Control CRDT](https://p2panda.org/2025/08/27/notes-convergent-access-control-crdt.html)
- [Local-First Software — Ink & Switch](https://www.inkandswitch.com/essay/local-first/)
- [PowerSync Tauri SDK](https://releases.powersync.com/announcements/introducing-the-powersync-tauri-sdk-alpha)
- [Building Local-First Tauri App with libSQL](https://dev.to/huakun/building-a-local-first-tauri-app-with-drizzle-orm-encryption-and-turso-sync-31pn)
- [Offline Sync & Conflict Resolution Patterns](https://www.sachith.co.uk/offline-sync-conflict-resolution-patterns-architecture-trade%E2%80%91offs-practical-guide-feb-19-2026/)
