# Sync — Manual Test Plan

A practical checklist for validating M7 sync end-to-end before tagging v0.2.0.
Written for Phase 4b-S3-shipped state. Some items defer to Phase 5 — those are
called out in §4.

Throughout: **PASS / FAIL / SKIP** in the box; jot the date + version next to
your initials when you sign off.

---

## 0. Pre-flight

- [ ] Latest code built & app launched: `cd /home/james/Projects/kankaClone && npm run tauri:dev`
- [ ] One Tome open (create or open from TomePicker)
- [ ] Sync backend reachable (Backblaze B2 / R2 / Minio / local folder — your choice)

> **Upgrade note (post Phase 5.0):** the bucket prefix for each Tome is now derived
> from a per-Tome UUID stored in `tome_metadata`, not from the local file path.
> Previously synced buckets still contain data under the old path-derived prefix
> (`tomes/<old-hash>/`); those prefixes become orphaned after this upgrade.
> Delete the old `tomes/*` entries from your bucket and re-enable sync — a new
> `tomes/<uuid>/` prefix is created on first upload.

For a B2-style smoke setup:
- Endpoint: `https://s3.us-west-000.backblazeb2.com`
- Region: `us-west-000`
- Bucket: a fresh empty bucket dedicated to testing
- Access key + secret from B2 Application Keys (scope to that bucket only)

---

## 1. Single-device flow

### 1.1 Enable sync
- [ ] Settings → Sync → "Enable sync"
- [ ] Choose backend (Filesystem or S3); fill in fields
- [ ] Set passphrase (≥8 chars, matched)
- [ ] Click "Enable sync for this Tome"
- [ ] **Expect:** status flips to enabled state. Sidebar pill appears (green ●).
- [ ] **Expect:** within ~10 s, pill flashes "Syncing…" then back to "Synced"
- [ ] Inspect bucket — see `<prefix>/snapshots/` and `<prefix>/journal/` populating

### 1.2 Edit drives sync
- [ ] Edit a page title
- [ ] **Expect:** sidebar pill flashes "Syncing…" within ~10 s of the edit (debounced)
- [ ] **Expect:** bucket gains a new `journal/<ulid>.op.enc` (~500 B – 1 KB)
- [ ] **Expect:** Settings → Sync status card "Pending uploads" returns to 0 after sync

### 1.3 Manual sync now
- [ ] Settings → Sync → "Sync now"
- [ ] **Expect:** toast notification (top-center): `Synced — N up, M down`
- [ ] **Expect:** Last sync timestamp updates

### 1.4 Manual snapshot
- [ ] Settings → Sync → "Take snapshot"
- [ ] **Expect:** toast: "Snapshot taken"
- [ ] **Expect:** new `snapshots/<ulid>.snap.enc` in bucket
- [ ] **Verify size:** for a small Tome, snapshot should be ~10–20 KB (gzip+encrypted). For a 5000-page Tome, expect ~100–500 KB. If unexpectedly large (>5 MB on small Tomes), file an issue — compression may not be applied.

### 1.5 Disable sync
- [ ] Settings → Sync → "Disable sync"
- [ ] **Expect:** status returns to off; sidebar pill disappears
- [ ] **Expect:** bucket contents are NOT deleted (we don't touch remote on disable)

---

## 2. Multi-device flow

Requires either:
- Two physical machines on the same backend, or
- One machine with two separate Vaelorium installs / Tome paths pointing at the same bucket

### 2.1 Sequential edits
- [ ] Device A: enable sync, create page "Aelinor", sync, close
- [ ] Device B: open empty Tome, enable sync (same backend + passphrase), sync
- [ ] **Expect:** "Aelinor" appears on B
- [ ] B: edit title to "Aelinor of Westmarch", sync
- [ ] A: re-open Tome, sync
- [ ] **Expect:** title shows "Aelinor of Westmarch" on A

### 2.2 Concurrent edits — disjoint fields auto-merge
- [ ] Both devices on same page
- [ ] A: edit title (don't sync yet)
- [ ] B: edit icon (don't sync yet)
- [ ] A: sync
- [ ] B: sync
- [ ] **Expect:** both sides converge — A's title + B's icon both present, **no conflict banner**, sidebar stays green

### 2.3 Concurrent edits — overlapping field conflict
- [ ] A: edit page title to "Laptop title" (don't sync)
- [ ] B: edit same page title to "Desktop title" (don't sync)
- [ ] A: sync
- [ ] B: sync
- [ ] **Expect:** ConflictResolver banner appears above page editor on B
  - Two side-by-side cards: "This device" / "Other device" with both values
  - Sidebar pill shows "1 conflict" (yellow ⚠)
- [ ] B: pick one, "Apply resolution"
- [ ] **Expect:** banner clears; pill returns to green
- [ ] A: sync — should pick up B's resolution choice

### 2.4 Long offline catch-up
- [ ] A: disconnect WiFi
- [ ] A: make ~20 edits across multiple pages
- [ ] A: reconnect WiFi
- [ ] A: sync (manual or wait for the runner)
- [ ] **Expect:** all 20 ops upload; sidebar settles to green
- [ ] B: sync
- [ ] **Expect:** all 20 changes visible on B; no errors

---

## 3. Failure modes (verify graceful recovery)

### 3.1 Wrong passphrase on re-enable
- [ ] Disable sync
- [ ] Re-enable with same backend but DIFFERENT passphrase
- [ ] **Expect:** Enable fails with a clear toast: "passphrase does not match the existing backend data — either the passphrase is wrong, or this bucket/folder already contains data from a different sync session"
- [ ] **Expect:** sync_config NOT updated; you can re-try with the correct passphrase
- [ ] (Pre-fix as of 2026-04-13: this used to silently accept wrong passphrase and produce confusing errors at sync time. Validate the fix actually rejects.)

### 3.2 Network down mid-sync
- [ ] Sync is enabled and working
- [ ] Disconnect WiFi
- [ ] Edit a page
- [ ] Click "Sync now"
- [ ] **Expect:** toast surfaces a network/auth error, NOT a panic
- [ ] **Expect:** edits queue locally (Pending uploads count grows)
- [ ] Reconnect WiFi, click "Sync now"
- [ ] **Expect:** queue drains; pill returns to green

### 3.3 Wrong bucket / bucket deleted
- [ ] Stop the sync, point at a non-existent bucket
- [ ] **Expect:** Enable fails with "bucket not found" or similar in the toast
- [ ] **Expect:** no crash, no zombie state

### 3.4 Auth failure (invalid keys)
- [ ] Stop sync, enable with an obviously invalid access key
- [ ] **Expect:** Enable fails with "authentication failed"

---

## 4. Deferred to Phase 5 (don't test these yet — not implemented)

- ❌ Recovery flow: download snapshot to fresh device with no local DB ("I lost my laptop, restore from sync")
- ❌ Automatic retry/backoff on transient errors
- ❌ Sync activity log viewer (Settings → Sync → Activity tab)
- ❌ OS keychain integration for credentials (still plaintext in `sync_config.backend_config`)
- ❌ First-run wizard (multi-step modal for new users)
- ❌ Multi-Tome background sync — runner only handles the active Tome

---

## 5. Things you've already validated (sanity baseline)

- ✅ S3 backend works against real Backblaze B2 (smoke test `tests/sync_s3_smoke.rs` passes; full round trip verified)
- ✅ Filesystem backend works (Phase 2 integration tests, 11 scenarios)
- ✅ Snapshot compression applied (97.3% reduction measured on real B2: 348 KB → 9.2 KB)
- ✅ Tauri runner spawns cleanly (the `tokio::spawn` panic was fixed in `tauri::async_runtime::spawn`)
- ✅ App launches cleanly on WSLg

---

## 6. Issues to track separately

- [ ] **Toast position** — moved from `bottom-right` to `top-center` 2026-04-13 because WSLg / window-sizing was hiding it. Re-verify on Windows desktop production builds; if still off, file follow-up.
- [ ] **Pending uploads counter accuracy** — Settings → Sync status card "Pending uploads" sometimes shows non-zero after a successful sync. The runner advances `last_uploaded_op_id` but the status query may race against the journal table; needs a deterministic refresh after sync completion.
- [ ] **"Take snapshot" feedback** — toast appears (since toast move) but verify it stays visible long enough; consider adding a "Snapshot taken (N KB uploaded)" with the size.

---

## When all of §1–§3 pass

Mark the master plan's Phase 5 prerequisite "v0.2.0 ship checklist → Manual smoke test of Filesystem backend end-to-end on desktop app" and "Manual smoke test of S3 backend against Minio per docs/sync-s3-testing.md" as ✅. Recovery (§4 first item) needs to be implemented + tested before tagging.
