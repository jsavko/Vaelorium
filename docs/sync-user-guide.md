# Vaelorium Sync — User Guide

Vaelorium can back up your Tomes to your own storage and sync them across as many devices as you want. Everything is end-to-end encrypted with a passphrase only you know.

## What sync does

- **Backs up** every edit to a destination you control: a folder on your disk, or any S3-compatible bucket (AWS, Cloudflare R2, Backblaze B2, Wasabi, Minio, Garage).
- **Syncs edits across your devices**. Open the same Tome on a laptop and a desktop and work lives on both after a sync tick.
- **Preserves per-field divergence**. If two devices edit different fields of the same page while offline, they auto-merge. If both edit the *same* field, the conflict resolver asks you which to keep.
- **Restores Tomes** to a fresh device from the backup alone — no need to copy `.tome` files between machines.

## What sync does NOT do

- **Recover your passphrase.** There is no recovery mechanism. If you lose it, your backed-up Tomes are unreadable. Write it down.
- **Sync images or attachments.** Media blobs are local-only in v0.2.0.
- **Provide a hosted service.** All storage is yours — this is the BYO-backend tier. Vaelorium Cloud is a future, optional addition.

## First-time setup

1. Open Vaelorium and click **Set up a backup destination** on the Tome picker (or Settings → Backup → "Set up backup…").
2. **Step 1 — Welcome.** Read the overview.
3. **Step 2 — Backend choice:**
   - **Folder** — simplest. The folder receives immutable encrypted op/snapshot files only; your live `.tome` database stays local, so Syncthing / Dropbox / iCloud watching that folder is safe.
   - **S3-compatible bucket** — recommended for >2 devices or when you want a cloud provider's availability guarantees.
4. **Step 3 — Backend-specific config:**
   - Folder: browse to a local directory.
   - S3: endpoint (blank for AWS), region, bucket, access key, secret key, optional prefix.
5. **Step 4 — Passphrase.** 8+ chars. **There is no recovery.** Write it down now.
6. **Step 5 — Review + device name.** Device name can be anything; a 4-hex suffix is appended automatically so two devices named "My Laptop" stay distinguishable.
7. **Connect.** Vaelorium verifies the backend is reachable and your passphrase decrypts any existing data.

After setup, each Tome opts in individually: Settings → Sync → **Enable sync on this Tome**.

## Adding a second device

1. Install Vaelorium on the new machine.
2. Settings → Backup → "Set up backup…" — use the **same** backend details and the **same** passphrase as the first device.
3. On the Tome picker, open the **Restore from backup** section. Each Tome the backend has snapshots for shows its name, size, and snapshot age.
4. Click **Restore** on the one you want. Vaelorium downloads + decrypts the snapshot, writes it to your app-data directory, opens it, and **auto-enables sync** with this device's identity.
5. Future edits flow both ways automatically.

## Resolving conflicts

When two devices edit the same field of the same row while one of them was offline, Vaelorium can't auto-merge. You'll see a "N conflict" pill in the sidebar.

**Click the pill** to open the conflicts modal. Each conflict shows:
- The table and row label (e.g. "Page · Sir Derlic the Bold", "Entity field value · My NPC · Class")
- The field name
- Both sides' values side-by-side

Pick which to keep per field, or use **Keep all local** / **Keep all remote** for bulk resolution. Apply propagates the choice as a new op — once both devices sync again, the conflict clears on both sides.

## Activity log

Settings → Sync → **Recent activity** (collapsible) shows the last 100 sync runs for this Tome: when, duration, op counts, conflict counts, and error messages on failures. Use it to answer "did my edits sync?" without digging in application logs.

## Snapshot cadence

Vaelorium takes an encrypted snapshot when any of:

- ~5 MB of journal entries accumulate
- 5,000 ops accumulate
- Weekly
- You click **Take snapshot** in Settings → Sync

Snapshots let a fresh device start from a compact baseline instead of replaying every op ever made. Old journal entries are pruned locally and remotely after a snapshot.

## Retries and offline

The background sync runner retries transient network errors (IO, timeout) up to three times with 1 s / 4 s / 16 s spacing before surfacing "Sync error" on the sidebar pill. It does NOT retry authentication failures (those need you to fix credentials) or protocol-level CAS mismatches.

User-clicked **Sync now** skips the retry loop — click it again if you want an immediate retry.

## Renaming or removing a device

- **Rename:** Settings → Backup → Device name field. The 4-hex suffix stays consistent with your device_id.
- **Stop syncing this Tome only:** Settings → Sync → "Stop syncing this Tome".
- **Disconnect backup entirely:** Settings → Backup → Disconnect. Removes local backup config + passphrase from the OS keychain. The bucket is untouched.

## Troubleshooting

- **"passphrase does not match the existing backup data"** — the passphrase you entered doesn't decrypt what's on the backend. Either the passphrase is wrong, or this bucket already contains data from a different passphrase.
- **"backup is locked"** — the app restarted and your passphrase isn't cached. Click the "Sync locked" pill or go to Settings → Backup to re-enter it. On platforms with a working OS keychain (macOS, Windows, Linux with gnome-keyring/kwallet), Vaelorium remembers the passphrase between sessions; on other platforms you unlock once per launch.
- **Syncthing `.sync-conflict-*` files appear in the backup folder** — rare; happens if two devices upload the same snapshot pointer at exactly the same moment. Delete the `.sync-conflict-*` sibling; the engine will re-upload on the next tick.
- **Sync activity shows "error" but Sync now works** — probably a transient network blip. The runner will retry on the next tick.

## Using a NAS or network share

The **Folder** backend accepts any path the OS can write to — local disk, a Syncthing/Dropbox folder, or a mounted NAS share. You don't need a separate backend for a NAS.

**Example paths by OS:**

- **Windows:** `\\UNRAID\vaelorium` (UNC) or `Z:\vaelorium` (mapped drive)
- **macOS:** `/Volumes/vaelorium` (after mounting the share in Finder)
- **Linux:** `/mnt/nas/vaelorium` (after an `fstab` or `gio mount`)

**Credentials** are managed by the OS, not Vaelorium — use your NAS's user account with the platform's usual mount flow (Windows credential manager, macOS Finder "Connect to Server…", Linux `fstab` or `gio`).

**Caveat:** the OS must keep the mount active while Vaelorium runs. A disconnected share surfaces as an IO error in the sync activity log; the runner retries transiently (1 s / 4 s / 16 s backoff) before surfacing the failure, so a brief network hiccup is usually invisible.

Why this works: the backend writes immutable encrypted op/snapshot files (atomic tempfile + rename) and never rewrites in place — exactly the workload a network filesystem handles well. The live `.tome` SQLite database stays local on each device, never on the share.

## Advanced

- **S3-compatible setup recipes:** see [sync-s3-testing.md](./sync-s3-testing.md) for a Minio-in-Docker recipe and notes on AWS / Cloudflare R2 / Backblaze B2.
- **Manual multi-device test plan:** see [sync-manual-testing.md](./sync-manual-testing.md).
