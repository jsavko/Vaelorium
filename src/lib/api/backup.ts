import { callCommand, isTauri } from './bridge'

export interface BackupStatus {
  configured: boolean
  locked: boolean
  backendKind: string | null
  backendSummary: string | null
}

interface RawBackupStatus {
  configured: boolean
  locked: boolean
  backend_kind: string | null
  backend_summary: string | null
}

const fromRaw = (r: RawBackupStatus): BackupStatus => ({
  configured: r.configured,
  locked: r.locked,
  backendKind: r.backend_kind,
  backendSummary: r.backend_summary,
})

export interface ConfigureBackupInput {
  backendKind: 'filesystem' | 's3'
  backendConfig: Record<string, unknown>
  passphrase: string
}

export async function configureBackup(input: ConfigureBackupInput): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_configure', {
    input: {
      backend_kind: input.backendKind,
      backend_config: input.backendConfig,
      passphrase: input.passphrase,
    },
  })
  return fromRaw(raw)
}

export async function disconnectBackup(): Promise<BackupStatus> {
  const raw = await callCommand<RawBackupStatus>('backup_disconnect')
  return fromRaw(raw)
}

export async function getBackupStatus(): Promise<BackupStatus> {
  if (!isTauri) {
    return { configured: false, locked: false, backendKind: null, backendSummary: null }
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
