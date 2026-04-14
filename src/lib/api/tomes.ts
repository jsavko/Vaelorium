import { callCommand } from './bridge'

export interface RecentTome {
  path: string
  name: string
  description: string | null
  last_opened: string
  /** Per-Tome stable UUID from tome_metadata; populated when the Tome
   *  is opened / created / restored. `null` on legacy entries from
   *  pre-M5 launches. */
  tome_uuid: string | null
  /** Whether this Tome currently has local sync enabled. Mirrored
   *  from its own `sync_config.enabled` at open / sync-toggle time.
   *  Drives the TomePicker cloud badge + restore-list membership so
   *  stop-sync produces coherent UX without opening each Tome's DB. */
  sync_enabled: boolean
}

export interface AppState {
  recent_tomes: RecentTome[]
}

export interface TomeInfo {
  path: string
  name: string
  description: string | null
}

export interface TomeMetadata {
  name: string
  description: string | null
  cover_image: string | null
  created_at: string
}

export async function getAppState(): Promise<AppState> {
  return callCommand('get_app_state')
}

export async function createTome(
  path: string,
  name: string,
  description?: string | null,
): Promise<TomeInfo> {
  return callCommand('create_tome', { path, name, description })
}

export async function openTome(path: string): Promise<TomeInfo> {
  return callCommand('open_tome', { path })
}

export async function closeTome(): Promise<void> {
  return callCommand('close_tome')
}

export async function getTomeMetadata(): Promise<TomeMetadata> {
  return callCommand('get_tome_metadata')
}

export async function updateTomeMetadata(key: string, value: string | null): Promise<void> {
  return callCommand('update_tome_metadata', { key, value })
}
