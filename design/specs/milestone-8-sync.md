# Design Spec: Milestone 8 — Sync Server

**Status:** Draft
**Mockups:** `design/vaelorium-settings.pen` (b4D1L)
**Depends on:** Milestone 1 (Wiki Engine), Milestone 7 (Permissions)

---

## Overview

The sync server is a standalone service that enables multi-device and multi-user collaboration. It relays Yjs document updates, syncs structured SQLite data, and transfers files. The same codebase serves both self-hosted deployments (single Docker container) and the managed SaaS offering (multi-tenant with billing).

---

## User Stories

- **As a GM**, I want to sync my campaign between my desktop and laptop so I can work from either device.
- **As a GM**, I want to invite players and have them see updates in near-real-time when we're both online.
- **As a GM**, I want sync to work gracefully when offline — changes queue up and merge when I reconnect.
- **As a GM**, I want to self-host the sync server on my own VPS or Raspberry Pi.
- **As a user**, I want to use the managed Vaelorium sync service if I don't want to run my own server.
- **As a GM**, I want to see the sync status (connected, syncing, offline) in the app at all times.
- **As a GM**, I want to see which devices are connected and when they last synced.

---

## Architecture

### Sync Server Components

```
Sync Server (Rust — Axum)
├── Auth Module
│   ├── Token validation (JWT)
│   ├── Device registration
│   └── Member role lookup
├── Yjs Relay
│   ├── WebSocket endpoint (/ws/yjs/{campaign_id}/{page_id})
│   ├── Document state persistence (server-side Yjs store)
│   └── Awareness protocol (cursor positions, online status)
├── Structured Data Sync
│   ├── REST endpoint (/api/sync/{campaign_id})
│   ├── Change log pull (GET — "give me changes since version X")
│   ├── Change log push (POST — "here are my local changes")
│   └── Conflict resolution (per-field last-write-wins with timestamps)
├── File Sync
│   ├── Upload endpoint (/api/files/{campaign_id})
│   ├── Download endpoint
│   └── Storage backend (filesystem for self-hosted, S3 for SaaS)
└── Admin (SaaS only)
    ├── Multi-tenant namespace isolation
    ├── Usage metering
    └── Billing webhook integration
```

### Sync Protocol

**Yjs Document Sync (rich text content):**
1. Client opens WebSocket to `/ws/yjs/{campaign_id}/{page_id}`
2. Server sends current Yjs state vector
3. Client computes diff and sends state update
4. Server merges update (Yjs CRDT — automatic conflict resolution)
5. Server broadcasts update to all other connected clients
6. If client is offline: changes accumulate in local Yjs doc. On reconnect, full state sync resolves all divergence.

**Structured Data Sync (SQLite rows — pages metadata, entity fields, relations, etc.):**
1. Each row has a `sync_version` (monotonic counter) and `updated_at` timestamp
2. Client calls `GET /api/sync/{campaign_id}?since={last_sync_version}`
3. Server returns all rows changed since that version
4. Client applies remote changes to local SQLite, resolving conflicts:
   - **Same row, same field:** Last-write-wins based on `updated_at`
   - **Same row, different fields:** Merge (both changes apply)
   - **Delete vs. edit:** Delete wins (tombstone marker)
5. Client calls `POST /api/sync/{campaign_id}` with its local changes
6. Server applies, increments version, responds with any server-side changes

**File Sync (images, map files):**
1. Files referenced by content have a hash-based identifier
2. Client checks which files the server has: `GET /api/files/{campaign_id}/manifest`
3. Missing files are uploaded/downloaded as needed
4. Files are immutable — edits create new files (old ones garbage-collected)

### Change Log Table (added to client SQLite)

```sql
CREATE TABLE sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    row_id TEXT NOT NULL,
    operation TEXT NOT NULL,           -- 'INSERT', 'UPDATE', 'DELETE'
    changed_fields TEXT,               -- JSON array of field names
    timestamp TEXT NOT NULL,           -- ISO 8601
    sync_version INTEGER DEFAULT 0,    -- 0 = not yet synced
    synced_at TEXT                     -- NULL until synced
);

CREATE INDEX idx_sync_log_unsynced ON sync_log(sync_version) WHERE sync_version = 0;
```

---

## Sync Server Deployment

### Self-Hosted (Docker)

```yaml
# docker-compose.yml
services:
  vaelorium-sync:
    image: vaelorium/sync-server:latest
    ports:
      - "8080:8080"
    environment:
      - DATABASE_URL=sqlite:///data/sync.db
      - STORAGE_PATH=/data/files
      - JWT_SECRET=your-secret-here
    volumes:
      - ./data:/data
```

Single container, SQLite backend, filesystem storage. Runs on any machine with Docker.

### SaaS (Managed)

- Multi-tenant Postgres backend (one schema per campaign or row-level isolation)
- S3-compatible storage (Cloudflare R2, AWS S3, MinIO)
- Horizontal scaling: stateless Axum servers behind load balancer
- WebSocket sticky sessions for Yjs relay
- Usage metering: storage used, bandwidth, active devices
- Billing: Stripe integration, subscription tiers

---

## Key Interactions (Client-Side)

### Sync Status Indicator

**Location:** Top toolbar, right side (shown in wiki editor mockup as green dot + "Synced").

**States:**
| State | Icon | Text | Color |
|-------|------|------|-------|
| Connected & synced | Green dot | "Synced" | #5C8A5C |
| Syncing | Spinning arrows | "Syncing..." | #C8A55C |
| Offline (changes pending) | Orange dot | "Offline — 3 pending" | #C8A55C |
| Offline (no changes) | Gray dot | "Offline" | foreground-tertiary |
| Error | Red dot | "Sync error" | #B85C5C |
| Not configured | No dot | "No sync" | foreground-tertiary |

Clicking the indicator opens a popover with details (last sync time, pending changes count, retry button).

### Sync Settings Page (per mockup)

**Connection section:**
- Server URL input
- Auth token input (masked)
- Auto-sync toggle (default: ON)
- "Test Connection" button
- "Disconnect" button

**Status card:**
- Connection status with green/red indicator
- Server URL, last sync time, device count, campaign size
- "Sync Now" button for manual trigger

**Connected Devices list:**
- Device name, platform, last sync time, online status dot
- Current device marked as "This device"

### First-Time Setup

**Flow for managed SaaS:**
1. Settings → Sync → "Connect to Vaelorium Sync"
2. Sign in with email (creates account or logs in)
3. Auto-generates token and fills server URL
4. "Enable Sync" button
5. Initial full sync begins (progress bar)

**Flow for self-hosted:**
1. Settings → Sync → "Connect to server"
2. Enter server URL manually
3. Enter auth token (generated from server admin panel or CLI)
4. "Test Connection" verifies connectivity
5. "Enable Sync" begins initial sync

---

## Edge Cases

- **First device sync:** Full upload of all local data. Server creates campaign namespace.
- **Second device joins:** Full download of campaign data from server. Merge with any local data (unlikely for fresh install).
- **Conflicting page edits:** Yjs handles rich text conflicts. Structured data uses per-field last-write-wins. No user-facing conflict dialogs in MVP.
- **Server unreachable:** App continues working fully offline. Changes queue in sync_log. Retry with exponential backoff (5s, 15s, 30s, 60s, then every 5 min).
- **Token revoked:** Server returns 401. Client shows "Sync disconnected — re-authenticate" message.
- **Large campaigns:** Initial sync for campaigns > 100MB shows progress with estimated time. Syncs incrementally after initial load.
- **Clock skew:** All timestamps are UTC. Server timestamp is authoritative for conflict resolution ordering.

---

## Security

- **Transport:** All sync communication over TLS (HTTPS/WSS). Self-hosted users must configure TLS (documented, with Let's Encrypt guidance).
- **Auth:** JWT tokens with expiration. Refresh token flow for long-lived sessions.
- **Data isolation:** Each campaign is a separate namespace. No cross-campaign data leakage.
- **Permission filtering:** Sync server checks member role before returning data. Players never receive GM-only content (see Milestone 7 spec).
- **At-rest encryption:** Optional for self-hosted (documented). SaaS uses encrypted storage.

---

## Performance

- **WebSocket connections:** Server should handle 100 concurrent connections per campaign, 10,000 total per instance.
- **Sync latency:** < 500ms for structured data sync round-trip on a healthy connection.
- **Yjs relay:** < 100ms document update relay between connected clients.
- **Initial sync:** Aim for 10MB/s throughput. 100MB campaign syncs in ~10 seconds.

---

## Accessibility

- **Sync status:** Announced to screen readers when state changes. "Campaign synced" / "3 changes pending" / "Sync error."
- **Settings page:** Standard form controls, fully keyboard navigable.
- **Progress indicators:** ARIA live regions for sync progress updates.
