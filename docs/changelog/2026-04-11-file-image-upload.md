# File-Based Image Upload for Wiki Pages

**Date:** 2026-04-11

## Summary

Images can now be uploaded from local files instead of only accepting URLs. Images are stored as blobs inside the Tome's SQLite database for full portability. Supports file picker, drag-and-drop, and both inline editor images and featured images.

## Changes

### Features
- Native file picker for image upload (Tauri dialog plugin on desktop, browser file input on web)
- Drag-and-drop image files onto the editor inserts them inline
- Images stored as blobs in SQLite `images` table inside the Tome
- Featured image in details panel uses file picker instead of URL prompt
- Image API: upload, get, delete, list commands

### Backend
- `tauri-plugin-dialog` and `tauri-plugin-fs` added for native file dialogs
- `004_images.sql` migration with images table (id, filename, mime_type, data blob)
- `images.rs` commands: upload_image, upload_image_data, get_image, delete_image, list_images
- Bridge mock stores images in memory

### Files Modified
- `src-tauri/Cargo.toml` — dialog + fs plugins
- `src-tauri/capabilities/default.json` — dialog + fs permissions
- `src-tauri/src/lib.rs` — register plugins + image commands
- `src-tauri/src/commands/images.rs` — new module
- `src-tauri/src/commands/mod.rs` — register images module
- `src-tauri/src/db/migrations.rs` — 004_images migration
- `src-tauri/migrations/004_images.sql` — new migration
- `src/lib/api/images.ts` — new TS API wrappers
- `src/lib/api/bridge.ts` — image mock commands
- `src/lib/components/Editor.svelte` — file picker + drag-drop for images
- `src/lib/components/DetailsPanel.svelte` — file picker for featured image
- `package.json` — @tauri-apps/plugin-dialog, @tauri-apps/plugin-fs
