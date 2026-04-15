<script lang="ts">
  import { currentPageId, loadPage, reorderPages, pageTree } from '../stores/pageStore'
  import { entityTypeMap } from '../stores/entityTypeStore'
  import type { PageTreeNode } from '../api/pages'
  import { get } from 'svelte/store'
  import PageTreeItem from './PageTreeItem.svelte'

  interface Props {
    node: PageTreeNode
    depth?: number
    onContextMenu?: (e: MouseEvent, node: PageTreeNode) => void
  }

  let { node, depth = 0, onContextMenu }: Props = $props()

  let expanded = $state(false)
  // Derive children from $pageTree directly. Using `getChildren(node.id)`
  // here (which calls get(pageTree) internally) doesn't register a
  // reactive dependency — see feedback_derived_plus_get_is_not_reactive.
  // Deleting a nested page updated the root-level list fine but left
  // expanded parents showing stale children until re-mount.
  let children = $derived(
    $pageTree
      .filter((n) => n.parent_id === node.id)
      .sort((a, b) => a.sort_order - b.sort_order),
  )
  let isActive = $derived($currentPageId === node.id)
  let dropPosition = $state<'before' | 'inside' | 'after' | null>(null)

  function toggle(e: MouseEvent) {
    e.stopPropagation()
    if (node.children_count > 0) {
      expanded = !expanded
    }
  }

  function select() {
    loadPage(node.id)
    window.dispatchEvent(new CustomEvent('vaelorium:page-selected'))
  }

  function handleDragStart(e: DragEvent) {
    e.dataTransfer?.setData('text/plain', node.id)
    e.dataTransfer!.effectAllowed = 'move'
  }

  function handleDragEnter(e: DragEvent) {
    // Required on Windows WebView2 to register this row as a valid
    // drop target before `dragover` fires. No-op on macOS/Linux.
    e.preventDefault()
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move'
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault()
    e.dataTransfer!.dropEffect = 'move'

    // Calculate drop position based on cursor Y within the row
    const target = e.currentTarget as HTMLElement
    const rect = target.getBoundingClientRect()
    const y = e.clientY - rect.top
    const height = rect.height
    const ratio = y / height

    if (ratio < 0.25) {
      dropPosition = 'before'
    } else if (ratio > 0.75) {
      dropPosition = 'after'
    } else {
      dropPosition = 'inside'
    }
  }

  function handleDragLeave() {
    dropPosition = null
  }

  function getSiblings(parentId: string | null): PageTreeNode[] {
    const tree = get(pageTree)
    return tree
      .filter((n) => n.parent_id === parentId)
      .sort((a, b) => a.sort_order - b.sort_order)
  }

  function computeSortOrder(siblings: PageTreeNode[], targetIndex: number): number {
    if (siblings.length === 0) return 1000

    if (targetIndex <= 0) {
      // Before the first item
      return siblings[0].sort_order - 1000
    }
    if (targetIndex >= siblings.length) {
      // After the last item
      return siblings[siblings.length - 1].sort_order + 1000
    }

    // Between two items — use midpoint
    const before = siblings[targetIndex - 1].sort_order
    const after = siblings[targetIndex].sort_order
    return Math.floor((before + after) / 2)
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault()
    const pos = dropPosition
    dropPosition = null
    const draggedId = e.dataTransfer?.getData('text/plain')
    if (!draggedId || draggedId === node.id) return

    if (pos === 'inside') {
      // Make child of this node (existing behavior)
      const childSiblings = getSiblings(node.id)
      const sortOrder = childSiblings.length > 0
        ? childSiblings[childSiblings.length - 1].sort_order + 1000
        : 1000
      await reorderPages([{ id: draggedId, parent_id: node.id, sort_order: sortOrder }])
      expanded = true
    } else if (pos === 'before' || pos === 'after') {
      // Insert as sibling before/after this node
      const parentId = node.parent_id
      const siblings = getSiblings(parentId)
      const myIndex = siblings.findIndex((s) => s.id === node.id)
      const targetIndex = pos === 'before' ? myIndex : myIndex + 1
      const sortOrder = computeSortOrder(
        siblings.filter((s) => s.id !== draggedId),
        pos === 'before' ? myIndex : myIndex + 1,
      )
      await reorderPages([{ id: draggedId, parent_id: parentId, sort_order: sortOrder }])
    }
  }

  function handleContextMenu(e: MouseEvent) {
    e.preventDefault()
    onContextMenu?.(e, node)
  }

  function getEntityColor(typeId: string | null): string {
    if (!typeId) return 'var(--color-fg-tertiary)'
    const type = $entityTypeMap.get(typeId)
    return type?.color || 'var(--color-fg-tertiary)'
  }
</script>

<div class="tree-item">
  <!-- div (not button) so the inner chevron <button> isn't nested in another
       <button> which is invalid HTML. role + tabindex + onkeydown keep it
       keyboard-accessible. -->
  <div
    role="button"
    tabindex="0"
    class="tree-row"
    class:active={isActive}
    class:drop-before={dropPosition === 'before'}
    class:drop-inside={dropPosition === 'inside'}
    class:drop-after={dropPosition === 'after'}
    style:padding-left="{8 + depth * 20}px"
    ondblclick={toggle}
    onclick={select}
    onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); select(); } }}
    oncontextmenu={handleContextMenu}
    draggable="true"
    ondragstart={handleDragStart}
    ondragenter={handleDragEnter}
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
      <span class="entity-dot" style:background-color={getEntityColor(node.entity_type_id)}></span>
    {:else}
      <span class="entity-dot" style:background-color="var(--color-fg-tertiary)"></span>
    {/if}

    <span class="page-title">{node.title}</span>
  </div>

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
    position: relative;
  }

  .tree-row:hover {
    background-color: var(--color-surface-tertiary);
  }

  .tree-row.active {
    background-color: var(--color-accent-gold-subtle);
    color: var(--color-fg-primary);
    font-weight: 500;
  }

  /* Drop position indicators */
  .tree-row.drop-inside {
    background-color: var(--color-accent-gold-subtle);
    outline: 1px dashed var(--color-accent-gold);
  }

  .tree-row.drop-before::before {
    content: '';
    position: absolute;
    top: -1px;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--color-accent-gold);
    border-radius: 1px;
  }

  .tree-row.drop-after::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--color-accent-gold);
    border-radius: 1px;
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
    font-family: 'Apple Color Emoji', 'Segoe UI Emoji', 'Noto Color Emoji', 'Twemoji Mozilla', sans-serif;
    line-height: 1;
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
