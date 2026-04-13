import { writable, derived } from 'svelte/store'
import { getSyncStatus, listConflicts, subscribeSyncStatus, tryAutoUnlock, type SyncStatus, type SyncConflict } from '../api/sync'

const initial: SyncStatus = {
  enabled: false,
  locked: false,
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

/** "idle" | "syncing" | "conflicts" | "offline" | "error" | "locked" */
export const syncIndicator = derived(
  [syncStatus, syncRunning],
  ([$status, $running]) => {
    if (!$status.enabled) return 'offline' as const
    if ($status.locked) return 'locked' as const
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
  } catch (e) {
    // Don't break UI if backend errors, but surface so the locked pill
    // not appearing isn't invisible.
    console.warn('[sync] refreshSyncStatus failed:', e)
  }
}

export async function initSyncStore() {
  await refreshSyncStatus()
  let current: SyncStatus | undefined
  syncStatus.subscribe((s) => { current = s })()
  console.info('[sync] initSyncStore → enabled=%s locked=%s tomeId=%s', current?.enabled, current?.locked, current?.tomeId)
  // If sync is configured but locked, try the OS keychain — most users will
  // have stored their passphrase and never see the manual unlock UI.
  if (current?.enabled && current.locked) {
    const ok = await tryAutoUnlock()
    if (ok) await refreshSyncStatus()
  }
  if (!unsubscribed) {
    unsubscribed = await subscribeSyncStatus((e) => {
      syncRunning.set(e.state === 'syncing')
      refreshSyncStatus()
    })
  }
}
