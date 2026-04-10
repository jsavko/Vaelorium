# Design Spec: Milestone 7 — Permissions & Roles

**Status:** Draft
**Mockups:** `design/vaelorium-settings.pen` (Zw4vB)
**Depends on:** Milestone 1 (Wiki Engine)

---

## Overview

Permissions control who can see and edit what within a campaign. The system uses a simple role model (GM, Co-GM, Player) with per-page visibility controls and secret sections within pages. Permissions are advisory at the local level (data lives on-device) and enforced via sync filtering (the sync server only sends data a user's role permits).

---

## User Stories

- **As a GM**, I want to invite players to my campaign and assign them roles.
- **As a GM**, I want to control which pages each role can see (private, players, public).
- **As a GM**, I want to mark sections within a page as "GM only" so players can read the page but not see hidden content.
- **As a GM**, I want to preview what a specific player or role sees so I can verify I haven't accidentally revealed secrets.
- **As a Co-GM**, I want the same content access as the GM but without the ability to delete the campaign.
- **As a player**, I want to see only the pages and sections the GM has shared with me.

---

## Data Model

### SQLite Schema

```sql
-- Campaign members
CREATE TABLE members (
    id TEXT PRIMARY KEY,
    user_identifier TEXT NOT NULL,     -- email, username, or device ID
    display_name TEXT NOT NULL,
    role_id TEXT NOT NULL REFERENCES roles(id),
    avatar_color TEXT,                 -- hex color for avatar circle
    status TEXT DEFAULT 'invited',     -- 'invited', 'active', 'inactive'
    joined_at TEXT,
    last_seen_at TEXT
);

-- Roles
CREATE TABLE roles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,                -- "Game Master", "Co-GM", "Player"
    permissions TEXT NOT NULL,         -- JSON object of permission flags
    icon TEXT,                         -- lucide icon
    color TEXT,                        -- hex color for role badge
    is_builtin BOOLEAN DEFAULT FALSE,
    sort_order INTEGER DEFAULT 0
);

-- Page-level visibility overrides
CREATE TABLE page_visibility (
    page_id TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
    role_id TEXT REFERENCES roles(id) ON DELETE CASCADE,
    member_id TEXT REFERENCES members(id) ON DELETE CASCADE,
    -- One of role_id or member_id must be set
    can_view BOOLEAN DEFAULT TRUE,
    can_edit BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (page_id, COALESCE(role_id, ''), COALESCE(member_id, ''))
);

-- Indexes
CREATE INDEX idx_members_role ON members(role_id);
CREATE INDEX idx_page_visibility_page ON page_visibility(page_id);
```

### Built-in Roles

| Role | Icon | Color | Permissions |
|------|------|-------|-------------|
| Game Master | crown | #C8A55C (gold) | All access. Create, edit, delete anything. Manage members and roles. Campaign settings. |
| Co-GM | shield | #4A8C6A (green) | All content access. Cannot delete campaign, manage billing, or remove GM. |
| Player | eye | #5C7AB8 (blue) | View pages marked as visible to Players or to them specifically. Cannot edit unless explicitly granted. |

### Permission Flags (JSON)

```json
{
    "content.create": true,
    "content.edit_own": true,
    "content.edit_all": true,
    "content.delete": true,
    "content.view_private": true,
    "members.invite": true,
    "members.manage_roles": true,
    "campaign.settings": true,
    "campaign.delete": false
}
```

### Page Visibility Model

Each page has a `visibility` field with three levels:
- **`private`** — Only GM and Co-GM roles can see it (default)
- **`players`** — All campaign members can see it
- **`public`** — Anyone with a link can see it (web publishing, future)

Additionally, `page_visibility` table allows per-member overrides (e.g., sharing a specific page with one player but not others).

### Secret Sections (TipTap Extension)

The `SecretBlock` TipTap node wraps content in a collapsible GM-only section:

```html
<secret-block visibility="gm_only" revealed-to="member_id_1,member_id_2">
    <p>This content is only visible to the GM and specifically revealed members.</p>
</secret-block>
```

**Rendering:**
- GM view: Dashed border with eye icon, fully visible, expandable
- Player view: Hidden entirely (or replaced with "[Hidden content]" placeholder if configured)
- Per-player reveal: GM can reveal specific secrets to specific players

---

## Key Interactions

### Role Management (Settings)

**Layout (per mockup):**
- Role cards in a grid: GM (gold border, crown icon, "Owner" badge), Co-GM, Player
- Click card to edit permissions
- "+ Add Role" for custom roles (e.g., "Trusted Player" with edit permissions)

### Member Management

**Layout (per mockup):**
- Table: Avatar, Name/email, Role badge, Status (Online/Offline), actions
- "Invite" button opens invite dialog

**Invite flow:**
1. Click "Invite"
2. Enter email or generate a share code/link
3. Select role for the invitee
4. Invitee appears in table with "Invited" status
5. When they connect via sync, status changes to "Active"

### Per-Page Visibility

**Location:** Page metadata (top of editor or detail panel).

**Control:** Dropdown next to page title: Private (lock icon), Players (users icon), Public (globe icon).

**Override:** "Share with specific members" option opens a member picker.

### GM/Player View Toggle

**Location:** Top toolbar button "View as Player" or eye icon.

**Behavior:**
- Switches the entire UI to show only what a Player role would see
- Secret blocks hidden, private pages removed from tree
- Banner at top: "Viewing as Player — Exit preview"
- Dropdown to preview as a specific member

---

## Sync Integration

Permissions are enforced at the sync layer:

1. When a device syncs, it authenticates with a member token
2. The sync server checks the member's role
3. Only pages visible to that role are included in the sync payload
4. Secret block content is stripped from the Yjs document before sending to non-authorized members
5. This means a Player's local database never contains GM-only data

**Self-hosted sync:** Same behavior. The sync server always filters.

**No sync (offline only):** All data lives locally. Permissions are UI-only — a technical user could read the SQLite file directly. Acceptable tradeoff documented for users.

---

## Edge Cases

- **Role deletion:** Cannot delete built-in roles. Custom role deletion re-assigns members to Player.
- **Last GM:** Cannot remove or demote the last GM. Must transfer ownership first.
- **Member removal:** Removes member from campaign. Their local copy retains whatever was synced. Their sync access is revoked.
- **Visibility change:** Changing a page from "Players" to "Private" takes effect on next sync. Already-synced copies remain on player devices until they sync and the page is removed.

---

## Accessibility

- **Role cards:** Keyboard navigable grid. Selected role announced.
- **Member table:** Standard table navigation. Role change via dropdown.
- **Visibility dropdown:** Standard select element with keyboard support.
- **View-as toggle:** Clear banner and escape mechanism.
