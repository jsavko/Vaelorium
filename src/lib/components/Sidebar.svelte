<script lang="ts">
  import { onMount } from 'svelte'
  import PageTreeItem from './PageTreeItem.svelte'
  import ContextMenu from './ContextMenu.svelte'
  import ConfirmDialog from './ConfirmDialog.svelte'
  import { nestedTree, loadPageTree, createPage, pageTree, deleteCurrentPage, loadPage, currentPageId } from '../stores/pageStore'
  import { showToast } from '../stores/toastStore'
  import type { PageTreeNode } from '../api/pages'
  import { deletePage } from '../api/pages'

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

  async function handleNewPage() {
    const page = await createPage('Untitled Page')
    showToast('Page created', 'success')
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
    const page = await createPage('Untitled Page', parentId)
    showToast('Child page created', 'success')
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
    showToast(`"${node.title}" deleted`, 'info')
  }
</script>

<aside class="sidebar">
  <header class="sidebar-header">
    <h1 class="logo">Vaelorium</h1>
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

  <div class="page-tree-section">
    <div class="section-header">
      <span class="section-label">PAGES</span>
      <button class="add-btn" title="New page" onclick={handleNewPage}>+</button>
    </div>

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
