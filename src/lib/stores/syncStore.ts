import { writable, derived, get } from 'svelte/store'
import { getSyncStatus, listConflicts, listActivity, subscribeSyncStatus, type SyncStatus, type SyncConflict, type ActivityRow } from '../api/sync'
import { getBackupStatus, listRestorableTomes, tryAutoUnlockBackup, type BackupStatus, type RestorableTome } from '../api/backup'

const initialSync: SyncStatus = {
  enabled: false,
  locked: false,
  backupMissing: true,
  tomeId: null,
  backendKind: null,
  backendSummary: null,
  deviceName: null,
  lastSyncAt: null,
  lastError: null,
  queueSize: 0,
  pendingConflicts: 0,
}

const initialBackup: BackupStatus = {
  configured: false,
  locked: false,
  backendKind: null,
  backendSummary: null,
  deviceName: null,
}

export const syncStatus = writable<SyncStatus>(initialSync)
export const backupStatus = writable<BackupStatus>(initialBackup)
export const syncConflicts = writable<SyncConflict[]>([])
export const syncActivity = writable<ActivityRow[]>([])
export const syncRunning = writable(false)

/** Runner poll interval in ms — mirrors `sync::runner::POLL_INTERVAL`
 *  in the Rust side. Exposed here purely so the sidebar pill can
 *  compute a "next sync in X" countdown. Not authoritative — the
 *  runner fires on nudges too (tome open/close, sync_enable, manual
 *  Sync now). */
export const SYNC_POLL_INTERVAL_MS = 10 * 60 * 1000

/** Cached list of Tomes the configured backend has. Loaded lazily on
 *  the first configured+unlocked transition or after explicit events
 *  that can change the list (sign-in, sign-out, sync enable/disable,
 *  delete-from-backup, manual Sync now). Read by TomePicker's
 *  "Restore from backup" panel + any Tome-card cloud-badge
 *  correlation. Never auto-polled. */
export const restorableTomes = writable<RestorableTome[]>([])
export const restorableLoading = writable(false)
export const restorableError = writable<string | null>(null)

let lastRestorableFetchKey = ''
let restorableInflight: Promise<void> | null = null

export async function refreshRestorable(force: boolean = false) {
  // Dedupe concurrent callers.
  if (restorableInflight) return restorableInflight
  const backup = get(backupStatus)
  const status = get(syncStatus)
  if (!backup.configured || backup.locked) {
    restorableTomes.set([])
    restorableError.set(null)
    lastRestorableFetchKey = 'locked'
    return
  }
  // Key-skip: re-fetching is wasteful if the relevant state
  // hasn't changed. Keyed on backend + email + lastSyncAt so a new
  // sync tick or account switch still invalidates the cache.
  const key = `${backup.backendKind}|${backup.deviceName}|${status.lastSyncAt ?? ''}`
  if (!force && key === lastRestorableFetchKey) return
  lastRestorableFetchKey = key
  restorableLoading.set(true)
  restorableError.set(null)
  const p = (async () => {
    try {
      const list = await listRestorableTomes()
      restorableTomes.set(list)
    } catch (e) {
      restorableError.set(e instanceof Error ? e.message : String(e))
      restorableTomes.set([])
    } finally {
      restorableLoading.set(false)
    }
  })()
  restorableInflight = p
  await p
  restorableInflight = null
}
/** Hosted-cloud device token revoked / expired. Until the user
 *  re-signs in from Settings → Backup, the runner will keep failing. */
export const syncUnauthorized = writable(false)

/** "idle" | "syncing" | "conflicts" | "offline" | "error" | "locked" | "backup-missing" | "unauthorized" */
export const syncIndicator = derived(
  [syncStatus, backupStatus, syncRunning, syncUnauthorized],
  ([$status, $backup, $running, $unauthorized]) => {
    if (!$backup.configured) return 'backup-missing' as const
    if ($backup.locked) return 'locked' as const
    if ($unauthorized) return 'unauthorized' as const
    if (!$status.enabled) return 'offline' as const
    if ($status.locked) return 'locked' as const
    if ($running) return 'syncing' as const
    if ($status.lastError) return 'error' as const
    if ($status.pendingConflicts > 0) return 'conflicts' as const
    return 'idle' as const
  },
)

let unsubscribed: (() => void) | null = null

export async function refreshBackupStatus() {
  try {
    backupStatus.set(await getBackupStatus())
  } catch (e) {
    console.warn('[sync] refreshBackupStatus failed:', e)
  }
}

export async function refreshActivity() {
  try {
    syncActivity.set(await listActivity(100))
  } catch (e) {
    console.warn('[sync] refreshActivity failed:', e)
  }
}

export async function refreshSyncStatus() {
  try {
    const s = await getSyncStatus()
    syncStatus.set(s)
    // Always fetch conflicts when the count is non-zero, regardless of
    // sync_config.enabled. A Tome can have unresolved conflicts in its DB
    // even with sync currently disabled (e.g. restored from backup before
    // the auto-enable fix shipped) — and the pill's conflict count comes
    // from a COUNT query that doesn't gate on enabled either. Mismatched
    // gating here caused the conflict pill to route nowhere useful.
    if (s.pendingConflicts > 0) {
      const c = await listConflicts()
      syncConflicts.set(c)
    } else {
      syncConflicts.set([])
    }
  } catch (e) {
    console.warn('[sync] refreshSyncStatus failed:', e)
  }
}

export async function initSyncStore() {
  await refreshBackupStatus()
  await refreshSyncStatus()
  let backup: BackupStatus | undefined
  backupStatus.subscribe((s) => { backup = s })()
  if (backup?.configured && backup.locked) {
    const ok = await tryAutoUnlockBackup()
    if (ok) {
      await refreshBackupStatus()
      await refreshSyncStatus()
    }
  }
  if (!unsubscribed) {
    unsubscribed = await subscribeSyncStatus((e) => {
      syncRunning.set(e.state === 'syncing')
      // Hosted-cloud token revoked → pill jumps to "unauthorized" state.
      // Cleared on successful signin (see cloud_signin handler).
      if (e.state === 'unauthorized') {
        syncUnauthorized.set(true)
      } else if (e.state === 'idle' || e.state === 'syncing') {
        syncUnauthorized.set(false)
      }
      refreshSyncStatus()
      refreshActivity()
      // A freshly-completed sync may have added or removed Tomes on
      // the backend; refresh the restorable cache to keep TomePicker
      // correct without adding its own poll loop.
      if (e.state === 'idle') {
        refreshRestorable()
      }
    })
  }
  await refreshActivity()
  // Initial fetch if already unlocked — lets TomePicker render the
  // restore list on first app open without hitting the network again
  // later.
  await refreshRestorable()
}
