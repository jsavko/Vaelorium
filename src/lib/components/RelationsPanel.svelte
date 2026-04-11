<script lang="ts">
  import { currentPageId, loadPage, pageTree } from '../stores/pageStore'
  import { relationTypes, currentPageRelations, loadPageRelations, loadRelationTypes, addRelation, removeRelation } from '../stores/relationStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import { onMount } from 'svelte'

  let adding = $state(false)
  let selectedTypeId = $state('')
  let searchQuery = $state('')
  let showResults = $state(false)

  onMount(() => {
    loadRelationTypes()
  })

  $effect(() => {
    const pageId = $currentPageId
    if (pageId) {
      loadPageRelations(pageId)
    }
  })

  let filteredPages = $derived(
    searchQuery.length > 0
      ? $pageTree
          .filter((p) => p.id !== $currentPageId && p.title.toLowerCase().includes(searchQuery.toLowerCase()))
          .slice(0, 8)
      : [],
  )

  async function handleAddRelation(targetPageId: string, targetTitle: string) {
    if (!$currentPageId || !selectedTypeId) return
    await addRelation($currentPageId, targetPageId, selectedTypeId)
    adding = false
    searchQuery = ''
    selectedTypeId = ''
    showResults = false
  }

  async function handleRemove(relationId: string) {
    if (!$currentPageId) return
    await removeRelation(relationId, $currentPageId)
  }

  function getEntityColor(typeId: string | null): string {
    if (!typeId) return 'var(--color-fg-tertiary)'
    return $entityTypeMap.get(typeId)?.color || 'var(--color-fg-tertiary)'
  }
</script>

<div class="section">
  <div class="section-header">
    <h3 class="section-label">RELATIONS</h3>
    <button class="add-btn" onclick={() => { adding = !adding }}>
      {adding ? '×' : '+'}
    </button>
  </div>

  {#if adding}
    <div class="add-form">
      <select class="type-select" bind:value={selectedTypeId}>
        <option value="">Select type...</option>
        {#each $relationTypes as rt (rt.id)}
          <option value={rt.id}>{rt.name}</option>
        {/each}
      </select>
      <div class="search-wrapper">
        <input
          class="search-input"
          placeholder="Search pages..."
          bind:value={searchQuery}
          onfocus={() => showResults = true}
        />
        {#if showResults && filteredPages.length > 0 && selectedTypeId}
          <div class="search-results">
            {#each filteredPages as page (page.id)}
              <button class="search-result" onclick={() => handleAddRelation(page.id, page.title)}>
                <span class="result-dot" style:background-color={getEntityColor(page.entity_type_id)}></span>
                {page.title}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if $currentPageRelations.length > 0}
    <div class="relation-list">
      {#each $currentPageRelations as rel (rel.id + rel.direction)}
        <div class="relation-row">
          <span class="rel-dot" style:background-color={getEntityColor(rel.page_entity_type_id)}></span>
          <button class="rel-link" onclick={() => loadPage(rel.page_id)}>
            {rel.page_title}
          </button>
          <span class="rel-type">— {rel.relation_label}</span>
          <button class="rel-remove" onclick={() => handleRemove(rel.id)} title="Remove relation">×</button>
        </div>
      {/each}
    </div>
  {:else if !adding}
    <p class="no-relations">No relations yet</p>
  {/if}
</div>

<style>
  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }

  .add-btn {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    font-size: 16px;
    padding: 0 4px;
  }

  .add-btn:hover {
    color: var(--color-accent-gold);
  }

  .add-form {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .type-select,
  .search-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-surface-primary);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    outline: none;
    box-sizing: border-box;
  }

  .search-input:focus {
    border-color: var(--color-accent-gold);
  }

  .search-wrapper {
    position: relative;
  }

  .search-results {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    z-index: 10;
    max-height: 200px;
    overflow-y: auto;
  }

  .search-result {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px 10px;
    background: none;
    border: none;
    text-align: left;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    cursor: pointer;
  }

  .search-result:hover {
    background: var(--color-surface-tertiary);
  }

  .result-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .relation-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .relation-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 0;
  }

  .relation-row:hover .rel-remove {
    opacity: 1;
  }

  .rel-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .rel-link {
    background: none;
    border: none;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-accent-gold);
    cursor: pointer;
    padding: 0;
    text-decoration: none;
  }

  .rel-link:hover {
    text-decoration: underline;
  }

  .rel-type {
    font-family: var(--font-ui);
    font-size: 11px;
    color: var(--color-fg-tertiary);
    flex: 1;
  }

  .rel-remove {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    font-size: 14px;
    padding: 0 2px;
    opacity: 0;
    transition: opacity 0.1s;
  }

  .rel-remove:hover {
    color: var(--color-status-error);
  }

  .no-relations {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
    margin: 0;
  }
</style>
