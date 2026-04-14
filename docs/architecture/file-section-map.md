# Monolith file-section map

Use with `Read offset/limit` so large files don't pull their whole body into context.

## `src/lib/components/Settings.svelte` (1,341 lines)

Tab-routed modal; the six tabs are defined at ~line 34 and rendered in one big `{#if activeTab === ...}` chain:

| Section | Lines | Notes |
|---|---|---|
| `<script>` imports + tab definition | 1–90 | Stores, API wrappers, tab list |
| Sync-tab helpers (state, handlers) | 90–270 | `configureBackup`, `handleEnableSync`, `handleDisableSync` |
| Cloud account helpers | 145–220 | `refreshCloudAccount` (shim), `handleCloudSignoutOnly`, `handleDisconnectBackup` |
| Other tab handlers (appearance, data, keybinds) | 270–500 | Theme set, export/import, keybind editor |
| **Template: Keybinds tab** | 508–536 | |
| **Template: Appearance tab** | 537–572 | |
| **Template: Data tab** | 573–588 | Export / import buttons |
| **Template: Backup tab** | 589–769 | Configure, unlock, cloud account card, disconnect. Longest section. |
| **Template: Sync tab** | 770–868 | Per-Tome enable / disable / sync-now / activity |
| **Template: About tab** | 869–915 | Version, links |
| `<style>` | 915–end | Dense; includes `.quota-banner`, `.sync-status-card`, `.account-row` |

Planned split (separate follow-up plan): `SettingsKeybinds.svelte`, `SettingsBackup.svelte`, `SettingsSync.svelte`, etc.

## `src-tauri/src/commands/backup.rs` (986 lines)

Groupings (functions at approximate line anchors — verify with `grep -n 'pub async fn' ...` if moved):

| Function group | Line | Purpose |
|---|---|---|
| `backup_configure` | ~49 | Write `app_backend.json`, set up keychain |
| `backup_disconnect` | ~197 | Tear down backup config |
| `backup_status` | ~214 | App-global status payload |
| `backup_set_device_name` | ~286 | Rename device |
| `backup_unlock` | ~304 | Passphrase → `SessionState` |
| `backup_try_auto_unlock` | ~365 | OS keychain auto-unlock |
| `build_raw_backend` / `parse_s3_config` | ~390 / ~424 | Backend factory |
| `backup_list_restorable_tomes` | ~537 | Filesystem/S3 restore listing |
| `backup_restore_tome` | ~614 | Pull snapshot → write `.tome` → seed sync_config |
| `list_hosted_restorable_tomes` | ~767 | Hosted-specific restore listing (via `GET /v1/tomes`) |
| `backup_delete_tome` | ~882 | Delete cloud copy |
| `peek_tome_metadata` / `sanitize_filename` | ~950 / ~971 | Helpers |

## `src/lib/components/Editor.svelte` (894 lines)

| Section | Lines |
|---|---|
| `<script>` — imports + image toolbar helpers | 1–80 |
| `loadEditor`, title, drop/paste handlers, embeds | 80–240 |
| TipTap init + Y.js wiring | 240–430 |
| Template | 430–438 |
| `<style>` | 438–end |

## `src/lib/components/TomePicker.svelte` (637 lines)

| Section | Lines |
|---|---|
| `<script>` — stores, restore handlers | 1–145 |
| Template — recent cards + restore panel | 145–430 |
| `<style>` | 430–end |

## `src/lib/components/Sidebar.svelte` (634 lines)

Single-file: nav + page tree + context menu. No internal tab routing. Split by template section headers when editing.

## Update policy

If you move a function, grep this file for its name before committing:
```
grep -n "<fn_name>" docs/architecture/file-section-map.md
```
Otherwise the anchors rot.
