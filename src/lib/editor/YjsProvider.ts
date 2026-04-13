import * as Y from 'yjs'
import { callCommand } from '../api/bridge'
import { savePageContent, getPageContent } from '../api/pages'

export interface SaveCallbacks {
  getHTML?: () => string
  getPlainText?: () => string
}

export class LocalYjsProvider {
  doc: Y.Doc
  pageId: string
  private saveTimeout: ReturnType<typeof setTimeout> | null = null
  private versionInterval: ReturnType<typeof setInterval> | null = null
  private saving = false
  private lastVersionTime = 0
  private dirtySinceLastVersion = false
  private callbacks: SaveCallbacks = {}

  constructor(pageId: string) {
    this.pageId = pageId
    this.doc = new Y.Doc()
  }

  setCallbacks(callbacks: SaveCallbacks) {
    this.callbacks = callbacks
  }

  async load(): Promise<void> {
    const stateArray = await getPageContent(this.pageId)
    if (stateArray && stateArray.length > 0) {
      const state = new Uint8Array(stateArray)
      Y.applyUpdate(this.doc, state)
    }

    this.doc.on('update', () => {
      this.dirtySinceLastVersion = true
      this.scheduleSave()
    })

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

      // Extract and save wiki links
      if (this.callbacks.getHTML) {
        const html = this.callbacks.getHTML()
        const links = this.extractWikiLinks(html)
        await callCommand('save_wiki_links', {
          sourcePageId: this.pageId,
          links,
        })
      }

      // Update search index
      if (this.callbacks.getPlainText) {
        const text = this.callbacks.getPlainText()
        await callCommand('update_search_index', {
          pageId: this.pageId,
          title: '', // Will be filled by the caller or from page data
          textContent: text,
        })
      }
    } finally {
      this.saving = false
    }
  }

  private extractWikiLinks(html: string): Array<{ target_page_id: string; link_text: string | null }> {
    const links: Array<{ target_page_id: string; link_text: string | null }> = []
    const regex = /href="#page:([^"]+)"[^>]*>([^<]*)</g
    let match
    while ((match = regex.exec(html)) !== null) {
      links.push({
        target_page_id: match[1],
        link_text: match[2] || null,
      })
    }
    return links
  }

  async createSnapshot(summary?: string): Promise<void> {
    const snapshot = Y.encodeStateAsUpdate(this.doc)
    await callCommand('create_version', {
      pageId: this.pageId,
      yjsSnapshot: Array.from(snapshot),
      summary: summary || null,
    })
    this.lastVersionTime = Date.now()
    this.dirtySinceLastVersion = false
  }

  private async maybeCreateVersion() {
    if (!this.dirtySinceLastVersion) return // nothing changed since last snapshot
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
