# Auto-Update Mechanism

**Date:** 2026-04-12

## Summary

Added self-update capability to the desktop app using the Tauri updater plugin with minisign-signed releases. Users now see an in-app banner when a new version is published, can install it with one click, and the app restarts automatically.

## Changes

### Features
- **Auto-update banner** — `UpdateNotification.svelte` checks for updates on launch (throttled to once per 6 hours), shows a subtle banner, and handles download progress, install, and relaunch.
- **Signed updates** — Tauri updater plugin verifies minisign signatures against a bundled public key before installing, preventing tampered updates.
- **CI signing pipeline** — GitHub Actions build now signs artifacts with `TAURI_SIGNING_PRIVATE_KEY` and publishes a `latest.json` manifest alongside each tagged release.

### Files Modified
- `src-tauri/Cargo.toml` — added `tauri-plugin-updater = "2"`.
- `src-tauri/src/lib.rs` — registered updater plugin.
- `src-tauri/tauri.conf.json` — enabled `createUpdaterArtifacts`, added `plugins.updater` with pubkey + GitHub Releases endpoint.
- `src-tauri/capabilities/default.json` — granted `updater:default`.
- `package.json` — added `@tauri-apps/plugin-updater` and `@tauri-apps/plugin-process`.
- `src/lib/components/UpdateNotification.svelte` — new component (banner + progress + install flow).
- `src/App.svelte` — wired `<UpdateNotification />` into the layout (visible when a Tome is open).
- `.github/workflows/build.yml` — passed signing env vars to `tauri-action`, expanded artifact upload globs to include signature/zip files, added a step that composes `latest.json` from the platform signatures and attaches it to the release.

### Infrastructure
- Generated Tauri signing keypair at `~/.tauri/vaelorium.key` (no passphrase).
- Set GitHub secrets `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.

## Rationale

Shipped full auto-update (original plan's Phase 2) directly instead of the notification-only Phase 1, since signing keys were trivial to generate and the gh CLI was already set up. The single-step delivery avoids a throwaway implementation and gives users a native update experience from the first release.
