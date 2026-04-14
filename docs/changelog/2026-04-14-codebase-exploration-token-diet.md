# Shrink the token cost of exploring this codebase

**Date:** 2026-04-14

## Summary

Introduces a **Codebase Map** section in `CLAUDE.md` plus seven short pointer briefs under `docs/architecture/`, so future sessions don't re-derive the sync / backup / cloud / tomes / pages layout from scratch. A 2026-04-14 planning task burned ~62k tokens on that exact re-discovery; these artifacts are meant to cut similar costs to a few thousand.

## Changes

### Features
- **CLAUDE.md** gains a Codebase Map (subsystem → brief + key files) and a "Before exploring" escalation ladder (memory → CLAUDE.md → brief → Grep → offset-Read → Explore as last resort).
- **`docs/architecture/`** (new) — short pointer briefs:
  - `README.md` — index.
  - `sync.md`, `backup.md`, `cloud.md`, `tomes.md`, `pages-editor.md`, `ui-theming.md`, `commands-registry.md` — one subsystem each, <120 lines, file:symbol pointers + gotchas (no prose architecture).
  - `file-section-map.md` — line-range anchors for monolith files (Settings.svelte 1,341, backup.rs 986, Editor.svelte 894, TomePicker 637, Sidebar 634) so `Read offset/limit` can target the right section.
- **Auto-memory** — new `feedback_explore_cheap_first.md` records the escalation discipline with the 62k incident as the reason line.

### Files Added
- `docs/architecture/README.md`
- `docs/architecture/sync.md`
- `docs/architecture/backup.md`
- `docs/architecture/cloud.md`
- `docs/architecture/tomes.md`
- `docs/architecture/pages-editor.md`
- `docs/architecture/ui-theming.md`
- `docs/architecture/commands-registry.md`
- `docs/architecture/file-section-map.md`

### Files Modified
- `CLAUDE.md` — +Codebase Map table, +"Before exploring" subsection.

### Rationale

Every session was starting subsystem discovery from zero, spawning Explore agents that produced long uncommitted reports. Committing the pointer map to the repo makes it reusable across sessions, improves human on-ramp, and (because `CLAUDE.md` is auto-loaded) the top-level index is effectively free context.

Monolith-component refactor (splitting Settings.svelte into per-tab components, `backup.rs` into function-group modules) is the real structural fix and is staged as a separate follow-up — this plan is low-risk documentation leverage only.
