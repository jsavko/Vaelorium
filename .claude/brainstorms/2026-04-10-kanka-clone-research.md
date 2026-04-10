---
status: research-complete
---
# Self-Hosted Offline-First LegendKeeper Alternative

**Date:** 2026-04-10

---

## Problem Statement

LegendKeeper's creator Braden Herndon originally championed local-first principles, stating users should feel their creative work "belongs to *YOU*, and not some rando business's server in the cloud somewhere." He built toward this vision with the Hydra update (2021), which added offline wiki/atlas support and a promised desktop app for Win/Mac/Linux. However:

- The desktop app was teased ("Hydra makes desktop a reality!") but never officially shipped
- The FAQ now states "there are no plans" for open source or self-hosting
- Infrastructure moved from self-managed Kubernetes to managed Railway hosting — prioritizing SaaS simplicity
- Offline mode is browser-cache-dependent — clearing browser data before syncing loses your changes
- No self-hosting option exists; your data lives on their servers

The tool evolved from a local-first vision into a cloud-first SaaS at $9/month. Users who want true data ownership, offline-first operation, and self-hosted infrastructure have no good purpose-built option.

## Target Users

- Tabletop RPG game masters and players who want structured campaign/world data
- Worldbuilders who work offline or in low-connectivity environments
- Privacy-conscious users who want full data ownership
- Groups who want to collaborate across multiple instances without depending on a cloud service
- Users frustrated by SaaS lock-in and subscription fatigue for creative tools

## What LegendKeeper Offers (the feature target)

### Core Modules

1. **Wiki/Editor** — Rich-text pages with infinite nesting, auto-linking between pages, slash commands, @-mentions, templates, tags, featured images, custom properties, drag-and-drop organization
2. **Atlas (Maps)** — Upload maps up to 14K pixels/100MB, customizable pins with icons/colors, infinite nested map hierarchy (world → region → town → dungeon), tag-based filtering, pins link to wiki pages
3. **Timelines** — Three views: Chronicle (narrative), Gantt (visual bars), Calendar (daily). Custom calendar systems with personalized months/days/weeks. Events link to wiki pages. Scales from billions of years to minutes
4. **Boards (Whiteboards)** — Collaborative visual canvases for flow charts, faction diagrams, family trees, quest flows, DM screens, pointcrawls. Cards link to wiki pages
5. **Asset Library** — Centralized image/map storage organized by folders

### Cross-Cutting Features

- **Real-time collaboration** with cursor tracking and presence indicators
- **Granular permissions** — private by default, control what players can see
- **Secret/GM-only content** hidden within shared pages
- **Full-text search** across all content
- **Web publishing** via custom URLs for player access
- **Export** as HTML or JSON (always free, not paywalled)
- **Page templates** reusable across projects
- **Version control** for document change tracking

### Planned but Unreleased Features (from LK roadmap)

- Map layers, region drawing, pin grouping, tokens, fog of war, text annotations
- Community marketplace
- Mobile support
- Procedural generation
- Audio/video embedding

## Existing Alternatives & Their Gaps

| Tool | Strengths | Gaps |
|------|-----------|------|
| **LegendKeeper** | Best editor UX, great maps, real-time collab | SaaS-only, no self-host, offline is fragile (browser cache), $9/mo |
| **Kanka** | Open source (GPL), 20+ entity types, strong relations | PHP/MySQL server stack, no offline, SaaS-oriented architecture |
| **World Anvil** | Most features, 25+ templates, statblocks | SaaS, expensive premium, overwhelming UI, no offline |
| **Fantasia Archive** | Free, offline, cross-platform (Electron) | Outdated UI, limited features, no sync, no collaboration, seemingly stale |
| **Obsidian + plugins** | Great editor, offline, plugin ecosystem | Not purpose-built for RPG; no structured entity types, maps are hacky |
| **TiddlyWiki** | Offline single-file, hackable | Steep learning curve, not RPG-focused, poor multi-device story |
| **Campfire Write** | Good writing tools, character/world building | SaaS pivot, limited free tier, no self-host |

### Key Insight

LegendKeeper's approach (free-form wiki + maps + boards) is more flexible than Kanka's approach (fixed entity types with structured fields). LK lets you organize however you want; Kanka imposes a schema. Both have merits — the question is which philosophy to follow.

## Key Technical Insights

### Sync Architecture — CRDTs vs FIFO

User initially suggested FIFO for conflict resolution. Research strongly suggests **CRDTs (Conflict-free Replicated Data Types)** are the better fit:

- FIFO (last-write-wins queue) risks silent data loss when two instances edit the same entity offline
- CRDTs guarantee convergence across replicas without a central coordinator — mathematically proven
- For a worldbuilding tool where two GMs might edit the same page independently, CRDTs preserve both edits rather than one overwriting the other
- LegendKeeper itself uses CRDT-style sync for its collaborative editing (Yjs-like approach)
- However, CRDTs add complexity. For an MVP, a simpler approach (operational log with manual conflict resolution) might be pragmatic

### Cross-Platform Desktop Tech

- **Tauri 2.0** (Rust + web frontend) is the current leader — small binaries, native performance, SQLite support via plugins, works on Win/Mac/Linux
- Tauri + SQLite + libSQL/Turso or PowerSync is a proven pattern for offline-first with sync
- Electron is the alternative but much heavier (~100MB+ bundles)
- LegendKeeper's abandoned desktop app was Electron-based

### Web Option

- The same web frontend used in Tauri can be served as a standalone web app
- Could share 90%+ of the codebase between desktop and web versions
- Web version serves users who want a single always-on instance (self-hosted server) without needing sync

### Storage

- **SQLite** replaces MySQL/Postgres for local-first — embedded, zero-config, battle-tested
- **SQLite FTS5** provides full-text search with no external service (replaces Meilisearch/Algolia)
- Local filesystem replaces S3/cloud storage for images and map files
- Rich text documents can be stored as JSON (like ProseMirror/TipTap document format) or as Yjs CRDT binary blobs for sync-friendly storage

## Open Questions

1. **Sync topology** — Peer-to-peer between instances, or hub-and-spoke with an optional central server? P2P is more "self-hosted" but harder to implement.
2. **CRDT vs simpler sync** — Full CRDT support is powerful but complex. Would you accept a simpler model (e.g., per-document version vectors with manual merge for conflicts) for an MVP?
3. **Wiki-style (LegendKeeper) vs Entity-style (Kanka)?** — Free-form pages with nesting, or structured entity types with specific fields? Or a hybrid?
4. **Map support** — Interactive maps with pins and nesting is a killer feature but also complex. MVP or later?
5. **Collaboration model** — Single user per instance with sync, or multi-user per instance (with permissions/roles)?
6. **Frontend framework preference** — React, Vue, Svelte, or Solid for the web layer inside Tauri?
7. **Rich text editor** — TipTap (ProseMirror-based, what LK likely uses), Lexical (Meta), or something else?

## Research Sources

- [LegendKeeper Features](https://www.legendkeeper.com/features/)
- [LegendKeeper FAQ](https://www.legendkeeper.com/faq/)
- [LegendKeeper 0.9 Hydra Changelog](https://www.legendkeeper.com/changelog/legendkeeper-0-9-0-0-hydra/)
- [LegendKeeper 0.8.0.2 — Braden's local-first philosophy](https://www.legendkeeper.com/changelog/legendkeeper-0-8-0-2-small-fix-and-some-thoughts/)
- [LegendKeeper Patreon](https://www.patreon.com/legendkeeper)
- [LegendKeeper "In the Clouds" — infra migration](https://www.legendkeeper.com/lk-weekly-in-the-clouds/)
- [Kanka GitHub Repository](https://github.com/owlchester/kanka)
- [AlternativeTo — LegendKeeper Alternatives](https://alternativeto.net/software/legendkeeper/)
- [Offline Sync & Conflict Resolution Patterns](https://www.sachith.co.uk/offline-sync-conflict-resolution-patterns-architecture-trade%E2%80%91offs-practical-guide-feb-19-2026/)
- [Building a Local-First Tauri App with Drizzle ORM and Turso Sync](https://dev.to/huakun/building-a-local-first-tauri-app-with-drizzle-orm-encryption-and-turso-sync-31pn)
- [PowerSync Tauri SDK](https://releases.powersync.com/announcements/introducing-the-powersync-tauri-sdk-alpha)
- [FOSDEM 2026 — Local-First Track](https://fosdem.org/2026/schedule/track/local-first/)
- [The Cascading Complexity of Offline-First Sync](https://dev.to/biozal/the-cascading-complexity-of-offline-first-sync-why-crdts-alone-arent-enough-2gf)
- [Worldbuilding Tools Comparison](https://www.anima-roleplay.com/resources/tools-comparisons)
