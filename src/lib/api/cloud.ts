import { callCommand, isTauri } from './bridge'

export interface CloudUsage {
  bytesUsed: number
  tomeCount: number
  quotaBytes: number
  /** `null` on Author tier (unlimited). */
  tomeLimit: number | null
  subscriptionStatus: string
}

export interface CloudAccountInfo {
  email: string
  accountId: string
  tier: string | null
  signedInAt: string | null
  usage: CloudUsage | null
}

interface RawCloudUsage {
  bytes_used: number
  tome_count: number
  quota_bytes: number
  tome_limit: number | null
  subscription_status: string
}

interface RawCloudAccountInfo {
  email: string
  account_id: string
  tier: string | null
  signed_in_at: string | null
  usage: RawCloudUsage | null
}

const fromRawUsage = (u: RawCloudUsage): CloudUsage => ({
  bytesUsed: u.bytes_used,
  tomeCount: u.tome_count,
  quotaBytes: u.quota_bytes,
  tomeLimit: u.tome_limit,
  subscriptionStatus: u.subscription_status,
})

const fromRaw = (r: RawCloudAccountInfo): CloudAccountInfo => ({
  email: r.email,
  accountId: r.account_id,
  tier: r.tier,
  signedInAt: r.signed_in_at,
  usage: r.usage ? fromRawUsage(r.usage) : null,
})

export interface CloudSigninInput {
  email: string
  password: string
  deviceName?: string
}

export async function cloudSignin(input: CloudSigninInput): Promise<CloudAccountInfo> {
  const raw = await callCommand<RawCloudAccountInfo>('cloud_signin', {
    input: {
      email: input.email,
      password: input.password,
      device_name: input.deviceName ?? null,
    },
  })
  return fromRaw(raw)
}

export async function cloudSignout(): Promise<void> {
  if (!isTauri) return
  await callCommand<void>('cloud_signout')
}

export async function cloudStatus(): Promise<CloudAccountInfo | null> {
  if (!isTauri) return null
  const raw = await callCommand<RawCloudAccountInfo | null>('cloud_status')
  return raw ? fromRaw(raw) : null
}

/**
 * Refresh cached account state by calling `/api/account`. Per cloud
 * `efc9286` no polling outside login / sync; this is the one-shot
 * "refresh state after sign-in survived from a prior launch" hook.
 */
export async function cloudAccountRefresh(): Promise<CloudAccountInfo | null> {
  if (!isTauri) return null
  const raw = await callCommand<RawCloudAccountInfo | null>('cloud_account_refresh')
  return raw ? fromRaw(raw) : null
}
