# Account Settings Tab with Version & Update Check

**Date:** 2026-04-12

## Summary
Added a new "Account" tab to the Settings modal showing the app version and a manual "Check for Updates" button that drives the Tauri self-updater. The existing 6-hour passive update banner remains unchanged.

## Changes

### Features
- **Account tab in Settings modal** — displays app name and version (via `@tauri-apps/api/app` `getVersion()` in Tauri, "web" in browser build).
- **Manual update check** — "Check for Updates" button invokes `check()` from `@tauri-apps/plugin-updater`, rendering inline status: checking / up-to-date / available (with version + release notes) / error.
- **In-tab install flow** — when an update is available, an Install button triggers a `ConfirmDialog`, then `downloadAndInstall` with progress percentage and `relaunch` on completion.
- **LocalStorage sync** — a successful manual check updates `vaelorium-last-update-check` and clears `vaelorium-dismissed-version`, so the passive banner and manual tab stay coherent.

### Tests
- Added Playwright test `account tab shows version and update controls` verifying the tab renders version info and the non-Tauri updates notice.

## Files Modified
- `src/lib/components/Settings.svelte` — new Account tab, state, handlers, styles.
- `e2e/settings.spec.ts` — new test for the Account tab.
- `docs/changelog/2026-04-12-account-settings-update-check.md` — this file.

## Rationale
Users previously had no surface to inspect their installed version or trigger an update on demand; they had to wait for the 6-hour passive check. This adds a discoverable, in-app path without disturbing the existing banner flow.
