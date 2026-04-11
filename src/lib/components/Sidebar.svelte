<script lang="ts">
  import { onMount } from 'svelte'
  import PageTreeItem from './PageTreeItem.svelte'
  import ContextMenu from './ContextMenu.svelte'
  import ConfirmDialog from './ConfirmDialog.svelte'
  import { nestedTree, loadPageTree, createPage, pageTree, deleteCurrentPage, loadPage, currentPageId } from '../stores/pageStore'
  import { builtinTypes, customTypes } from '../stores/entityTypeStore'
  import type { PageTreeNode } from '../api/pages'
  import { deletePage } from '../api/pages'

  interface Props {
    onOpenSettings?: () => void
    onNewPage?: () => void
    onSelectType?: (typeId: string) => void
    activeTypeId?: string | null
  }

  let { onOpenSettings, onNewPage, onSelectType, activeTypeId = null }: Props = $props()

  // Collapsible sections with persisted state
  let typesCollapsed = $state(localStorage.getItem('vaelorium-types-collapsed') === 'true')
  let pagesCollapsed = $state(localStorage.getItem('vaelorium-pages-collapsed') === 'true')

  function toggleTypesCollapsed() {
    typesCollapsed = !typesCollapsed
    localStorage.setItem('vaelorium-types-collapsed', String(typesCollapsed))
  }

  function togglePagesCollapsed() {
    pagesCollapsed = !pagesCollapsed
    localStorage.setItem('vaelorium-pages-collapsed', String(pagesCollapsed))
  }

  const navItems = [
    { id: 'wiki', label: 'Wiki', active: true },
    { id: 'atlas', label: 'Atlas', active: false },
    { id: 'chronicle', label: 'Chronicle', active: false },
    { id: 'boards', label: 'Boards', active: false },
    { id: 'relations', label: 'Relations', active: false },
  ]

  onMount(() => {
    loadPageTree()
  })

  function handleNewPage() {
    if (onNewPage) {
      onNewPage()
    } else {
      createPage('Untitled Page')
    }
  }

  let hasPages = $derived($pageTree.length > 0)

  // Context menu state
  let contextMenu = $state<{ x: number; y: number; node: PageTreeNode } | null>(null)
  let deleteConfirm = $state<{ node: PageTreeNode } | null>(null)

  function handleContextMenu(e: MouseEvent, node: PageTreeNode) {
    contextMenu = { x: e.clientX, y: e.clientY, node }
  }

  function closeContextMenu() {
    contextMenu = null
  }

  async function handleNewChildPage() {
    if (!contextMenu) return
    const parentId = contextMenu.node.id
    closeContextMenu()
    await createPage('Untitled Page', parentId)
  }

  function handleDeletePage() {
    if (!contextMenu) return
    deleteConfirm = { node: contextMenu.node }
    closeContextMenu()
  }

  async function confirmDelete() {
    if (!deleteConfirm) return
    const node = deleteConfirm.node
    deleteConfirm = null
    await deletePage(node.id)
    if ($currentPageId === node.id) {
      await deleteCurrentPage()
    }
    await loadPageTree()
  }
</script>

<aside class="sidebar">
  <header class="sidebar-header">
    <h1 class="logo">Vaelorium</h1>
    <button class="settings-btn" onclick={() => onOpenSettings?.()} aria-label="Settings">
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="3"></circle>
        <path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"></path>
      </svg>
    </button>
  </header>

  <div class="divider"></div>

  <nav class="nav-section">
    {#each navItems as item}
      <button class="nav-item" class:active={item.active}>
        <span class="nav-label">{item.label}</span>
      </button>
    {/each}
  </nav>

  <div class="divider"></div>

  <div class="types-section">
    <div class="section-header collapsible">
      <button class="collapse-toggle" onclick={toggleTypesCollapsed}>
        <span class="section-chevron" class:collapsed={typesCollapsed}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </span>
        <span class="section-label">TYPES</span>
      </button>
    </div>
    {#if !typesCollapsed}
    <div class="type-list">
      {#each $builtinTypes as type (type.id)}
        <button
          class="type-item"
          class:active={activeTypeId === type.id}
          onclick={() => onSelectType?.(type.id)}
        >
          <span class="type-dot" style:background-color={type.color || 'var(--color-fg-tertiary)'}></span>
          <span class="type-name">{type.name}s</span>
        </button>
      {/each}
      {#each $customTypes as type (type.id)}
        <button
          class="type-item"
          class:active={activeTypeId === type.id}
          onclick={() => onSelectType?.(type.id)}
        >
          <span class="type-dot" style:background-color={type.color || 'var(--color-fg-tertiary)'}></span>
          <span class="type-name">{type.name}s</span>
        </button>
      {/each}
    </div>
    {/if}
  </div>

  <div class="divider"></div>

  <div class="page-tree-section">
    <div class="section-header collapsible">
      <button class="collapse-toggle" onclick={togglePagesCollapsed}>
        <span class="section-chevron" class:collapsed={pagesCollapsed}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </span>
        <span class="section-label">PAGES</span>
      </button>
      <button class="add-btn" title="New page" onclick={handleNewPage}>+</button>
    </div>

    {#if !pagesCollapsed}
      {#if hasPages}
        <div class="tree-list">
          {#each $nestedTree as node (node.id)}
            <PageTreeItem {node} onContextMenu={handleContextMenu} />
          {/each}
        </div>
      {:else}
        <div class="tree-placeholder">
          <button class="create-first" onclick={handleNewPage}>
            Create your first page
          </button>
        </div>
      {/if}
    {/if}
  </div>
</aside>

{#if contextMenu}
  <ContextMenu
    x={contextMenu.x}
    y={contextMenu.y}
    onClose={closeContextMenu}
    items={[
      { label: 'New child page', action: handleNewChildPage },
      { label: 'Delete', action: handleDeletePage, danger: true },
    ]}
  />
{/if}

{#if deleteConfirm}
  <ConfirmDialog
    open={true}
    title="Delete page"
    message={'Are you sure you want to delete "' + deleteConfirm.node.title + '"? This cannot be undone.'}
    confirmLabel="Delete"
    danger={true}
    onConfirm={confirmDelete}
    onCancel={() => deleteConfirm = null}
  />
{/if}

<style>
  .sidebar {
    width: 280px;
    min-width: 280px;
    height: 100%;
    background-color: var(--color-surface-secondary);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sidebar-header {
    padding: 16px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .logo {
    font-family: var(--font-heading);
    font-size: 20px;
    font-weight: 700;
    color: var(--color-accent-gold);
    margin: 0;
  }

  .settings-btn {
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
  }

  .settings-btn:hover {
    color: var(--color-fg-primary);
    background: var(--color-surface-tertiary);
  }

  .divider {
    height: 1px;
    background-color: var(--color-border-subtle);
    flex-shrink: 0;
  }

  .nav-section {
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    border: none;
    background: transparent;
    cursor: pointer;
    font-family: var(--font-ui);
    font-size: 14px;
    font-weight: 500;
    color: var(--color-fg-secondary);
    text-align: left;
    width: 100%;
  }

  .nav-item:hover {
    background-color: var(--color-surface-tertiary);
  }

  .nav-item.active {
    background-color: var(--color-accent-gold-subtle);
    color: var(--color-fg-primary);
  }

  .page-tree-section {
    flex: 1;
    padding: 12px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 4px 8px;
  }

  .section-label {
    font-family: var(--font-ui);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 2px;
    color: var(--color-fg-tertiary);
  }

  .collapse-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
  }

  .section-chevron {
    display: inline-flex;
    transition: transform 0.15s ease;
    color: var(--color-fg-tertiary);
  }

  .section-chevron.collapsed {
    transform: rotate(-90deg);
  }

  .types-section {
    padding: 8px 12px;
  }

  .type-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .type-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 5px 8px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    text-align: left;
    width: 100%;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-secondary);
  }

  .type-item:hover {
    background: var(--color-surface-tertiary);
  }

  .type-item.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-accent-gold);
  }

  .type-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .type-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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

  .tree-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .tree-placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
  }

  .create-first {
    background: none;
    border: 1px dashed var(--color-border-default);
    color: var(--color-fg-tertiary);
    font-family: var(--font-ui);
    font-size: 13px;
    padding: 12px 20px;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .create-first:hover {
    border-color: var(--color-accent-gold);
    color: var(--color-accent-gold);
  }
</style>
