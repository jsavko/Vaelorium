# Phase 5.0 — Path-Independent Tome Identity

**Date:** 2026-04-13
**Plan:** `.claude/plans/archive/2026-04-13-m7-sync-phase5-0-tome-uuid.md`

## Summary

Decoupled the sync bucket prefix from the local `.tome` file path. Each Tome now carries a stable UUID in `tome_metadata` that is used directly as the bucket folder segment (`tomes/{uuid}/`). Prereq for Phase 5.1 recovery: without this, a second device restoring a Tome would resume sync under a different prefix than the first device, orphaning existing bucket data.

## Changes

### Features
- `sync::tome_identity::get_or_create_uuid(pool)` — lazy-creates a per-Tome UUID in `tome_metadata` on first access, returns the same value on repeat calls.

### Refactoring
- `sync::backend::prefixed::tome_prefix` now takes a UUID and emits `tomes/{uuid}/` directly. Previous SHA-256-of-local-path scheme removed.
- `commands::sync::build_tome_backend` now takes `(&AppHandle, &SqlitePool)` and resolves the Tome UUID internally. All three callers (`sync_now`, `sync_take_snapshot`, `runner`) updated.
- `commands::sync::sync_enable` materializes the Tome UUID immediately on enable so the prefix is locked in before the first op ships.

### Breaking change (acceptable: no production users)
- Buckets with data under the old path-derived prefix become orphaned; users must delete old `tomes/*` entries and re-enable sync. Documented in `docs/sync-manual-testing.md`.

## Files Modified
- `src-tauri/src/sync/tome_identity.rs` (new)
- `src-tauri/src/sync/mod.rs` — register module
- `src-tauri/src/sync/backend/prefixed.rs` — rewrite `tome_prefix`, replace test
- `src-tauri/src/commands/sync.rs` — `build_tome_backend` signature + three call sites; `sync_enable` pre-materializes UUID
- `src-tauri/src/sync/runner.rs` — pass pool to `build_tome_backend`
- `docs/sync-manual-testing.md` — upgrade note
- `CLAUDE.md` — Tome identity bullet in Sync section

## Rationale
The old `tome_prefix(path)` scheme hashed the local file path; the same logical Tome on two machines produced two unrelated prefixes, which would break cross-device restore. Moving identity into the Tome itself (via `tome_metadata`) means the bucket structure is determined by the Tome, not by where any given device stores the file.

## Test evidence
- 53 library unit tests passing (3 new for `tome_identity`)
- 11 sync integration scenarios passing (A–F + registry checks)
