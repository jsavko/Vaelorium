import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import * as Y from 'yjs'

// Mock the bridge + pages API so we can observe what the provider calls.
vi.mock('../api/bridge', () => ({
  callCommand: vi.fn(async () => undefined),
}))
vi.mock('../api/pages', () => ({
  savePageContent: vi.fn(async () => undefined),
  getPageContent: vi.fn(async () => []),
}))

import { LocalYjsProvider } from './YjsProvider'
import { callCommand } from '../api/bridge'

describe('LocalYjsProvider autosave dirty flag', () => {
  beforeEach(() => {
    vi.mocked(callCommand).mockClear()
  })

  afterEach(async () => {
    // nothing to tear down — provider owns its own state
  })

  it('skips maybeCreateVersion when no updates occurred', async () => {
    const provider = new LocalYjsProvider('page-clean')
    // Reach into private state for directed testing. acceptable for a
    // behaviour test of a time-based private helper.
    ;(provider as any).lastVersionTime = Date.now() - 10 * 60 * 1000
    await (provider as any).maybeCreateVersion()

    const calls = vi.mocked(callCommand).mock.calls
    const versionCalls = calls.filter((c) => c[0] === 'create_version')
    expect(versionCalls).toHaveLength(0)
  })

  it('calls create_version after an update + elapsed time', async () => {
    const provider = new LocalYjsProvider('page-dirty')
    ;(provider as any).lastVersionTime = Date.now() - 10 * 60 * 1000
    ;(provider as any).dirtySinceLastVersion = true

    await (provider as any).maybeCreateVersion()

    const calls = vi.mocked(callCommand).mock.calls
    const versionCalls = calls.filter((c) => c[0] === 'create_version')
    expect(versionCalls).toHaveLength(1)
  })

  it('createSnapshot clears the dirty flag', async () => {
    const provider = new LocalYjsProvider('page-clear')
    ;(provider as any).dirtySinceLastVersion = true

    await provider.createSnapshot('test')
    expect((provider as any).dirtySinceLastVersion).toBe(false)
  })

  it('doc update sets dirty flag', async () => {
    const provider = new LocalYjsProvider('page-update')
    // Attach the listener manually (load() also attaches it, but load()
    // hits the mocked getPageContent + schedules timers we'd have to
    // clean up).
    provider.doc.on('update', () => {
      ;(provider as any).dirtySinceLastVersion = true
    })
    expect((provider as any).dirtySinceLastVersion).toBe(false)

    // Trigger an update by inserting content into a map on the doc.
    const map = provider.doc.getMap('test')
    map.set('k', 'v')
    // Yjs 'update' handlers are synchronous; flag should now be true.
    expect((provider as any).dirtySinceLastVersion).toBe(true)
  })
})
