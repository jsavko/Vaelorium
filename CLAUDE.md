# Vaelorium

Self-hosted, offline-first LegendKeeper alternative for worldbuilders. Cross-platform desktop app (Tauri + Svelte 5 + Rust) with optional sync and a web build.

- **Domain:** vaelorium.com
- **Theme:** arcane library; dark walnut palette is the default
- **Tome file extension:** `.tome`
- **Status:** v0.1.1 released; milestones M0–M6, M9, M10 complete; M7 + M8 remaining

## Stack

- **Frontend:** Svelte 5 (runes: `$state`, `$props`, `$derived`), TypeScript, Vite
- **Backend:** Rust, Tauri v2, SQLite
- **Testing:** Vitest (unit), Playwright (e2e)
- **CI/CD:** GitHub Actions builds Windows/macOS/Linux; Tauri auto-updater with signed releases

## Project Conventions

- **Tauri `invoke()` args must be camelCase** — snake_case silently fails
- **No native dialogs** — never use `confirm()`/`alert()`; use `ConfirmDialog.svelte`. Skip toasts for routine actions
- **Theming** — never hardcode colors; use CSS custom properties from the design tokens. Dark cozy is default, multiple themes supported
- **Testing workflow** — always runtime-test via browser (Tauri WebKitGTK on WSL can't connect to Chrome DevTools MCP). Compile success ≠ feature works
- **Task completion bar** — a task is not done until it's fully functional AND has passing unit + Playwright tests
- **Scope discipline** — don't add features, refactor, or "improve" beyond what was asked; don't add comments/docstrings to code you didn't change

## Key Paths

- `src/lib/components/` — Svelte components
- `src/lib/stores/` — Svelte stores
- `src/lib/api/` — Tauri bridge + API wrappers
- `src-tauri/src/` — Rust backend
- `src-tauri/tauri.conf.json` — version, updater config
- `tests/` — Playwright e2e

## Updater

- Signing keys: `~/.tauri/vaelorium.key`
- GitHub release workflow publishes `latest.json` endpoint
- Runtime check in `src/lib/components/UpdateNotification.svelte` (6-hour passive auto-check)

## System Dependencies

- `fonts-noto-color-emoji` required for emoji rendering
