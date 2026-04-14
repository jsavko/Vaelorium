import { callCommand, isTauri } from './bridge'

export interface CloudAccountInfo {
  email: string
  accountId: string
  tier: string | null
  signedInAt: string | null
}

interface RawCloudAccountInfo {
  email: string
  account_id: string
  tier: string | null
  signed_in_at: string | null
}

const fromRaw = (r: RawCloudAccountInfo): CloudAccountInfo => ({
  email: r.email,
  accountId: r.account_id,
  tier: r.tier,
  signedInAt: r.signed_in_at,
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
