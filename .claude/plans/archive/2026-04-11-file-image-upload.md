---
status: completed
---
# File-Based Image Upload for Wiki Pages

**Date:** 2026-04-11

---

## Goal

The editor's image insertion currently only accepts a URL via `prompt()`. It should also accept local files — both for inline images in the TipTap editor and for the featured image in the details panel. Images need to be stored in a way that works with the Tome system (portable `.vaelorium` files).

**Where this fits:** This is a standalone improvement that spans M1 (editor) and M2.5 (Tomes). It doesn't belong to M3 (Relations). Best addressed as an immediate standalone plan before moving to M3.

## Approaches Considered

### 1. Store Images as Base64 in SQLite (Inside the Tome)
- **Description:** Convert uploaded files to base64 and store them in a new `images` table inside the Tome's SQLite database. Reference images by ID in the editor HTML (`vaelorium://image/{id}`). Serve via a custom Tauri protocol handler.
- **Pros:** Fully portable — images travel with the `.vaelorium` file. No external file dependencies. Works offline. Backup is one file.
- **Cons:** Bloats the SQLite database. Large images (5MB+) degrade DB performance. Base64 encoding adds 33% size overhead.

### 2. Store Images as Blobs in SQLite
- **Description:** Same as #1 but store raw binary blobs instead of base64. SQLite handles blobs efficiently up to ~100MB per row.
- **Pros:** No base64 overhead. Still fully portable. SQLite blob I/O is fast with incremental reads.
- **Cons:** Still bloats DB size. Can't preview images by opening the file externally.

### 3. Store Images in a Companion Directory
- **Description:** Each Tome `foo.vaelorium` gets a companion `foo.vaelorium.images/` directory. Images stored as files, referenced by hash filename. The DB stores relative paths.
- **Pros:** No DB bloat. Standard file I/O. Images viewable externally. Good for very large images.
- **Cons:** Two things to manage (file + directory). Portability requires copying both. Easy to break references.

### 4. Store Images as Blobs with Size Threshold
- **Description:** Hybrid — images under 2MB stored as blobs in SQLite. Larger images stored in companion directory. Editor doesn't care which storage is used — the Rust backend abstracts it.
- **Pros:** Best of both worlds. Small icons/screenshots stay portable. Large images don't bloat the DB.
- **Cons:** More complex implementation. Two code paths to maintain.

## Chosen Approach

**Approach 2: Store Images as Blobs in SQLite.** For a worldbuilding tool, most images are character portraits, maps, and location art — typically under 5MB. Storing them inside the Tome database keeps everything in one portable file. SQLite handles this well, and the Tauri custom protocol handler serves them to the frontend efficiently.

If performance becomes an issue with very large images later, we can add the companion directory as an optimization in M10.

## Tasks

- [x] **1.** Add `tauri-plugin-dialog` and `tauri-plugin-fs` to Cargo.toml + Tauri config for file picker access.

- [x] **2.** Create `images` SQLite table in a new migration (`004_images.sql`): `id TEXT PRIMARY KEY, filename TEXT, mime_type TEXT, data BLOB, created_at TEXT`. Register migration.

- [x] **3.** Create `src-tauri/src/commands/images.rs` — Rust commands:
  - `upload_image(path: String) -> ImageInfo` — read file from disk, store blob in DB, return ID + filename
  - `get_image(id: String) -> ImageData` — return image blob + mime type
  - `delete_image(id: String)`
  - `list_images() -> Vec<ImageInfo>` — for an image picker/gallery

- [x] **4.** Register a custom Tauri protocol `vaelorium://` that serves images from the DB — allows `<img src="vaelorium://image/{id}">` in the editor HTML.

- [x] **5.** Update the editor "Img" toolbar button — replace `prompt()` with a dialog that offers both "Upload File" (opens native file picker) and "Paste URL" options.

- [x] **6.** Add image upload to bridge.ts mock — store images as data URLs in memory for browser testing.

- [x] **7.** Update the featured image in DetailsPanel — replace `prompt()` with the same file picker / URL dialog.

- [x] **8.** Add drag-and-drop image support in the editor — dropping an image file onto the editor uploads it and inserts it inline.

- [x] **9.** Write E2E tests for image upload flow.

- [x] **10.** Verify all existing tests pass.

## Notes

- The Tauri custom protocol handler (`tauri::protocol::asset`) can serve blobs from SQLite on demand.
- Image MIME types to accept: JPEG, PNG, GIF, WebP, SVG.
- For the browser mock, use FileReader to convert to data URLs.
- Clipboard paste support (Ctrl+V an image) is a nice-to-have but can wait.
- The `featured_image_path` field on pages currently stores a URL string. After this change, it could store either a URL or a `vaelorium://image/{id}` reference.
