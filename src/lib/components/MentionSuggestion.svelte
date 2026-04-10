<script lang="ts">
  import type { PageTreeNode } from '../api/pages'

  interface Props {
    items: PageTreeNode[]
    selectedIndex: number
    onSelect: (item: PageTreeNode) => void
  }

  let { items, selectedIndex, onSelect }: Props = $props()

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
</script>

<div class="mention-dropdown">
  <div class="mention-header">LINK TO PAGE</div>
  {#if items.length === 0}
    <div class="mention-empty">No pages found</div>
  {:else}
    {#each items as item, index (item.id)}
      <button
        class="mention-item"
        class:selected={index === selectedIndex}
        onclick={() => onSelect(item)}
      >
        <span
          class="mention-dot"
          style:background-color={entityColors[item.entity_type_id ?? ''] || 'var(--color-fg-tertiary)'}
        ></span>
        <span class="mention-title">{item.title}</span>
        <span class="mention-spacer"></span>
        {#if item.entity_type_id}
          <span class="mention-type">{item.entity_type_id}</span>
        {/if}
      </button>
    {/each}
  {/if}
</div>

<style>
  .mention-dropdown {
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 6px;
    min-width: 280px;
    max-height: 300px;
    overflow-y: auto;
  }

  .mention-header {
    font-family: var(--font-ui);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
    padding: 6px 10px;
  }

  .mention-empty {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    padding: 10px;
    text-align: center;
  }

  .mention-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
  }

  .mention-item:hover,
  .mention-item.selected {
    background: var(--color-accent-gold-subtle);
  }

  .mention-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .mention-title {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

  .mention-spacer {
    flex: 1;
  }

  .mention-type {
    font-size: 11px;
    color: var(--color-fg-tertiary);
    text-transform: capitalize;
  }
</style>
