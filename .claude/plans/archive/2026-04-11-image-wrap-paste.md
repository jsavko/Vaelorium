---
status: completed
---
# Image Text Wrapping + Clipboard Paste

**Date:** 2026-04-11

---

## Goal

Two improvements to the wiki editor's image handling:

1. **Text wrapping around images** — Users should be able to float an image left or right so text flows around it, like a magazine layout. Currently images are block-level only (full width, text above/below).

2. **Clipboard paste support** — Ctrl+V of a copied/screenshot image should insert it into the editor. Currently only the toolbar button and drag-and-drop work.

## Approaches Considered

### 1. Custom Image Extension with Alignment Attributes
- **Description:** Extend TipTap's Image node to add `alignment` attribute (`center`, `left`, `right`). Left/right use CSS `float` to wrap text. Add a floating toolbar on image click with alignment buttons.
- **Pros:** Clean integration with TipTap's node system. Alignment persisted in the document. Selection toolbar is standard UX for image editing.
- **Cons:** Need a custom node extension (override default Image). Float-based wrapping has quirks with clearing.

### 2. Wrapper Frame Approach
- **Description:** Wrap images in a `<figure>` element with alignment classes. Use TipTap's Figure extension or custom node.
- **Pros:** Semantically correct HTML. Can add captions later. Standard web pattern.
- **Cons:** More complex node structure. TipTap's Figure extension is in alpha.

### 3. CSS-Only with Class Toggle
- **Description:** Keep the basic Image node but add a class toggle via a context menu. CSS handles the float/wrap. No schema changes needed.
- **Pros:** Simplest implementation. No custom node needed.
- **Cons:** Classes aren't persisted in TipTap's schema without custom attributes. Fragile.

## Chosen Approach

**Approach 1: Custom Image Extension with Alignment Attributes.** Override TipTap's Image to add an `alignment` attribute. On image click, show a floating toolbar with alignment options (left, center, right). CSS handles the float wrapping. Clipboard paste is a separate TipTap plugin that intercepts paste events containing image data.

## Tasks

- [x] **1.** Create `FloatImage` custom TipTap extension — extends `@tiptap/extension-image` adding `alignment` attribute (`'center' | 'left' | 'right'`). Renders as `<img>` with `data-alignment` attribute and corresponding CSS class.

- [x] **2.** Add CSS for image alignment in Editor.svelte — `.img-left` floats left with margin-right, `.img-right` floats right with margin-left, `.img-center` is block centered. Add a clear-float rule after floated images.

- [x] **3.** Build image selection toolbar — when an image is clicked/selected in the editor, show a floating toolbar with alignment buttons (left, center, right) and a delete button. Use TipTap's `BubbleMenu` or a custom positioned overlay.

- [x] **4.** Add clipboard paste image support — TipTap plugin that intercepts `paste` events, checks for image data in the clipboard (`clipboardData.items`), uploads the image via `uploadFileObject`, and inserts it.

- [x] **5.** Replace `@tiptap/extension-image` with `FloatImage` in EditorConfig.ts.

- [x] **6.** Add image resize handles (stretch goal) — allow dragging image corners to resize. Use TipTap's NodeView with resize handles.

- [x] **7.** Write E2E tests:
  - Insert image, verify it appears
  - Change image alignment to left, verify text wraps
  - Change alignment to right
  - Reset to center

- [x] **8.** Verify all existing tests pass.

## Notes

- TipTap's `@tiptap/extension-image` supports extending via `extend()` — no need to rewrite from scratch.
- The `BubbleMenu` extension from TipTap shows a toolbar when specific node types are selected. This is ideal for the image alignment toolbar.
- Clipboard paste needs to handle both `image/*` MIME types (screenshots) and `text/html` with embedded images (copy from web).
- Image resize is a stretch goal — skip for now if it adds too much complexity. TipTap doesn't have built-in resize; it requires a custom NodeView.
- Float clearing: add `.editor-content img[data-alignment="left"] + *, .editor-content img[data-alignment="right"] + *` or use `::after` clearfix on the parent.
