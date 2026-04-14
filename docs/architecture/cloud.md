# Vaelorium Cloud (hosted backend)

## Responsibility
Hosted SaaS backup tier. Third `SyncBackend` implementation behind Vaelorium-issued auth; conforms to the frozen M7 protocol (bucket layout `tomes/<tome_uuid>/`, client-side encryption). The cloud repo itself is a sibling private repo at `~/Projects/vaelorium-cloud/`. **M5.1 shipped**, M5.2–5.4 pending.

## Entry points

### Rust commands (`src-tauri/src/commands/cloud.rs`)
- `cloud_signin` — email+password → device token in keychain; returns `CloudAccountInfo`.
- `cloud_signout` — revokes device token server-side + clears local.
- `cloud_status` — cached account info (no network).
- `cloud_account_refresh` — one-shot `GET /api/account`. **No polling** per cloud `efc9286`; refresh only on user-initiated surfaces.

### HostedBackend (`src-tauri/src/sync/backend/hosted/`)
- `hosted::HostedBackend` — implements `SyncBackend` against `cloud.vaelorium.com`.
- `hosted::crypto` — server-side auth hash derivation (separate key from content encryption).
- `hosted::client` — HTTP client with bearer device token.

### Frontend
- `src/lib/api/cloud.ts` — thin TS wrappers over the Rust commands; snake_case → camelCase conversion.
- `src/lib/stores/cloudStore.ts` — shared `cloudAccount` writable + `atTomeQuota` derived store. Seeded from `cloud_status` at app init, refreshed on signin / Settings open / CreateTomeModal open.
- `src/lib/components/BackupSetupWizard.svelte` — 5-step wizard (choose backend → hosted: signin → passphrase → device name → done).
- Settings → Backup tab (`Settings.svelte` Backup section — see file-section-map.md) — account display, sign-out, disconnect.

## Data flow (signin)
1. Wizard → `cloud_signin({email, password})` → cloud computes Argon2id(password, salt) auth hash → returns device token.
2. Rust stores device token in OS keychain; returns `CloudAccountInfo` (email, tier, usage).
3. Frontend seeds `cloudAccount` store directly from response (no second round-trip).
4. `configure_backup({kind: hosted, ...})` then writes `app_backend.json` with `BackendKind::Hosted`.

## Quota / Tome limits
- `CloudUsage.tomeCount` + `tomeLimit` (null = Author tier unlimited).
- `atTomeQuota` derived store returns true when `tomeCount >= tomeLimit` AND `backendKind === 'hosted'`.
- Settings.svelte uses a UUID-aware variant (`currentTomeAtQuota`): an already-synced UUID is treated as idempotent, not a new slot.
- Server returns 413 `tome_limit_exceeded` as the authoritative rejection; client check is UX only.

## Gotchas
- `cloudAccount` must be refreshed from the response on every signin (see `BackupSetupWizard.svelte` — `cloudAccount.set(info)`).
- Never poll `/api/account` on a timer — the cloud's no-polling contract is explicit.
- Device tokens are per-install; `cloud_signout` invalidates server-side so a subsequent signin issues a fresh token.
- Hosted backend doesn't use `PrefixedBackend` — prefixing happens server-side.

## Protocol invariants (frozen)
See `project_m7_sync` + `project_vaelorium_cloud` auto-memories. The cloud must conform to: bucket layout, encryption (client-side Argon2id + ChaCha20-Poly1305), CAS on sync-meta.json, snapshot+journal format. **Don't relitigate** in cloud sessions.

## See also
- `~/Projects/vaelorium-cloud/` — the cloud backend repo (private).
- `project_m5_cloud_integration` auto-memory.

## Where NOT to look
- `src-tauri/src/sync/backend/s3.rs` — BYOS3, not hosted.
- `src-tauri/src/sync/backend/filesystem.rs` — local-only.
