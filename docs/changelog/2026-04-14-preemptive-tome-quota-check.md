# Preemptive Tome-limit check for Vaelorium Cloud users

**Date:** 2026-04-14

## Summary

Surfaces a warning banner in the Create-New-Tome modal and the Settings → Sync "Back up this Tome" action when a hosted-cloud user is already at their plan's Tome limit. Previously the app let the user complete creation, then sync failed minutes later with a 413 `tome_limit_exceeded` — the fix hoists that signal to the moment the user commits.

## Changes

### Features
- **CreateTomeModal** — Warning banner when `backendKind === 'hosted'` and cached cloud usage reports the user is at quota. Create button stays enabled (offline-first: creating a local Tome is never blocked on cloud state).
- **Settings → Sync** — Same banner + hard-disabled **Back up this Tome** button when over quota, preventing individual 413s after the modal is dismissed.
- **Cloud account store** — New `src/lib/stores/cloudStore.ts` lifts `cloudAccount` out of the Settings component into a shared writable + derived `atTomeQuota`. Populated on app init (post-Tome-open) and seeded from the `cloudSignin` response in the setup wizard.

### Files Modified
- `src/lib/stores/cloudStore.ts` (new) — shared `cloudAccount`, `refreshCloudAccount`, `atTomeQuota`.
- `src/lib/components/CreateTomeModal.svelte` — quota banner + `refreshCloudAccount()` on open.
- `src/lib/components/Settings.svelte` — drops local `cloudAccount` state, subscribes to the shared store, adds banner + disabled state on the enable-sync CTA.
- `src/lib/components/BackupSetupWizard.svelte` — seeds `cloudAccount` store from the sign-in response so the first Create after signin shows correct quota state.
- `src/App.svelte` — wires `refreshCloudAccount()` into the `$isTomeOpen` init path next to `initSyncStore()`.

### Rationale

Offline-first: creating a local Tome should never be gated on cloud state. Users choosing Vaelorium Cloud still deserve to know *before* they commit that the new file won't back up — so they can make an informed trade-off (upgrade, free a slot, or accept local-only). Server-side 413 enforcement stays authoritative; this is purely a pre-flight UX surface.
