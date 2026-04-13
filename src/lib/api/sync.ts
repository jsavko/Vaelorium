import { callCommand, isTauri } from './bridge'

export interface SyncStatus {
  enabled: boolean
  tomeId: string | null
  backendKind: string | null
  backendSummary: string | null
  deviceName: string | null
  lastSyncAt: string | null
  lastError: string | null
  queueSize: number
  pendingConflicts: number
}

export interface SyncOutcome {
  ops_uploaded: number
  ops_applied: number
  conflicts_created: number
  snapshot_taken: string | null
  error: string | null
}

export interface SyncConflict {
  conflictId: string
  tableName: string
  rowId: string
  fieldName: string
  /** JSON-encoded; parse with JSON.parse, may be null. */
  localValue: string | null
  remoteValue: string | null
  localOpId: string
  remoteOpId: string
  detectedAt: string
}

interface RawStatus {
  enabled: boolean
  tome_id: string | null
  backend_kind: string | null
  backend_summary: string | null
  device_name: string | null
  last_sync_at: string | null
  last_error: string | null
  queue_size: number
  pending_conflicts: number
}

interface RawConflict {
  conflict_id: string
  table_name: string
  row_id: string
  field_name: string
  local_value: string | null
  remote_value: string | null
  local_op_id: string
  remote_op_id: string
  detected_at: string
}

const fromRawStatus = (r: RawStatus): SyncStatus => ({
  enabled: r.enabled,
  tomeId: r.tome_id,
  backendKind: r.backend_kind,
  backendSummary: r.backend_summary,
  deviceName: r.device_name,
  lastSyncAt: r.last_sync_at,
  lastError: r.last_error,
  queueSize: r.queue_size,
  pendingConflicts: r.pending_conflicts,
})

const fromRawConflict = (r: RawConflict): SyncConflict => ({
  conflictId: r.conflict_id,
  tableName: r.table_name,
  rowId: r.row_id,
  fieldName: r.field_name,
  localValue: r.local_value,
  remoteValue: r.remote_value,
  localOpId: r.local_op_id,
  remoteOpId: r.remote_op_id,
  detectedAt: r.detected_at,
})

export interface EnableSyncInput {
  tomeId: string
  backendKind: 'filesystem' | 's3'
  backendConfig: Record<string, unknown>
  passphrase: string
  deviceName?: string
}

export async function enableSync(input: EnableSyncInput): Promise<SyncStatus> {
  const raw = await callCommand<RawStatus>('sync_enable', {
    input: {
      tome_id: input.tomeId,
      backend_kind: input.backendKind,
      backend_config: input.backendConfig,
      passphrase: input.passphrase,
      device_name: input.deviceName ?? null,
    },
  })
  return fromRawStatus(raw)
}

export async function disableSync(tomeId: string): Promise<SyncStatus> {
  const raw = await callCommand<RawStatus>('sync_disable', { tomeId })
  return fromRawStatus(raw)
}

export async function syncNow(): Promise<SyncOutcome> {
  return callCommand<SyncOutcome>('sync_now')
}

export async function getSyncStatus(): Promise<SyncStatus> {
  if (!isTauri) {
    return {
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
  }
  const raw = await callCommand<RawStatus>('sync_status')
  return fromRawStatus(raw)
}

export async function takeSnapshot(): Promise<string> {
  return callCommand<string>('sync_take_snapshot')
}

export async function listConflicts(): Promise<SyncConflict[]> {
  if (!isTauri) return []
  const raw = await callCommand<RawConflict[]>('sync_list_conflicts')
  return raw.map(fromRawConflict)
}

export async function resolveConflict(conflictId: string, chooseLocal: boolean): Promise<void> {
  await callCommand<void>('sync_resolve_conflict', {
    input: { conflict_id: conflictId, choose_local: chooseLocal },
  })
}

export async function unlockSync(passphrase: string): Promise<SyncStatus> {
  const raw = await callCommand<RawStatus>('sync_unlock', { passphrase })
  return fromRawStatus(raw)
}

/** Subscribe to sync:status events. Returns an unsubscribe function. */
export async function subscribeSyncStatus(
  cb: (event: { tome_id: string; state: string; ops_uploaded: number; ops_applied: number; conflicts_created: number; error: string | null }) => void,
): Promise<() => void> {
  if (!isTauri) return () => {}
  const { listen } = await import('@tauri-apps/api/event')
  const un = await listen<{ tome_id: string; state: string; ops_uploaded: number; ops_applied: number; conflicts_created: number; error: string | null }>(
    'sync:status',
    (e) => cb(e.payload),
  )
  return un
}
