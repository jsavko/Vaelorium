import { writable, derived } from 'svelte/store'
import { getSyncStatus, listConflicts, subscribeSyncStatus, type SyncStatus, type SyncConflict } from '../api/sync'

const initial: SyncStatus = {
  enabled: false,
  tomeId: null,
  backendKind: null,
  backendSummary: null,
  deviceName: null,
  lastSyncAt: null,
  lastError: null,
  queueSize: 0,
  pendingConflicts: 0,
}

export const syncStatus = writable<SyncStatus>(initial)
export const syncConflicts = writable<SyncConflict[]>([])
export const syncRunning = writable(false)

/** "idle" | "syncing" | "conflicts" | "offline" | "error" */
export const syncIndicator = derived(
  [syncStatus, syncRunning],
  ([$status, $running]) => {
    if (!$status.enabled) return 'offline' as const
    if ($running) return 'syncing' as const
    if ($status.lastError) return 'error' as const
    if ($status.pendingConflicts > 0) return 'conflicts' as const
    return 'idle' as const
  },
)

let unsubscribed: (() => void) | null = null

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
  } catch {
    // silent — sync may not be configured yet
  }
}

export async function initSyncStore() {
  await refreshSyncStatus()
  if (!unsubscribed) {
    unsubscribed = await subscribeSyncStatus((e) => {
      syncRunning.set(e.state === 'syncing')
      // After any event, refresh status to pull queue size + conflicts.
      refreshSyncStatus()
    })
  }
}
