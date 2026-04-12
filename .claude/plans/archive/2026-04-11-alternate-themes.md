---
status: completed
---
# Alternate Themes

**Date:** 2026-04-11

---

## Goal

Create 8 alternate themes (mix of dark and light) beyond the default "Dark Library" theme. Themes should appeal to a broad range of users — from cozy dark to clean modern to high-contrast.

## Approaches Considered

### 1. CSS Custom Properties Override via data-theme attribute
- **Description:** Define each theme as a set of CSS custom property overrides in `app.css` scoped to `[data-theme="theme-name"]`. The settings store applies the attribute to `<html>`.
- **Pros:** Simple, no JS runtime cost. Works with existing token system. Easy to add more themes.
- **Cons:** All theme CSS loaded upfront (small cost for 9 themes).

### 2. Dynamic CSS injection
- **Description:** Store theme values in JS and inject them as inline styles on `:root`.
- **Pros:** More flexible. Could allow user-customized themes.
- **Cons:** More complex. Harder to debug. Flash of unstyled content.

### 3. Separate CSS files per theme
- **Description:** Each theme is a separate CSS file, loaded on demand.
- **Pros:** Clean separation. Only active theme loaded.
- **Cons:** Async loading causes flash. More build complexity.

## Chosen Approach

**Approach 1: CSS Custom Properties Override.** Define themes as `[data-theme]` selectors in app.css. Settings store sets `document.documentElement.dataset.theme`. Simple, performant, and works with the existing design system.

## Tasks

- [x] **1.** Define 8 theme palettes in `app.css` as `[data-theme="..."]` overrides.

- [x] **2.** Update `settingsStore.ts` to apply `data-theme` attribute on the document element when theme changes.

- [x] **3.** Update Settings.svelte theme list with all 9 themes (including default).

- [x] **4.** Add theme preview swatches in the settings UI.

- [x] **5.** Verify all existing tests pass.

## Themes

1. **Dark Library** (default) — Warm walnut browns, gold accents. Candlelit scriptorium.
2. **Midnight Ink** — Deep blue-black with silver/ice-blue accents. Starlit study.
3. **Obsidian** — Pure dark with neon green accents. Hacker/cyberpunk vibe.
4. **Ember Hearth** — Dark charcoal with warm orange/red accents. Fireside warmth.
5. **Moonstone** — Light mode! Cream/off-white with slate blue accents. Clean and airy.
6. **Parchment** — Light mode. Warm ivory with sepia/brown accents. Old manuscript.
7. **Dusk** — Medium dark purple-gray with lavender accents. Twilight atmosphere.
8. **Forest** — Dark green-black with emerald/moss accents. Deep woods.
9. **Storm** — Dark slate gray with electric blue accents. Moody and modern.

## Notes

- Entity type colors should remain consistent across themes for recognition.
- Accent color changes per theme (gold → blue → green → etc.).
- Light themes need color-scheme: light override and inverted foreground/background.
- The `color-scheme: dark` on html/body needs to flip for light themes.
