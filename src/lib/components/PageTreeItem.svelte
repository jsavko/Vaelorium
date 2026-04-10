<script lang="ts">
  import { getChildren, currentPageId, loadPage, reorderPages } from '../stores/pageStore'
  import type { PageTreeNode } from '../api/pages'
  import PageTreeItem from './PageTreeItem.svelte'

  interface Props {
    node: PageTreeNode
    depth?: number
    onContextMenu?: (e: MouseEvent, node: PageTreeNode) => void
  }

  let { node, depth = 0, onContextMenu }: Props = $props()

  let expanded = $state(false)
  let children = $derived(getChildren(node.id))
  let isActive = $derived($currentPageId === node.id)
  let dragOver = $state(false)

  function toggle(e: MouseEvent) {
    e.stopPropagation()
    if (node.children_count > 0) {
      expanded = !expanded
    }
  }

  function select() {
    loadPage(node.id)
  }

  function handleDragStart(e: DragEvent) {
    e.dataTransfer?.setData('text/plain', node.id)
    e.dataTransfer!.effectAllowed = 'move'
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault()
    e.dataTransfer!.dropEffect = 'move'
    dragOver = true
  }

  function handleDragLeave() {
    dragOver = false
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault()
    dragOver = false
    const draggedId = e.dataTransfer?.getData('text/plain')
    if (!draggedId || draggedId === node.id) return

    // Drop on this node = make it a child
    await reorderPages([{ id: draggedId, parent_id: node.id, sort_order: 0 }])
    expanded = true
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault()
    onContextMenu?.(e, node)
  }

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

<div class="tree-item">
  <button
    class="tree-row"
    class:active={isActive}
    class:drag-over={dragOver}
    style:padding-left="{8 + depth * 20}px"
    ondblclick={toggle}
    onclick={select}
    oncontextmenu={handleContextMenu}
    draggable="true"
    ondragstart={handleDragStart}
    ondragover={handleDragOver}
    ondragleave={handleDragLeave}
    ondrop={handleDrop}
  >
    {#if node.children_count > 0}
      <button class="chevron" class:expanded onclick={toggle} aria-label="Toggle children">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      </button>
    {:else}
      <span class="chevron-spacer"></span>
    {/if}

    {#if node.icon}
      <span class="page-icon">{node.icon}</span>
    {:else if node.entity_type_id}
      <span class="entity-dot" style:background-color={entityColors[node.entity_type_id] || 'var(--color-fg-tertiary)'}></span>
    {:else}
      <span class="entity-dot" style:background-color="var(--color-fg-tertiary)"></span>
    {/if}

    <span class="page-title">{node.title}</span>
  </button>

  {#if expanded && node.children_count > 0}
    {#each children as child (child.id)}
      <PageTreeItem node={child} depth={depth + 1} {onContextMenu} />
    {/each}
  {/if}
</div>

<style>
  .tree-row {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    width: 100%;
    text-align: left;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
    transition: background-color 0.1s;
  }

  .tree-row:hover {
    background-color: var(--color-surface-tertiary);
  }

  .tree-row.active {
    background-color: var(--color-accent-gold-subtle);
    color: var(--color-fg-primary);
    font-weight: 500;
  }

  .tree-row.drag-over {
    background-color: var(--color-accent-gold-subtle);
    outline: 1px dashed var(--color-accent-gold);
  }

  .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    color: var(--color-fg-tertiary);
    transition: transform 0.15s;
    flex-shrink: 0;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
  }

  .chevron.expanded {
    transform: rotate(90deg);
  }

  .chevron-spacer {
    width: 14px;
    flex-shrink: 0;
  }

  .page-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .entity-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .page-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
