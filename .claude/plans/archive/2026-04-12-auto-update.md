---
status: completed
---
# Auto-Update Mechanism

**Date:** 2026-04-12

---

## Goal

Add self-update capability to the desktop app so users automatically get new versions without manually downloading installers. Check for updates on launch, notify the user, and install the update with a restart.

## Approaches Considered

### 1. Tauri Updater Plugin (`tauri-plugin-updater`)
- **Description:** Tauri's official updater plugin. Checks a signed JSON endpoint for new versions, downloads the platform-specific bundle, verifies the signature, and installs/restarts.
- **Pros:** First-party support. Signed updates (security). Cross-platform (.msi/.dmg/.AppImage). Integrates with our existing GitHub Releases. Small code footprint. Battle-tested.
- **Cons:** Requires signing keys to be set up. Endpoint hosting (can use GitHub Releases). Initial setup complexity.

### 2. Electron-style auto-updater from scratch
- **Description:** Roll our own: check GitHub Releases API, download installer, prompt user to run it.
- **Pros:** No signing needed (just hash verification). Full control.
- **Cons:** Massive amount of code. Security risks without code signing. Platform-specific install logic. Reinventing the wheel.

### 3. In-app update notification only
- **Description:** App checks version on launch, shows a "new version available" notification with a link to GitHub Releases. User manually downloads and installs.
- **Pros:** Zero security concerns. Trivial to implement. No signing setup.
- **Cons:** Not automatic — user has to download and install manually. Friction.

### 4. Hybrid: Notification now, full updater later
- **Description:** Ship notification-only approach immediately (Approach 3), add full Tauri updater (Approach 1) as a follow-up when signing keys are set up.
- **Pros:** Immediate value. Low risk. Clear upgrade path.
- **Cons:** Requires two iterations.

## Chosen Approach

**Approach 4: Hybrid — Notification now, full updater later.** Start with a simple version check against GitHub Releases API that notifies the user of available updates. Users click through to download. Later, once signing keys are generated and configured, migrate to Tauri's updater plugin for automatic install.

This gets immediate value without the upfront complexity of signing key management.

## Tasks

### Phase 1: Version Check Notification

**Superseded — went directly to Phase 2 (full Tauri updater) since signing infra was available. The `UpdateNotification.svelte` component handles both notification and auto-install in one UI.**

- [~] **1.** Add Tauri HTTP plugin (`tauri-plugin-http`) for making GitHub API requests.

- [ ] **2.** Create `src/lib/api/updates.ts` — TypeScript function that calls GitHub Releases API to check for the latest release, compares to current version, returns update info if newer.

- [ ] **3.** Build `UpdateNotification.svelte` — small banner that appears when an update is available, with "Download" button linking to the release URL and "Dismiss" button.

- [ ] **4.** Wire update check to run on app launch (after Tome is open). Cache result for 24 hours to avoid repeated API calls.

- [ ] **5.** Add "Check for updates" button in Settings > About (or similar section).

### Phase 2: Tauri Updater Plugin (Future)

- [x] **6.** Generate signing keys (tauri signer generate).

- [x] **7.** Add `tauri-plugin-updater` to Cargo.toml and capabilities.

- [x] **8.** Configure updater endpoint in tauri.conf.json pointing to GitHub Releases `latest.json`.

- [x] **9.** Generate `latest.json` in CI on tagged releases (GitHub Actions workflow).

- [x] **10.** Add updater signing secret to GitHub Actions.

- [x] **11.** Replace notification with auto-install UI (download progress, restart prompt).

### Testing

- [x] **12.** Test update component end-to-end (component built, tests pass, awaiting real release to validate pipeline).

- [x] **13.** Verify all existing tests pass. (44 tests passing)

## Notes

- **Version comparison:** Use simple semver comparison (compare major.minor.patch).
- **Current version:** Read from `app.getVersion()` in Tauri (v2 API).
- **GitHub API endpoint:** `https://api.github.com/repos/jsavko/Vaelorium/releases/latest`
- **Dismissal:** Remember user's dismissal for the same version via localStorage.
- **For Phase 2:** Signing keys must be kept secure — use GitHub Actions secrets.
- **Browser mock:** Update notification should be hidden in browser mode (not applicable).
