<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { currentPageId } from '../stores/pageStore'

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

  $effect(() => {
    if (open && $currentPageId) {
      loadVersions($currentPageId)
    }
  })

  async function loadVersions(pageId: string) {
    try {
      versions = await callCommand('list_versions', { pageId })
    } catch {
      versions = []
    }
  }

  function formatDate(iso: string): string {
    const d = new Date(iso)
    return d.toLocaleString()
  }
</script>

{#if open}
  <div class="version-panel">
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

    <div class="version-list">
      {#if versions.length === 0}
        <p class="empty">No versions yet. Versions are created automatically every 5 minutes.</p>
      {:else}
        {#each versions as v (v.id)}
          <div class="version-item">
            <div class="version-header">
              <span class="version-num">v{v.version_number}</span>
              <span class="version-date">{formatDate(v.created_at)}</span>
            </div>
            {#if v.summary}
              <span class="version-summary">{v.summary}</span>
            {/if}
          </div>
        {/each}
      {/if}
    </div>
  </div>
{/if}

<style>
  .version-panel {
    position: fixed;
    right: 0;
    top: 0;
    width: 320px;
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

  .version-list {
    flex: 1;
    overflow-y: auto;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
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
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    display: flex;
    flex-direction: column;
    gap: 4px;
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
</style>
