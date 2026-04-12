---
status: in-progress
---
# Milestone 9: Import/Export

**Date:** 2026-04-11

---

## Goal

Export Tome data as JSON or Markdown for backup/sharing. Import from Markdown folders and other worldbuilding tools. Selective export by page or subtree.

## Chosen Approach

Build export commands in Rust that query all data and serialize to JSON/Markdown. Import reads files and creates pages/entities. Start with JSON export (lossless) and Markdown export (human-readable), then Markdown import.

## Tasks

### Export

- [ ] **1.** Create `src-tauri/src/commands/export.rs` — Rust commands:
  - `export_tome_json() -> String` — full Tome export as JSON (pages, content, entity types, fields, values, relations, maps, pins, timelines, events, boards)
  - `export_tome_markdown(path: String)` — export pages as .md files in a directory structure

- [ ] **2.** Register commands, add to bridge mock.

- [ ] **3.** Create `src/lib/api/export.ts` — TypeScript API wrappers.

- [ ] **4.** Build export UI — "Export" option in settings or ⋯ menu with format picker (JSON/Markdown) and save dialog.

### Import

- [ ] **5.** Create `src-tauri/src/commands/import_data.rs` — Rust commands:
  - `import_markdown_folder(path: String)` — read .md files, create pages with content
  - `import_json(json: String)` — import full JSON export

- [ ] **6.** Register commands, add to bridge mock.

- [ ] **7.** Build import UI — "Import" option in settings with format picker and file/folder dialog.

### Testing

- [ ] **8.** Write E2E tests + verify no regressions.

## Notes

- JSON export includes everything — pages, Yjs content (as base64), entity types, fields, values, relations, maps, pins, timelines, events, boards, images.
- Markdown export creates a folder with one .md file per page. Frontmatter includes entity type, fields. Wiki links converted to relative links.
- Import is additive — doesn't replace existing data, creates new pages alongside existing ones.
- LegendKeeper/Kanka JSON import deferred — focus on our own format first.
