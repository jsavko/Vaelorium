# Image Text Wrapping + Clipboard Paste

**Date:** 2026-04-11

## Summary

Images in the wiki editor can now float left or right with text wrapping around them. Clicking an image shows a floating toolbar with alignment controls. Images can also be pasted directly from the clipboard via Ctrl+V.

## Changes

### Features
- `FloatImage` TipTap extension with `alignment` attribute (center/left/right)
- CSS float-based text wrapping for left/right aligned images
- Floating image toolbar on selection with alignment buttons + delete
- `ImagePastePlugin` TipTap extension for clipboard image paste
- Images max-width 50% when floated, 100% when centered
- Gold outline on selected images

### Files Modified
- `src/lib/editor/FloatImage.ts` — new custom image extension
- `src/lib/editor/ImagePastePlugin.ts` — new clipboard paste plugin
- `src/lib/editor/EditorConfig.ts` — replaced Image with FloatImage + ImagePastePlugin
- `src/lib/components/Editor.svelte` — image toolbar UI, alignment CSS, container wrapper
- `package.json` — @tiptap/extension-bubble-menu

## Notes
- Image resize handles (task 6) deferred — can be added later with a custom NodeView.
