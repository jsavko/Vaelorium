# UI / theming

## Responsibility
Design tokens, theme palettes, and shared UI primitives (confirm dialog, toast). Centralizes the "arcane library" aesthetic so components never hardcode colors.

## Entry points

### Tokens
- `src/app.css` — all CSS custom properties live here, in `@theme { ... }`. Surfaces, foreground, accent (gold), borders, entity-type colors, status. Dark walnut palette is the default.
- `src/lib/stores/settingsStore.ts` — `appearance` config including theme selection.

### Primitives
- `src/lib/components/ConfirmDialog.svelte` — **always use this** instead of `confirm()` (`feedback_no_native_dialogs`).
- `src/lib/stores/toastStore.ts` — `showToast(msg, kind)`. Positioned top-center, z-index 9999 (`feedback_toast_position`).
- `src/lib/components/ToastContainer.svelte` — renders the queue.

### Theme switching
- Multiple themes planned; dark cozy is default. Never hardcode colors — use `var(--color-*)` from app.css.
- Theme preference persisted via `settingsStore`.

## Conventions
- **Scrim close on modals:** gate on `e.target === e.currentTarget`, not `stopPropagation()` — drag-aware clicks bypass inner stop handlers (`feedback_modal_scrim_close`).
- **Skip toasts for routine actions** (save, etc.). Reserve for user-actionable state changes.
- **No native dialogs** — `confirm()` / `alert()` don't integrate with Tauri WebKitGTK, break scrim semantics, and look out of place.

## Typography
- `--font-heading` — serif, used for titles.
- `--font-ui` — sans, UI chrome.
- `--font-body` — body text in page content.

## Gotchas
- Pencil MCP workflow for mockups uses literal font names (`feedback_pencil_workflow`).
- In WSLg, bottom-positioned toasts are invisible — that's why the global position is top-center.

## See also
- `project_design_system` + `feedback_dark_theme` auto-memories.
- `project_branding` — logo, icon, color anchor points.

## Where NOT to look
- No global "theme" file beyond `app.css`. If you're hunting per-component styling, it's inline in each `.svelte` file.
