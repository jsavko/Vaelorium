<script lang="ts">
  import Editor from './Editor.svelte'
  import { currentPage } from '../stores/pageStore'

  interface Props {
    onToggleDetails: () => void
    detailsOpen: boolean
  }

  let { onToggleDetails, detailsOpen }: Props = $props()
</script>

<main class="main-content">
  <header class="toolbar">
    <div class="toolbar-left">
      {#if $currentPage}
        {#if $currentPage.parent_id}
          <span class="breadcrumb-parent">...</span>
          <span class="breadcrumb-sep">/</span>
        {/if}
        <span class="breadcrumb-current">{$currentPage.title}</span>
      {:else}
        <span class="breadcrumb-text">Welcome</span>
      {/if}
    </div>
    <div class="toolbar-right">
      {#if $currentPage}
        <button class="details-toggle" class:active={detailsOpen} onclick={onToggleDetails}>
          Details
        </button>
      {/if}
      <span class="sync-indicator">
        <span class="sync-dot"></span>
        <span class="sync-text">No sync</span>
      </span>
    </div>
  </header>
  <div class="divider"></div>
  <div class="editor-area">
    <Editor />
  </div>
</main>

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
    gap: 16px;
  }

  .breadcrumb-text,
  .breadcrumb-parent {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
  }

  .breadcrumb-sep {
    font-family: var(--font-ui);
    font-size: 13px;
    color: var(--color-fg-tertiary);
  }

  .breadcrumb-current {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 500;
    color: var(--color-fg-primary);
  }

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

  .details-toggle:hover {
    background: var(--color-surface-tertiary);
  }

  .details-toggle.active {
    background: var(--color-accent-gold-subtle);
    color: var(--color-accent-gold);
  }

  .sync-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sync-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-fg-tertiary);
  }

  .sync-text {
    font-family: var(--font-ui);
    font-size: 12px;
    color: var(--color-fg-tertiary);
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
