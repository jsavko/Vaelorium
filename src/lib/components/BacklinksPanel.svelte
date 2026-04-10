<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { currentPageId, loadPage } from '../stores/pageStore'

  interface Backlink {
    page_id: string
    title: string
    entity_type_id: string | null
  }

  let backlinks = $state<Backlink[]>([])

  const entityColors: Record<string, string> = {
    character: 'var(--color-entity-character)',
    location: 'var(--color-entity-location)',
    quest: 'var(--color-entity-quest)',
    organisation: 'var(--color-entity-organisation)',
    item: 'var(--color-entity-item)',
    creature: 'var(--color-entity-creature)',
    event: 'var(--color-entity-event)',
    journal: 'var(--color-entity-journal)',
  }

  $effect(() => {
    const pageId = $currentPageId
    if (pageId) {
      loadBacklinks(pageId)
    } else {
      backlinks = []
    }
  })

  async function loadBacklinks(pageId: string) {
    try {
      backlinks = await callCommand('get_backlinks', { pageId })
    } catch {
      backlinks = []
    }
  }
</script>

<div class="backlinks-section">
  <h3 class="section-label">BACKLINKS</h3>
  {#if backlinks.length === 0}
    <p class="empty-text">No other pages link here yet</p>
  {:else}
    {#each backlinks as bl (bl.page_id)}
      <button class="backlink-item" onclick={() => loadPage(bl.page_id)}>
        <span
          class="bl-dot"
          style:background-color={entityColors[bl.entity_type_id ?? ''] || 'var(--color-fg-tertiary)'}
        ></span>
        <span class="bl-title">{bl.title}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .backlinks-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .empty-text {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .backlink-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    width: 100%;
    text-align: left;
  }

  .backlink-item:hover {
    border-color: var(--color-border-default);
  }

  .bl-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .bl-title {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-accent-gold);
  }
</style>
