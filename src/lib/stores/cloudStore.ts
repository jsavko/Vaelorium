import { writable, derived, get } from 'svelte/store'
import { cloudAccountRefresh, cloudStatus, type CloudAccountInfo } from '../api/cloud'
import { backupStatus } from './syncStore'

/** Shared cloud-account state. Populated on app init (from cached
 *  `cloud_status`) and refreshed whenever the user opens a surface
 *  that shows account / usage info. Per the cloud no-polling contract,
 *  we don't auto-refresh on a timer — each refresh is user-initiated
 *  (signin, Settings open, create-Tome modal open). */
export const cloudAccount = writable<CloudAccountInfo | null>(null)

/** Refresh cached account state. Prefers `/api/account` so plan/usage
 *  changes made out-of-band (Stripe webhook, admin tool) are visible
 *  without waiting for the next sync. Falls back to the locally cached
 *  `cloud_status` if the network call errors, so the UI keeps showing
 *  stale-but-present data offline. */
export async function refreshCloudAccount(): Promise<void> {
  try {
    const fresh = await cloudAccountRefresh().catch(() => null)
    cloudAccount.set(fresh ?? (await cloudStatus()))
  } catch (e) {
    console.warn('[cloud] refreshCloudAccount failed:', e)
    cloudAccount.set(null)
  }
}

/** True when the user is on a hosted backup AND the cached plan usage
 *  says they're at or above their plan's Tome limit. Author tier has
 *  `tomeLimit: null` (unlimited) so it never trips. Non-hosted backups
 *  and signed-out states return `false` — the banner is cloud-specific
 *  and shouldn't appear for filesystem / S3 users. */
export const atTomeQuota = derived(
  [cloudAccount, backupStatus],
  ([$account, $backup]) => {
    if ($backup.backendKind !== 'hosted') return false
    const usage = $account?.usage
    if (!usage || usage.tomeLimit === null) return false
    return usage.tomeCount >= usage.tomeLimit
  },
)

/** Convenience accessor for the cached usage snapshot — lets callers
 *  render "N of M Tomes" copy without redundantly subscribing to the
 *  whole account store. */
export function getCloudUsage() {
  return get(cloudAccount)?.usage ?? null
}
