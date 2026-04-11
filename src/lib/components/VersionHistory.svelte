<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { currentPageId, loadPage } from '../stores/pageStore'
  import { savePageContent } from '../api/pages'
  import { showToast } from '../stores/toastStore'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()

  interface Version {
    id: string
    page_id: string
    version_number: number
    created_at: string
    created_by: string | null
    summary: string | null
  }

  let versions = $state<Version[]>([])
  let selectedVersion = $state<Version | null>(null)
  let snapshotPreview = $state<string>('')

  $effect(() => {
    if (open && $currentPageId) {
      loadVersions($currentPageId)
      selectedVersion = null
      snapshotPreview = ''
    }
  })

  async function loadVersions(pageId: string) {
    try {
      versions = await callCommand('list_versions', { pageId })
    } catch {
      versions = []
    }
  }

  async function viewVersion(version: Version) {
    selectedVersion = version
    try {
      const snapshot: number[] = await callCommand('get_version_snapshot', { versionId: version.id })
      if (snapshot && snapshot.length > 0) {
        // Decode Yjs snapshot to get text preview
        const Y = await import('yjs')
        const doc = new Y.Doc()
        Y.applyUpdate(doc, new Uint8Array(snapshot))
        const fragment = doc.getXmlFragment('content')
        snapshotPreview = extractText(fragment)
        doc.destroy()
      } else {
        snapshotPreview = '(empty snapshot)'
      }
    } catch {
      snapshotPreview = '(unable to load snapshot)'
    }
  }

  function extractText(node: any): string {
    const parts: string[] = []
    try {
      const items = node.toArray ? node.toArray() : []
      for (const item of items) {
        if (item.toString) {
          const str = item.toString()
          if (str && !str.startsWith('[object')) {
            parts.push(str)
          }
        }
        if (item.toArray) {
          parts.push(extractText(item))
        }
      }
    } catch {}
    return parts.join('\n') || '(no text content)'
  }

  async function restoreVersion() {
    if (!selectedVersion || !$currentPageId) return
    try {
      const snapshot: number[] = await callCommand('get_version_snapshot', { versionId: selectedVersion.id })
      if (snapshot && snapshot.length > 0) {
        await savePageContent($currentPageId, snapshot)
        showToast(`Restored to v${selectedVersion.version_number}`, 'success')
        // Reload the page to reflect restored content
        await loadPage($currentPageId)
        onClose()
      }
    } catch {
      showToast('Failed to restore version', 'error')
    }
  }

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString()
  }
</script>

{#if open}
  <div class="version-panel" data-testid="version-history">
    <header class="panel-header">
      <span class="panel-title">Version History</span>
      <button class="close-btn" onclick={onClose} aria-label="Close">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>
    </header>

    <div class="panel-divider"></div>

    <div class="panel-body">
      <div class="version-list">
        {#if versions.length === 0}
          <p class="empty">No versions yet. Versions are created automatically every 5 minutes.</p>
        {:else}
          {#each versions as v (v.id)}
            <button
              class="version-item"
              class:selected={selectedVersion?.id === v.id}
              onclick={() => viewVersion(v)}
            >
              <div class="version-header">
                <span class="version-num">v{v.version_number}</span>
                <span class="version-date">{formatDate(v.created_at)}</span>
              </div>
              {#if v.summary}
                <span class="version-summary">{v.summary}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>

      {#if selectedVersion}
        <div class="version-preview">
          <div class="preview-header">
            <span class="preview-title">v{selectedVersion.version_number} Preview</span>
            <button class="restore-btn" onclick={restoreVersion}>
              Restore this version
            </button>
          </div>
          <div class="preview-content">
            <pre>{snapshotPreview}</pre>
          </div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .version-panel {
    position: fixed;
    right: 0;
    top: 0;
    width: 400px;
    height: 100%;
    background: var(--color-surface-secondary);
    border-left: 1px solid var(--color-border-subtle);
    box-shadow: -8px 0 24px rgba(0, 0, 0, 0.2);
    z-index: 150;
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
  }

  .panel-title {
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 600;
    color: var(--color-fg-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
  }

  .panel-divider {
    height: 1px;
    background: var(--color-border-subtle);
  }

  .panel-body {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .version-list {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .empty {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    text-align: center;
    padding: 20px;
  }

  .version-item {
    padding: 10px 12px;
    background: transparent;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: pointer;
    text-align: left;
    width: 100%;
  }

  .version-item:hover {
    background: var(--color-surface-tertiary);
  }

  .version-item.selected {
    background: var(--color-accent-gold-subtle);
    border-color: var(--color-accent-gold);
  }

  .version-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .version-num {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-accent-gold);
  }

  .version-date {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
  }

  .version-summary {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
  }

  .version-preview {
    border-top: 1px solid var(--color-border-subtle);
    padding: 12px;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .preview-title {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 600;
    color: var(--color-fg-primary);
  }

  .restore-btn {
    padding: 6px 14px;
    background: var(--color-accent-gold);
    border: none;
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 600;
    color: var(--color-fg-inverse);
    cursor: pointer;
  }

  .restore-btn:hover {
    background: var(--color-accent-gold-hover);
  }

  .preview-content {
    flex: 1;
    overflow-y: auto;
    background: var(--color-surface-primary);
    border-radius: var(--radius-sm);
    padding: 12px;
  }

  .preview-content pre {
    font-family: var(--font-body);
    font-size: 13px;
    color: var(--color-fg-secondary);
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }
</style>
