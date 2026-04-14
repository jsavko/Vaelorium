# Infer first-device vs existing-setup from cloud sign-in

**Date:** 2026-04-14

## Summary

`BackupSetupWizard.svelte` step 4 no longer makes the user pick between "First device" and "Adding to existing" when the answer is derivable from the data we already have. For **Vaelorium Cloud**, sign-in already returns `CloudAccountInfo.usage.tomeCount` + `bytesUsed` — that's authoritative. For **filesystem / S3** we now probe the bucket at step-3 completion. The radio is hidden when inference is confident; a `Change` disclosure re-exposes it for edge cases (e.g. intentional passphrase rotation).

User quote:
> "Shouldn't logging in return if I am first time setup or accessing a cloud with data in it already?"

Yes — and now it does.

## Changes

### Features
- **Hosted inference** from sign-in response: step 4 renders either "Create an encryption passphrase" (empty account) or "Enter your encryption passphrase" (account has data) with no user choice required.
- **FS/S3 inference** via a new Rust command `backup_probe_bucket_has_data(backend_kind, backend_config)` that lists `tomes/` on a raw backend and checks for any `.snap.enc` or `.op.enc` object. Runs at step 3→4 transition. Probe errors surface as a clear step-3 error rather than letting the user proceed blindly.
- **Manual override** preserved as a `Change` inline link under the inferred header — flips back to the legacy radio + both forms.

### Files Modified
- `src-tauri/src/commands/backup/config.rs` — new `backup_probe_bucket_has_data` command.
- `src-tauri/src/lib.rs` — registered the new command via `commands::backup::config::backup_probe_bucket_has_data`.
- `src/lib/api/backup.ts` — `probeBucketHasData` TS wrapper.
- `src/lib/components/BackupSetupWizard.svelte` — `inferredIntent` + `showIntentOverride` state, probe on step 3→4, step 4 template now branches on inference with a manual-override escape hatch.
- `docs/architecture/cloud.md` — signin data-flow mentions usage-driven inference.
- `docs/architecture/backup.md` — config.rs bullet includes the new probe command.

### Verification
- `npm run check` — 3,954 files clean.
- `cargo test --lib` — 84 passed.
- `scripts/check-architecture-docs.sh` — clean.
- Runtime verification (walk the wizard against an empty + non-empty account) is on the user; Tauri WebKitGTK + WSL can't drive automated browser tests.

### Rationale

The wizard was asking the user to mirror a fact the app already knew, inviting mis-selection ("pick First device on an account that already has data" → passphrase mismatch → opaque first-sync decryption failure). Making sign-in authoritative removes a decision + a foot-gun. The manual override preserves the escape hatch for confident users with edge-case needs.
