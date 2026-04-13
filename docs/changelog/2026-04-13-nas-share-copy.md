# Clarify that NAS / network shares work with the Filesystem backend

**Date:** 2026-04-13
**Plan:** `.claude/plans/archive/2026-04-13-add-webdav-backend.md`

## Summary

Re-scoped from "add a WebDAV backend" after verifying that SMB/UNC/NFS-mounted shares work today with the existing Filesystem backend. Updated wizard copy + user guide to make that fact discoverable. No backend code changes.

## Files Modified

- `src/lib/components/BackupSetupWizard.svelte` — step 2 Folder card copy mentions NAS + UNC explicitly; step 3 picker helper text notes mounted shares work.
- `docs/sync-user-guide.md` — new "Using a NAS or network share" section with per-OS example paths, credentials note, and the mount-stays-active caveat.

## Rationale

Users asking "can I use my Unraid / Synology / TrueNAS share?" were landing on the Folder card and assuming it meant local disk only. The Filesystem backend has always accepted any path the OS can write to; the wizard just didn't say so. Dropping the dedicated WebDAV backend plan (previously considered, ~300-400 LOC) in favor of this 2-field + 1-section copy edit.
