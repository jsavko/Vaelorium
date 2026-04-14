# Architecture briefs

Short pointer docs for the main subsystems. Each brief is <120 lines — file:symbol pairs + gotchas, not prose architecture.

Start here for a new task: find the subsystem your task touches, read its brief, then grep for the specific symbol you need. If the task spans subsystems, read each brief's "Responsibility" section only.

| Brief | Subsystem |
|---|---|
| [sync.md](./sync.md) | Sync engine — schema registry, journal, runner, apply paths |
| [backup.md](./backup.md) | Backup destination — filesystem / S3 / hosted dispatch, snapshots, restore, delete |
| [cloud.md](./cloud.md) | Vaelorium Cloud hosted backend — `cloud_*` commands, account / quota, HostedBackend (M5) |
| [tomes.md](./tomes.md) | Tomes & registry — metadata, stable `tome_uuid`, recent_tomes, picker, create/restore |
| [pages-editor.md](./pages-editor.md) | Pages & editor — TipTap, page_content BLOB, wiki links, mention, slash menu |
| [ui-theming.md](./ui-theming.md) | UI / theming — design tokens, theme switching, ConfirmDialog / Toast conventions |
| [commands-registry.md](./commands-registry.md) | Tauri command bridge — `#[tauri::command]`, callCommand, camelCase rule |
| [file-section-map.md](./file-section-map.md) | Line-range maps for monolith files (Settings.svelte, backup.rs, Editor.svelte) |

## Maintenance

- Pointers should reference **module paths + function names**. Line numbers are allowed only for stable anchors (migrations registry, tauri.conf.json keys).
- If a brief grows past ~120 lines, that's a signal the subsystem needs splitting — not that the brief needs expanding.
- Every paragraph that isn't a pointer rots. Prefer a 1-line bullet over a paragraph.
- If you land a refactor that moves a named function, grep the briefs for the old name before committing.

## Validator

`scripts/check-architecture-docs.sh` enforces that every file path and namespaced symbol (`mod::fn`) referenced in a brief still resolves in the working tree. It runs automatically via a `PreToolUse` Bash hook on `git commit` (see `.claude/settings.json`) and will fail the commit on first miss.

**Limits:**
- Only catches *missing* references, not *incorrect* ones (a renamed fn with the wrong description still passes).
- Symbol check is presence-only (`grep -w`) — it won't notice if the symbol exists but in the wrong subsystem.
- Not a substitute for human review when a brief becomes misleading.

Run manually:
```
bash scripts/check-architecture-docs.sh
```
