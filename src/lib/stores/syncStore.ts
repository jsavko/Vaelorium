import { writable, derived } from 'svelte/store'
import { getSyncStatus, listConflicts, subscribeSyncStatus, type SyncStatus, type SyncConflict } from '../api/sync'
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
export const syncRunning = writable(false)

/** "idle" | "syncing" | "conflicts" | "offline" | "error" | "locked" | "backup-missing" */
export const syncIndicator = derived(
  [syncStatus, backupStatus, syncRunning],
  ([$status, $backup, $running]) => {
    if (!$backup.configured) return 'backup-missing' as const
    if ($backup.locked) return 'locked' as const
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

export async function refreshSyncStatus() {
  try {
    const s = await getSyncStatus()
    syncStatus.set(s)
    if (s.enabled) {
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
      refreshSyncStatus()
    })
  }
}
