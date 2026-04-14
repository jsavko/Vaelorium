import { callCommand, isTauri } from './bridge'

export interface BackupStatus {
  configured: boolean
  locked: boolean
  backendKind: string | null
  backendSummary: string | null
  deviceName: string | null
}

interface RawBackupStatus {
  configured: boolean
  locked: boolean
  backend_kind: string | null
  backend_summary: string | null
  device_name: string | null
}

const fromRaw = (r: RawBackupStatus): BackupStatus => ({
  configured: r.configured,
  locked: r.locked,
  backendKind: r.backend_kind,
  backendSummary: r.backend_summary,
  deviceName: r.device_name,
})

export interface ConfigureBackupInput {
  backendKind: 'filesystem' | 's3' | 'hosted'
  backendConfig: Record<string, unknown>
  passphrase: string
  deviceName?: string
}

/** Does this backend already contain encrypted Tome data? Lets the setup
 *  wizard pre-select the "first device" vs "adding to existing" branch
 *  without asking the user. Not meaningful for hosted — callers should
 *  read `CloudAccountInfo.usage` from the signin response instead. */
export async function probeBucketHasData(
  backendKind: 'filesystem' | 's3',
  backendConfig: Record<string, unknown>,
): Promise<boolean> {
  if (!isTauri) return false
  return callCommand<boolean>('backup_probe_bucket_has_data', {
    backendKind,
    backendConfig,
  })
}

export async function configureBackup(input: ConfigureBackupInput): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_configure', {
    input: {
      backend_kind: input.backendKind,
      backend_config: input.backendConfig,
      passphrase: input.passphrase,
      device_name: input.deviceName ?? null,
    },
  })
  return fromRaw(raw)
}

export async function setBackupDeviceName(deviceName: string): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_set_device_name', {
    input: { device_name: deviceName },
  })
  return fromRaw(raw)
}

export async function disconnectBackup(): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_disconnect')
  return fromRaw(raw)
}

export async function getBackupStatus(): Promise<BackupStatus> {
  if (!isTauri) {
    return { configured: false, locked: false, backendKind: null, backendSummary: null, deviceName: null }
  }
  const raw = await callCommand<RawBackupStatus>('backup_status')
  return fromRaw(raw)
}

export async function unlockBackup(passphrase: string): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_unlock', { passphrase })
  return fromRaw(raw)
}

export async function tryAutoUnlockBackup(): Promise<boolean> {
  if (!isTauri) return false
  try {
    return await callCommand<boolean>('backup_try_auto_unlock')
  } catch {
    return false
  }
}

export interface RestorableTome {
  tomeUuid: string
  snapshotId: string
  name: string
  description: string | null
  sizeBytes: number
  lastModified: string
}

interface RawRestorableTome {
  tome_uuid: string
  snapshot_id: string
  name: string
  description: string | null
  size_bytes: number
  last_modified: string
}

export async function listRestorableTomes(): Promise<RestorableTome[]> {
  if (!isTauri) return []
  const raw = await callCommand<RawRestorableTome[]>('backup_list_restorable_tomes')
  return raw.map((r) => ({
    tomeUuid: r.tome_uuid,
    snapshotId: r.snapshot_id,
    name: r.name,
    description: r.description,
    sizeBytes: r.size_bytes,
    lastModified: r.last_modified,
  }))
}

export interface RestoredTome {
  path: string
  name: string
  tomeUuid: string
}

interface RawRestoredTome {
  path: string
  name: string
  tome_uuid: string
}

export async function restoreTomeFromBackup(tomeUuid: string): Promise<RestoredTome> {
  const raw = await callCommand<RawRestoredTome>('backup_restore_tome', {
    input: { tome_uuid: tomeUuid },
  })
  return { path: raw.path, name: raw.name, tomeUuid: raw.tome_uuid }
}

export interface DeleteTomeResult {
  deletedObjects: number
  deletedBytes: number
}

interface RawDeleteTomeResult {
  deleted_objects: number
  deleted_bytes: number
}

export async function deleteTomeFromBackup(tomeUuid: string): Promise<DeleteTomeResult> {
  const raw = await callCommand<RawDeleteTomeResult>('backup_delete_tome', {
    input: { tome_uuid: tomeUuid },
  })
  return { deletedObjects: raw.deleted_objects, deletedBytes: raw.deleted_bytes }
}
