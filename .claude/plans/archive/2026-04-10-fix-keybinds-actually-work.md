---
status: completed
---
# Fix: Keybinds Settings Actually Change Keyboard Shortcuts

**Date:** 2026-04-10

---

## Goal

The Settings page allows users to view and edit keybinds, but changing a keybind has no effect — the actual keyboard handler in `App.svelte` uses hardcoded key checks (`e.key === 'k'`, etc.) and never reads from the `settings` store. This plan makes the keybind settings functional end-to-end: editing a keybind in Settings immediately changes the keyboard shortcut behavior.

## Approaches Considered

### 1. Reactive Matcher in App.svelte
- **Description:** Replace the hardcoded `handleKeydown` in App.svelte with a function that reads keybinds from the `settings` store reactively. Parse each keybind string (e.g., "Ctrl+K") into a matcher that checks `e.ctrlKey`, `e.key`, etc.
- **Pros:** Simple, centralized. One handler, one place to maintain. Svelte store reactivity means changes take effect immediately.
- **Cons:** All shortcuts in one handler — could get complex with TipTap's own shortcuts.

### 2. Key Binding Manager Class
- **Description:** Create a `KeybindManager` class that registers action handlers and matches incoming keyboard events against the current keybind config. Each feature registers its action (search, new page, etc.) and the manager handles dispatch.
- **Pros:** Clean separation. Easy to add new keybinds. Testable as a unit.
- **Cons:** More abstraction than needed for 6 keybinds.

### 3. Per-Component Keybind Subscription
- **Description:** Each component that handles a shortcut subscribes to the settings store and sets up its own keybind matching. Search overlay watches for the search keybind, etc.
- **Pros:** Decentralized — each feature owns its shortcut.
- **Cons:** Scattered logic. Hard to prevent conflicts. Multiple keydown listeners.

## Chosen Approach

**Approach 1: Reactive Matcher in App.svelte.** 

It's the simplest correct approach for 6 keybinds. The `handleKeydown` function reads the current keybind config from the store, parses the key combo string, and matches against the event. Since the store is reactive, changes in Settings take effect immediately.

## Tasks
- [x] **1.** Create `parseKeybind` in `src/lib/utils/keybinds.ts`
- [x] **2.** Create `matchesKeybind` in `src/lib/utils/keybinds.ts`
- [x] **3.** App.svelte now reads keybinds from settings store via `matchesKeybind`
- [x] **4.** 14 unit tests for parseKeybind (6) and matchesKeybind (8), all passing
- [x] **5.** Playwright E2E: change search to Ctrl+J, verify Ctrl+K stops working, Ctrl+J works. Passing.
- [x] **6.** Playwright E2E: reset to defaults, verify Ctrl+K restored. Passing.

## Notes
- TipTap handles its own shortcuts (Bold=Ctrl+B, Italic=Ctrl+I) independently. Those are registered as TipTap extensions, not through our keybind system. The keybinds in settings for Bold/Italic are informational — they show what TipTap uses but can't currently override it. This is acceptable for MVP. If we want to override TipTap shortcuts later, we'd need to intercept before TipTap processes them.
- The keybind string format should be normalized: "Ctrl+K" (capital modifiers, plus separator, uppercase key for single chars).
- `metaKey` (Cmd on Mac) should be treated as equivalent to Ctrl for cross-platform support.
