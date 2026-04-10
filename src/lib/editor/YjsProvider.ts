import * as Y from 'yjs'
import { callCommand } from '../api/bridge'
import { savePageContent, getPageContent } from '../api/pages'

export class LocalYjsProvider {
  doc: Y.Doc
  pageId: string
  private saveTimeout: ReturnType<typeof setTimeout> | null = null
  private versionInterval: ReturnType<typeof setInterval> | null = null
  private saving = false
  private lastVersionTime = 0

  constructor(pageId: string) {
    this.pageId = pageId
    this.doc = new Y.Doc()
  }

  async load(): Promise<void> {
    const stateArray = await getPageContent(this.pageId)
    if (stateArray && stateArray.length > 0) {
      const state = new Uint8Array(stateArray)
      Y.applyUpdate(this.doc, state)
    }

    this.doc.on('update', () => {
      this.scheduleSave()
    })

    // Auto-versioning: snapshot every 5 minutes of active editing
    this.lastVersionTime = Date.now()
    this.versionInterval = setInterval(() => {
      this.maybeCreateVersion()
    }, 5 * 60 * 1000)
  }

  private scheduleSave() {
    if (this.saveTimeout) {
      clearTimeout(this.saveTimeout)
    }
    this.saveTimeout = setTimeout(() => {
      this.save()
    }, 1000)
  }

  async save(): Promise<void> {
    if (this.saving) return
    this.saving = true
    try {
      const state = Y.encodeStateAsUpdate(this.doc)
      await savePageContent(this.pageId, Array.from(state))
    } finally {
      this.saving = false
    }
  }

  async createSnapshot(summary?: string): Promise<void> {
    const snapshot = Y.encodeStateAsUpdate(this.doc)
    await callCommand('create_version', {
      pageId: this.pageId,
      yjsSnapshot: Array.from(snapshot),
      summary: summary || null,
    })
    this.lastVersionTime = Date.now()
  }

  private async maybeCreateVersion() {
    const elapsed = Date.now() - this.lastVersionTime
    if (elapsed >= 5 * 60 * 1000) {
      await this.createSnapshot('Auto-save')
    }
  }

  async destroy(): Promise<void> {
    if (this.saveTimeout) clearTimeout(this.saveTimeout)
    if (this.versionInterval) clearInterval(this.versionInterval)
    await this.save()
    this.doc.destroy()
  }
}
