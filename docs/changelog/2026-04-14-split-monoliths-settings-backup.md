# Split Settings.svelte + backup.rs monoliths; add docs validator

**Date:** 2026-04-14

## Summary

Splits the two worst-offender monolith files into per-concern units along their natural fault lines (tab boundaries for Settings, function-group boundaries for backup.rs), updates every `docs/architecture/*.md` pointer that referred to them, and adds a validator script + git pre-commit hook so future staleness fails the commit rather than silently rotting.

## Changes

### Refactoring
- **`src/lib/components/Settings.svelte`** — 1,341 lines → 205-line shell + six per-tab components (`SettingsKeybinds`, `SettingsAppearance`, `SettingsData`, `SettingsBackup`, `SettingsSync`, `SettingsAbout`). Shell keeps shared controls (`.data-btn`, `.settings-section-title`, etc.) as `:global()` so tabs don't redeclare them.
- **`src-tauri/src/commands/backup.rs`** — 986 lines → `src-tauri/src/commands/backup/` directory with `mod.rs` (shared types + `ensure_device_name_stub` + unit tests), `config.rs`, `unlock.rs`, `restore.rs`, `delete.rs`. `lib.rs` registration paths updated (`commands::backup::config::backup_configure`, etc. — Tauri's `#[tauri::command]` macro keys on the module the fn was annotated in, so re-exports don't suffice).
- `src-tauri/src/commands/sync.rs` — updated `use crate::commands::backup as backup_cmd` to `use crate::commands::backup::config as backup_cmd`.

### Features
- **`scripts/check-architecture-docs.sh`** — greps every `docs/architecture/*.md` for referenced paths + `mod::fn` symbols, verifies each resolves in the tree, exits non-zero on first miss.
- **`.githooks/pre-commit`** — runs the validator. Enable per-clone with `git config core.hooksPath .githooks` (note added to CLAUDE.md).
- **`.claude/settings.local.json`** — `PostToolUse` hook on `Edit`/`Write` emits a non-blocking reminder when a brief's references have gone stale (complements the blocking git hook).

### Documentation
- `CLAUDE.md` — Codebase Map row for Backup updated to the new submodule list. Monolith-files line drops Settings + backup.rs, adds a pointer to the "Completed splits" section. New Git hooks section.
- `docs/architecture/backup.md` — Commands section rewritten around the new submodules, with a call-out on the Tauri macro gotcha.
- `docs/architecture/file-section-map.md` — drops the Settings + backup.rs line-range tables (obsolete); adds a "Completed splits" section documenting what landed where.
- `docs/architecture/README.md` — new "Validator" section with limits.
- `docs/architecture/sync.md` — fixed stale `pages::save_page` → `pages::update_page` and `src/lib/yjs/` → `src/lib/editor/YjsProvider.ts` references caught by the validator.
- `docs/architecture/tomes.md` — updated `commands/backup.rs` reference to `commands/backup/`.

### Files added
- `src/lib/components/SettingsKeybinds.svelte`
- `src/lib/components/SettingsAppearance.svelte`
- `src/lib/components/SettingsData.svelte`
- `src/lib/components/SettingsBackup.svelte`
- `src/lib/components/SettingsSync.svelte`
- `src/lib/components/SettingsAbout.svelte`
- `src-tauri/src/commands/backup/mod.rs`
- `src-tauri/src/commands/backup/config.rs`
- `src-tauri/src/commands/backup/unlock.rs`
- `src-tauri/src/commands/backup/restore.rs`
- `src-tauri/src/commands/backup/delete.rs`
- `scripts/check-architecture-docs.sh`
- `.githooks/pre-commit`

### Files removed
- `src-tauri/src/commands/backup.rs` (content moved into `backup/` submodules)

### Verification
- `npm run check` — 3,954 files clean (0 errors, 0 warnings).
- `cargo test --lib` — 84 passed, 0 failed.
- `scripts/check-architecture-docs.sh` — clean.

### Rationale

Svelte components are the project's native composition unit; the 1,341-line Settings was an accident of history, not a design choice. Same for backup.rs — its fault lines (connect / unlock / restore / delete) are already clear in the code structure, they just needed to become file boundaries. The validator + git hook pair ensures the architecture briefs don't silently rot as the codebase evolves — broken references now fail commits rather than misleading future sessions.

### Known limits
- Validator is presence-only — won't catch a rename that replaces a symbol with a differently-named one of similar shape.
- Runtime verification (each Settings tab still renders + its primary action still works) requires the desktop app; WSL + WebKitGTK can't drive Chrome DevTools MCP, so this step is on the user.
