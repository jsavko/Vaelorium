<script lang="ts">
  import { callCommand } from '../api/bridge'
  import { loadPage, recentPageIds, pageTree } from '../stores/pageStore'
  import type { PageTreeNode } from '../api/pages'

  interface Props {
    open: boolean
    onClose: () => void
  }

  let { open, onClose }: Props = $props()

  interface SearchResult {
    page_id: string
    title: string
    entity_type_id: string | null
    snippet: string | null
  }

  let query = $state('')
  let results = $state<SearchResult[]>([])
  let selectedIndex = $state(0)
  let inputEl: HTMLInputElement
  let isSearching = $state(false)

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

  // Get recent pages from tree
  let recentPages = $derived(
    $recentPageIds
      .map((id) => $pageTree.find((p) => p.id === id))
      .filter(Boolean) as PageTreeNode[]
  )

  let displayItems = $derived(
    query.trim()
      ? results.map((r) => ({ id: r.page_id, title: r.title, entity_type_id: r.entity_type_id }))
      : recentPages.map((p) => ({ id: p.id, title: p.title, entity_type_id: p.entity_type_id }))
  )

  $effect(() => {
    if (open && inputEl) {
      query = ''
      results = []
      selectedIndex = 0
      setTimeout(() => inputEl?.focus(), 50)
    }
  })

  async function handleInput() {
    selectedIndex = 0
    if (query.trim().length === 0) {
      results = []
      return
    }
    isSearching = true
    try {
      results = await callCommand('search_pages', { query: query.trim() })
    } catch {
      results = []
    } finally {
      isSearching = false
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      selectedIndex = Math.min(selectedIndex + 1, displayItems.length - 1)
    } else if (e.key === 'ArrowUp') {
      e.preventDefault()
      selectedIndex = Math.max(selectedIndex - 1, 0)
    } else if (e.key === 'Enter') {
      e.preventDefault()
      const item = displayItems[selectedIndex]
      if (item) selectItem(item.id)
    } else if (e.key === 'Escape') {
      onClose()
    }
  }

  function selectItem(pageId: string) {
    loadPage(pageId)
    onClose()
  }
</script>

{#if open}
  <div class="overlay" onclick={onClose} onkeydown={handleKeydown} role="dialog" aria-modal="true">
    <div class="search-modal" onclick={(e) => e.stopPropagation()}>
      <div class="search-header">
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="var(--color-fg-tertiary)" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          bind:this={inputEl}
          class="search-input"
          placeholder="Search the archives..."
          bind:value={query}
          oninput={handleInput}
          onkeydown={handleKeydown}
        />
        <span class="shortcut-badge">Esc</span>
      </div>

      <div class="search-divider"></div>

      <div class="search-results">
        {#if !query.trim() && recentPages.length > 0}
          <div class="results-label">RECENT PAGES</div>
        {/if}

        {#each displayItems as item, index (item.id)}
          <button
            class="result-item"
            class:selected={index === selectedIndex}
            onclick={() => selectItem(item.id)}
          >
            <span
              class="result-dot"
              style:background-color={entityColors[item.entity_type_id ?? ''] || 'var(--color-fg-tertiary)'}
            ></span>
            <span class="result-title">{item.title}</span>
            <span class="result-spacer"></span>
            {#if item.entity_type_id}
              <span class="result-type">{item.entity_type_id}</span>
            {/if}
          </button>
        {/each}

        {#if query.trim() && displayItems.length === 0 && !isSearching}
          <div class="no-results">No pages found</div>
        {/if}
      </div>

      <div class="search-footer">
        <span class="footer-hint"><kbd>&uarr;&darr;</kbd> Navigate</span>
        <span class="footer-hint"><kbd>&crarr;</kbd> Open</span>
        <span class="footer-hint"><kbd>Esc</kbd> Close</span>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    justify-content: center;
    padding-top: 120px;
    z-index: 100;
  }

  .search-modal {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-lg);
    box-shadow: 0 12px 60px rgba(0, 0, 0, 0.3);
    width: 700px;
    max-height: 520px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .search-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 20px;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-family: var(--font-ui);
    font-size: 16px;
    color: var(--color-fg-primary);
  }

  .search-input::placeholder {
    color: var(--color-fg-tertiary);
  }

  .shortcut-badge {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 500;
    color: var(--color-fg-tertiary);
    background: var(--color-surface-tertiary);
    padding: 3px 8px;
    border-radius: 4px;
  }

  .search-divider {
    height: 1px;
    background: var(--color-border-subtle);
  }

  .search-results {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .results-label {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    padding: 6px 12px 8px;
  }

  .result-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
  }

  .result-item:hover,
  .result-item.selected {
    background: var(--color-accent-gold-subtle);
  }

  .result-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .result-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

  .result-spacer {
    flex: 1;
  }

  .result-type {
    font-size: 12px;
    color: var(--color-fg-tertiary);
    text-transform: capitalize;
  }

  .no-results {
    padding: 20px;
    text-align: center;
    font-family: var(--font-ui);
    font-size: 14px;
    color: var(--color-fg-tertiary);
  }

  .search-footer {
    display: flex;
    gap: 20px;
    padding: 10px 20px;
    background: var(--color-surface-secondary);
    border-top: 1px solid var(--color-border-subtle);
  }

  .footer-hint {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  kbd {
    font-family: var(--font-ui);
    font-size: 10px;
    background: var(--color-surface-tertiary);
    padding: 2px 6px;
    border-radius: 3px;
    color: var(--color-fg-tertiary);
  }
</style>
