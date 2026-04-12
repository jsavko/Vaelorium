---
status: completed
---
# Change Tome File Extension from .vaelorium to .tome

**Date:** 2026-04-12

---

## Goal

Rename the Tome file extension from `.vaelorium` to `.tome` for brevity and clarity. The file format itself doesn't change — it's still a SQLite database. Existing `.vaelorium` files from beta testers must still open.

## Approaches Considered

### 1. Accept Both Extensions in File Dialogs, Default to .tome
- **Description:** File open dialogs accept both `.tome` and `.vaelorium` extensions. Save dialogs default to `.tome`. Auto-migration of legacy DB creates `.tome` files.
- **Pros:** Zero breaking change for existing users. Gradual migration. Simple implementation.
- **Cons:** Two extensions in the wild.

### 2. Rename Only, Force Migration
- **Description:** Only accept `.tome`. Show a "rename required" dialog for `.vaelorium` files.
- **Pros:** Clean — one extension going forward.
- **Cons:** User-facing migration friction. Breaks existing file associations.

### 3. Alias — Treat .vaelorium as .tome
- **Description:** Internally both extensions map to the same format. No migration needed.
- **Pros:** Simplest implementation.
- **Cons:** Same as #1 essentially — just drops the distinction.

## Chosen Approach

**Approach 1: Accept Both Extensions.** File open dialogs accept both `.tome` and `.vaelorium`. Save/create dialogs default to `.tome`. Auto-migration still creates `.tome`. Legacy users just keep using their existing files, and any new file created gets the new extension.

## Tasks

- [x] **1.** Update `CreateTomeModal.svelte` — save dialog filter to use `.tome` as default, include `.vaelorium` as fallback.

- [x] **2.** Update `TomePicker.svelte` — open dialog filter to accept both `.tome` and `.vaelorium`.

- [x] **3.** Update `lib.rs` migrate_legacy_db — create `My Campaign.tome` instead of `.vaelorium`.

- [x] **4.** Update Tauri config `tauri.conf.json` — add file type association for `.tome`.

- [x] **5.** Update the Create Tome modal body text to reference `.tome`.

- [x] **6.** Verify tests pass.

## Notes

- The file format is unchanged — it's a SQLite database either way.
- Existing `.vaelorium` files continue to work since the open dialog accepts both.
- Documentation and design specs can remain using `.vaelorium` for now — update in a polish pass later.
- Could add a "Rename to .tome" option later to help users consolidate.
