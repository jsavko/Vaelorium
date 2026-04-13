<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { currentPageId, loadPage, triggerPageReload } from '../stores/pageStore'
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
  let previewHTML = $state<string>('')
  let previewLoading = $state(false)

  $effect(() => {
    if (open && $currentPageId) {
      loadVersions($currentPageId)
      selectedVersion = null
      previewHTML = ''
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
    previewLoading = true
    previewHTML = ''
    try {
      const snapshot: number[] = await callCommand('get_version_snapshot', { versionId: version.id })
      if (!snapshot || snapshot.length === 0) {
        previewHTML = '<p class="preview-empty">(empty snapshot)</p>'
        return
      }
      // Render via a throwaway read-only Tiptap instance sharing the live
      // editor's extensions — guarantees the preview formatting exactly
      // matches what the editor displays.
      const [{ Editor }, Y, { createEditorExtensions }] = await Promise.all([
        import('@tiptap/core'),
        import('yjs'),
        import('../editor/EditorConfig'),
      ])
      const doc = new Y.Doc()
      Y.applyUpdate(doc, new Uint8Array(snapshot))
      const tmpEl = document.createElement('div')
      const editor = new Editor({
        element: tmpEl,
        editable: false,
        // Use the preview-mode extension list: no suggestion/mention
        // handlers that would conflict with the live editor's singletons.
        extensions: createEditorExtensions(doc, { forPreview: true }),
      })
      previewHTML = editor.getHTML()
      editor.destroy()
      doc.destroy()
    } catch (e) {
      console.warn('version preview failed', e)
      previewHTML = '<p class="preview-empty">(unable to load snapshot)</p>'
    } finally {
      previewLoading = false
    }
  }

  async function restoreVersion() {
    if (!selectedVersion || !$currentPageId) return
    try {
      // Take a "before restore" snapshot so the current content is never
      // lost silently. The live editor's YjsProvider will eventually
      // autosave too, but being explicit here means a Restore always has
      // an undo path even if the user restores immediately.
      try {
        // Save current content as a named version before replacing it.
        // We approximate by making a normal snapshot via the editor's
        // autosave cycle on the next tick — but we can also ask the
        // backend to create one directly from the current page content.
        const currentBytes: number[] = await callCommand('get_page_content', {
          pageId: $currentPageId,
        })
        if (currentBytes && currentBytes.length > 0) {
          await callCommand('create_version', {
            pageId: $currentPageId,
            yjsSnapshot: currentBytes,
            summary: `Before restore to v${selectedVersion.version_number}`,
          })
        }
      } catch (e) {
        console.warn('pre-restore snapshot failed; continuing', e)
      }

      const snapshot: number[] = await callCommand('get_version_snapshot', { versionId: selectedVersion.id })
      if (snapshot && snapshot.length > 0) {
        await savePageContent($currentPageId, snapshot)
        // Re-fetch metadata in case title etc. was part of the restore
        // (today page metadata is separate from yjs_state, so this is a no-op
        // for content but future-proofs). More importantly: signal the
        // Editor to destroy its in-memory Y.Doc and reload from DB —
        // otherwise the old content autosaves over the restore.
        await loadPage($currentPageId)
        triggerPageReload()
        showToast(`Restored to v${selectedVersion.version_number}`, 'success')
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

    <div class="panel-body" class:has-selection={!!selectedVersion}>
      <div class="version-list">
        {#if versions.length === 0}
          <p class="empty">No versions yet. Versions are created automatically every 5 minutes when the page has changed.</p>
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
            <span class="preview-title">v{selectedVersion.version_number} — {formatDate(selectedVersion.created_at)}</span>
            <button class="restore-btn" onclick={restoreVersion} title="Restore this version. A 'Before restore' snapshot of the current content is created first.">
              Restore this version
            </button>
          </div>
          <div class="preview-content">
            {#if previewLoading}
              <p class="preview-empty">Loading…</p>
            {:else}
              <!-- eslint-disable-next-line svelte/no-at-html-tags -->
              {@html previewHTML}
            {/if}
          </div>
        </div>
      {:else if versions.length > 0}
        <div class="version-preview placeholder">
          <p class="preview-empty">Select a version to preview it here.</p>
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
    width: 720px;
    max-width: 80vw;
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
    display: grid;
    grid-template-columns: 1fr;
    overflow: hidden;
    min-height: 0;
  }
  .panel-body.has-selection {
    grid-template-columns: 240px 1fr;
  }

  .version-list {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    border-right: 1px solid var(--color-border-subtle);
  }
  .panel-body:not(.has-selection) .version-list {
    border-right: none;
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
    font-size: 11px;
    color: var(--color-fg-tertiary);
  }

  .version-summary {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-secondary);
  }

  .version-preview {
    padding: 12px 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
    overflow: hidden;
  }
  .version-preview.placeholder {
    align-items: center;
    justify-content: center;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    flex-shrink: 0;
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
    border: 1px solid var(--color-border-subtle);
    padding: 16px 20px;
    min-height: 0;
  }

  .preview-empty {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    text-align: center;
    margin: 0;
  }

  /* Typography for the rendered HTML preview — approximates the editor's
     own styles. Keep selectors narrow so we don't bleed into other UI. */
  .preview-content :global(h1),
  .preview-content :global(h2),
  .preview-content :global(h3) {
    font-family: var(--font-heading);
    color: var(--color-fg-primary);
    margin: 16px 0 8px;
    line-height: 1.25;
  }
  .preview-content :global(h1) { font-size: 1.6rem; }
  .preview-content :global(h2) { font-size: 1.35rem; }
  .preview-content :global(h3) { font-size: 1.15rem; }
  .preview-content :global(p) {
    font-family: var(--font-body);
    font-size: 14px;
    color: var(--color-fg-primary);
    line-height: 1.55;
    margin: 8px 0;
  }
  .preview-content :global(a) {
    color: var(--color-accent-gold);
    text-decoration: none;
  }
  .preview-content :global(a:hover) {
    text-decoration: underline;
  }
  .preview-content :global(ul),
  .preview-content :global(ol) {
    margin: 8px 0;
    padding-left: 22px;
    font-family: var(--font-body);
    font-size: 14px;
    color: var(--color-fg-primary);
  }
  .preview-content :global(blockquote) {
    border-left: 3px solid var(--color-accent-gold);
    padding: 2px 14px;
    margin: 10px 0;
    color: var(--color-fg-secondary);
    font-style: italic;
  }
  .preview-content :global(code) {
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 0.9em;
    background: var(--color-surface-tertiary);
    padding: 1px 6px;
    border-radius: 3px;
  }
  .preview-content :global(pre) {
    background: var(--color-surface-tertiary);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    padding: 10px 14px;
    overflow-x: auto;
    font-family: 'JetBrains Mono', ui-monospace, monospace;
    font-size: 13px;
  }
  .preview-content :global(img) {
    max-width: 100%;
    height: auto;
    border-radius: var(--radius-sm);
  }
</style>
