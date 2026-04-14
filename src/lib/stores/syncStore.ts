import { writable, derived } from 'svelte/store'
import { getSyncStatus, listConflicts, listActivity, subscribeSyncStatus, type SyncStatus, type SyncConflict, type ActivityRow } from '../api/sync'
import { getBackupStatus, tryAutoUnlockBackup, type BackupStatus } from '../api/backup'

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
    })
  }
  await refreshActivity()
}
