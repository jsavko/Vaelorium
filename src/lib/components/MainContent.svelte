<script lang="ts">
  import Editor from './Editor.svelte'
  import ReadingView from './ReadingView.svelte'
  import VersionHistory from './VersionHistory.svelte'
  import ConfirmDialog from './ConfirmDialog.svelte'
  import { currentPage, deleteCurrentPage, pageTree, loadPage } from '../stores/pageStore'

  interface Props {
    onToggleDetails: () => void
    detailsOpen: boolean
  }

  let { onToggleDetails, detailsOpen }: Props = $props()

  let readingMode = $state(false)
  let moreMenuOpen = $state(false)
  let versionHistoryOpen = $state(false)
  let deleteConfirmOpen = $state(false)

  // Build breadcrumb chain
  let breadcrumbs = $derived.by(() => {
    const page = $currentPage
    if (!page) return []
    const tree = $pageTree
    const chain: { id: string; title: string }[] = []
    let current = page
    while (current?.parent_id) {
      const parent = tree.find((p) => p.id === current!.parent_id)
      if (parent) {
        chain.unshift({ id: parent.id, title: parent.title })
        current = { ...current, parent_id: parent.parent_id } as any
      } else break
    }
    return chain
  })

  function toggleMoreMenu() {
    moreMenuOpen = !moreMenuOpen
  }

  function handleVersionHistory() {
    moreMenuOpen = false
    versionHistoryOpen = !versionHistoryOpen
  }

  function handleDelete() {
    moreMenuOpen = false
    deleteConfirmOpen = true
  }

  async function confirmDelete() {
    deleteConfirmOpen = false
    await deleteCurrentPage()
  }
</script>

<svelte:window onclick={() => moreMenuOpen = false} />

<main class="main-content">
  <header class="toolbar">
    <div class="toolbar-left">
      {#if $currentPage}
        {#each breadcrumbs as crumb}
          <button class="breadcrumb-link" onclick={() => loadPage(crumb.id)}>{crumb.title}</button>
          <span class="breadcrumb-sep">/</span>
        {/each}
        <span class="breadcrumb-current">{$currentPage.title}</span>
      {:else}
        <span class="breadcrumb-text">Welcome</span>
      {/if}
    </div>
    <div class="toolbar-right">
      {#if $currentPage}
        <button
          class="mode-toggle"
          class:active={readingMode}
          onclick={() => readingMode = !readingMode}
        >
          {readingMode ? 'Edit' : 'Read'}
        </button>
        <button class="details-toggle" class:active={detailsOpen} onclick={onToggleDetails}>
          Details
        </button>
        <div class="more-menu-container">
          <button class="more-btn" onclick={(e) => { e.stopPropagation(); toggleMoreMenu(); }}>
            &#x22EF;
          </button>
          {#if moreMenuOpen}
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="more-dropdown" onclick={(e) => e.stopPropagation()}>
              <button class="more-item" onclick={handleVersionHistory}>Version History</button>
              <button class="more-item danger" onclick={handleDelete}>Delete Page</button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  </header>
  <div class="divider"></div>
  <div class="editor-area">
    {#if readingMode && $currentPage}
      <ReadingView />
    {:else}
      <Editor />
    {/if}
  </div>
</main>

<VersionHistory open={versionHistoryOpen} onClose={() => versionHistoryOpen = false} />

<ConfirmDialog
  open={deleteConfirmOpen}
  title="Delete Page"
  message={`Delete "${$currentPage?.title}"? This cannot be undone.`}
  confirmLabel="Delete"
  danger={true}
  onConfirm={confirmDelete}
  onCancel={() => deleteConfirmOpen = false}
/>

<style>
  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 24px;
    background-color: var(--color-surface-secondary);
    flex-shrink: 0;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .breadcrumb-text,
  .breadcrumb-sep {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
  }

  .breadcrumb-link {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
  }

  .breadcrumb-link:hover {
    color: var(--color-accent-gold);
  }

  .breadcrumb-current {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

  .mode-toggle,
  .details-toggle {
    font-family: var(--font-ui);
    font-size: 12px;
    font-weight: 500;
    padding: 5px 10px;
    border-radius: var(--radius-sm);
    border: none;
    cursor: pointer;
    background: transparent;
    color: var(--color-fg-tertiary);
  }

  .mode-toggle:hover,
  .details-toggle:hover {
    background: var(--color-surface-tertiary);
  }

  .mode-toggle.active,
  .details-toggle.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-accent-gold);
  }

  .more-menu-container {
    position: relative;
  }

  .more-btn {
    font-size: 18px;
    background: none;
    border: none;
    color: var(--color-fg-tertiary);
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
  }

  .more-btn:hover {
    background: var(--color-surface-tertiary);
    color: var(--color-fg-primary);
  }

  .more-dropdown {
    position: absolute;
    right: 0;
    top: 100%;
    margin-top: 4px;
    background: var(--color-surface-card);
    border: 1px solid var(--color-border-default);
    border-radius: var(--radius-md);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
    padding: 4px;
    min-width: 160px;
    z-index: 50;
  }

  .more-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    border: none;
    background: transparent;
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-primary);
    text-align: left;
    cursor: pointer;
    border-radius: var(--radius-sm);
  }

  .more-item:hover {
    background: var(--color-surface-tertiary);
  }

  .more-item.danger {
    color: var(--color-status-error);
  }

  .more-item.danger:hover {
    background: rgba(184, 92, 92, 0.15);
  }

  .divider {
    height: 1px;
    background-color: var(--color-border-subtle);
    flex-shrink: 0;
  }

  .editor-area {
    flex: 1;
    overflow-y: auto;
    display: flex;
  }
</style>
