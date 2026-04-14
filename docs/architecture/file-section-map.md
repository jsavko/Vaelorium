# Monolith file-section map

Use with `Read offset/limit` so large files don't pull their whole body into context.

**Status (2026-04-14):** `Settings.svelte` (1,341 → 205) and `commands/backup.rs` (986 → 5 submodules ≤378 each) have been split along natural fault lines. Remaining monoliths below are tracked for potential follow-up splits; for now their section maps let targeted reads avoid full-file loads.

## `src/lib/components/Editor.svelte` (894 lines)

| Section | Lines |
|---|---|
| `<script>` — imports + image toolbar helpers | 1–80 |
| `loadEditor`, title, drop/paste handlers, embeds | 80–240 |
| TipTap init + Y.js wiring | 240–430 |
| Template | 430–438 |
| `<style>` | 438–end |

High-risk to split (TipTap + Y.js lifecycle is tightly coupled). Read with offset/limit targeted at the section you need.

## `src/lib/components/TomePicker.svelte` (637 lines)

| Section | Lines |
|---|---|
| `<script>` — stores, restore handlers | 1–145 |
| Template — recent cards + restore panel | 145–430 |
| `<style>` | 430–end |

Medium-risk split candidate: the `syncingUuids` derivation is shared across the recents grid and the restore-filter, so extraction would pass it as a prop.

## `src/lib/components/Sidebar.svelte` (634 lines)

Single-file: nav + page tree + context menu. No internal tab routing. Split by template section headers when editing.

## `src/lib/components/BackupSetupWizard.svelte` (562 lines)

Five-step wizard with a shared state object; plausible split but moderate risk.

## Completed splits

- **`src/lib/components/Settings.svelte`** — split into `Settings.svelte` (shell, ~200 lines) + per-tab components: `SettingsKeybinds`, `SettingsAppearance`, `SettingsData`, `SettingsBackup`, `SettingsSync`, `SettingsAbout`. Shell holds sidebar nav + shared controls CSS (as `:global()`); each tab component owns its state, handlers, and specific styles. See `src/lib/components/Settings*.svelte`.
- **The former `commands/backup` monolith** — split into `src-tauri/src/commands/backup/` submodules: `config.rs` (connect/disconnect/status/factory), `unlock.rs`, `restore.rs`, `delete.rs`. `mod.rs` holds shared payload types + `ensure_device_name_stub`. Registration paths in `lib.rs` updated to `commands::backup::<submodule>::<fn>`.

## Update policy

If you move a function, grep this file for its name before committing:
```
grep -n "<fn_name>" docs/architecture/file-section-map.md
```
Otherwise the anchors rot. The `scripts/check-architecture-docs.sh` validator catches missing file paths + namespaced symbols, but not renamed sections.
