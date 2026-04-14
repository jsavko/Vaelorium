# Backup destination

## Responsibility
Backend-agnostic storage layer the sync engine writes to. Three implementations (filesystem, S3, hosted cloud) behind a `SyncBackend` trait. Also owns snapshot creation, listing, restore, and delete, plus the encryption boundary between plaintext SQLite state and on-wire ciphertext.

## Entry points

### Rust backend (`src-tauri/src/sync/backend/`)
- `backend::mod::SyncBackend` trait — the abstraction. `put`, `get`, `list_prefix`, `delete_prefix`, `cas_update`.
- `backend::filesystem::FilesystemBackend` — local folder or Syncthing share.
- `backend::s3::S3Backend` — `aws-sdk-s3`-based; talks to AWS, R2, Minio, B2, Wasabi, Garage.
- `backend::hosted::*` — Vaelorium Cloud client (see cloud.md).
- `backend::prefixed::PrefixedBackend` — wraps a raw backend with `tomes/<uuid>/` prefix. Applied to filesystem + S3; hosted does prefixing server-side.

### Commands (`src-tauri/src/commands/backup.rs` — 986 lines, see file-section-map.md)
- `configure_backup` / `disconnect_backup` — app-global backup config (`app_backend.json` + keychain).
- `unlock_backup` / `try_auto_unlock_backup` — passphrase → key material unlock.
- `list_restorable_tomes` / `list_hosted_restorable_tomes` — enumerate `tomes/<uuid>/snapshots/*.snap.enc`, decrypt peek at `tome_metadata.name`.
- `restore_tome_from_backup` — pulls latest snapshot, decrypts, writes `.tome` file, registers in recent_tomes, seeds `sync_config.enabled=true`.
- `backup_delete_tome(tome_uuid)` — hosted: `DELETE /v1/tomes/<uuid>`; filesystem/S3: `list_prefix + delete`. Called from TomePicker trash button.

### App-global state
- `sync::app_backend::{load, save}` (`src-tauri/src/sync/app_backend.rs`) — serializes `AppBackupConfig` to disk.
- `sync::keychain::*` (`src-tauri/src/sync/keychain.rs`) — passphrase-derived key storage (OS keychain when available).
- `sync::session::SessionState` (`src-tauri/src/sync/session.rs`) — in-memory unlocked key; gates every sync operation.

### Snapshots
- `sync::snapshot::take_snapshot` (`src-tauri/src/sync/snapshot.rs`) — gzip-encrypted SQLite dump.
- `sync::snapshot::list_tome_snapshots` — backend listing + metadata peek (filesystem/S3 variant).
- `sync::snapshot::apply_snapshot` — on restore.

## Encryption boundary
- `sync::crypto::KeyMaterial` + `derive_key` — Argon2id KDF from passphrase + device salt.
- ChaCha20-Poly1305 AEAD per-chunk; nonce per-op. Keys NEVER leave the device.
- Backends only see ciphertext. The hosted backend enforces its own auth layer on top.

## Data flow (restore)
1. `list_restorable_tomes` → frontend renders cards in TomePicker.
2. User clicks restore → `restore_tome_from_backup(tome_uuid)` downloads snapshot, decrypts, writes to app-data `.tome` file.
3. Handler seeds `sync_config` with `enabled=true`, registers in `recent_tomes` with `sync_enabled=true`.

## Gotchas
- Filesystem backend uses OS mtime for CAS-ish checks; Syncthing propagation delay is accepted.
- S3 backend has unit tests but no automated integration tests (`docs/sync-s3-testing.md` covers manual).
- Restore does NOT re-run migrations (snapshot is already schema-complete).
- Snapshot metadata peek opens a tiny in-memory sqlite just to read `tome_metadata.name`; any extra columns read here mean extra decrypt cost per restore-list entry.

## Where NOT to look
- `src/lib/api/backup.ts` is a thin wrapper — actual logic is Rust-side.
- `commands/export.rs` handles JSON/Markdown export; unrelated to encrypted backups.
